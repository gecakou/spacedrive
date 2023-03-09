use crate::{
	invalidate_query,
	job::{JobError, JobReportUpdate, JobResult, WorkerContext},
	library::LibraryContext,
	location::file_path_helper::{
		file_path_just_id, file_path_just_id_materialized_path_date_created, FilePathError,
	},
	object::{cas::generate_cas_id, object_just_pub_id_with_file_paths_just_id_cas_id},
	prisma::{file_path, location, object, PrismaClient},
	sync,
	sync::SyncManager,
};

use sd_file_ext::{extensions::Extension, kind::ObjectKind};
use sd_sync::CRDTOperation;

use futures::future::join_all;
use int_enum::IntEnum;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
	collections::{HashMap, HashSet},
	path::{Path, PathBuf},
};
use thiserror::Error;
use tokio::{fs, io};
use tracing::{error, info};
use uuid::Uuid;

pub mod file_identifier_job;
pub mod shallow_file_identifier_job;

// we break these jobs into chunks of 100 to improve performance
const CHUNK_SIZE: usize = 100;

#[derive(Error, Debug)]
pub enum FileIdentifierJobError {
	#[error("File path related error (error: {0})")]
	FilePathError(#[from] FilePathError),

	#[error("Missing file_paths for a location")]
	MissingFilePaths,
}

#[derive(Debug, Clone)]
pub struct FileMetadata {
	pub cas_id: String,
	pub kind: ObjectKind,
	pub fs_metadata: std::fs::Metadata,
}

impl FileMetadata {
	/// Assembles `create_unchecked` params for a given file path
	pub async fn new(
		location_path: impl AsRef<Path>,
		materialized_path: impl AsRef<Path>, // TODO: use dedicated CreateUnchecked type
	) -> Result<FileMetadata, io::Error> {
		let path = location_path.as_ref().join(materialized_path.as_ref());

		let fs_metadata = fs::metadata(&path).await?;

		assert!(
			!fs_metadata.is_dir(),
			"We can't generate cas_id for directories"
		);

		// derive Object kind
		let kind = Extension::resolve_conflicting(&path, false)
			.await
			.map(Into::into)
			.unwrap_or(ObjectKind::Unknown);

		let cas_id = generate_cas_id(&path, fs_metadata.len()).await?;

		info!("Analyzed file: {:?} {:?} {:?}", path, cas_id, kind);

		Ok(FileMetadata {
			cas_id,
			kind,
			fs_metadata,
		})
	}
}

#[derive(Serialize, Deserialize, Debug)]
struct FilePathIdAndLocationIdCursor {
	file_path_id: i32,
	location_id: i32,
}

impl From<&FilePathIdAndLocationIdCursor> for file_path::UniqueWhereParam {
	fn from(cursor: &FilePathIdAndLocationIdCursor) -> Self {
		file_path::location_id_id(cursor.location_id, cursor.file_path_id)
	}
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FileIdentifierReport {
	location_path: PathBuf,
	total_orphan_paths: usize,
	total_objects_created: usize,
	total_objects_linked: usize,
	total_objects_ignored: usize,
}

async fn identifier_job_step(
	LibraryContext { db, sync, .. }: &LibraryContext,
	location: &location::Data,
	file_paths: &[file_path_just_id_materialized_path_date_created::Data],
) -> Result<(usize, usize), JobError> {
	let file_path_metas = join_all(file_paths.iter().map(|file_path| async move {
		FileMetadata::new(&location.path, &file_path.materialized_path)
			.await
			.map(|params| (file_path.id, (params, file_path)))
	}))
	.await
	.into_iter()
	.flat_map(|data| {
		if let Err(e) = &data {
			error!("Error assembling Object metadata: {e}");
		}

		data
	})
	.collect::<HashMap<i32, _>>();

	// Assign cas_id to each file path
	sync.write_ops(
		db,
		file_path_metas
			.iter()
			.map(|(id, (meta, _))| {
				(
					sync.owned_update(
						sync::file_path::SyncId {
							id: *id,
							location: sync::location::SyncId {
								pub_id: location.pub_id.clone(),
							},
						},
						[("cas_id", json!(&meta.cas_id))],
					),
					db.file_path().update(
						file_path::location_id_id(location.id, *id),
						vec![file_path::cas_id::set(Some(meta.cas_id.clone()))],
					),
				)
			})
			.unzip::<_, _, _, Vec<_>>(),
	)
	.await?;

	let unique_cas_ids = file_path_metas
		.values()
		.map(|(meta, _)| meta.cas_id.clone())
		.collect::<HashSet<_>>()
		.into_iter()
		.collect();

	// Retrieves objects that are already connected to file paths with the same id
	let existing_objects = db
		.object()
		.find_many(vec![object::file_paths::some(vec![
			file_path::cas_id::in_vec(unique_cas_ids),
		])])
		.select(object_just_pub_id_with_file_paths_just_id_cas_id::select())
		.exec()
		.await?;

	let existing_object_cas_ids = existing_objects
		.iter()
		.flat_map(|o| o.file_paths.iter().filter_map(|fp| fp.cas_id.as_ref()))
		.collect::<HashSet<_>>();

	// Attempt to associate each file path with an object that has been
	// connected to file paths with the same cas_id
	let updated_file_paths = sync
		.write_ops(
			db,
			file_path_metas
				.iter()
				.flat_map(|(id, (meta, _))| {
					existing_objects
						.iter()
						.find(|o| {
							o.file_paths
								.iter()
								.any(|fp| fp.cas_id.as_ref() == Some(&meta.cas_id))
						})
						.map(|o| (*id, o))
				})
				.map(|(id, object)| {
					file_path_object_connect_ops(
						id,
						// SAFETY: This pub_id is generated by the uuid lib, but we have to store bytes in sqlite
						Uuid::from_slice(&object.pub_id).unwrap(),
						location,
						sync,
						db,
					)
				})
				.unzip::<_, _, Vec<_>, Vec<_>>(),
		)
		.await?;

	info!(
		"Found {} existing Objects in Library, linking file paths...",
		existing_objects.len()
	);

	// extract objects that don't already exist in the database
	let file_paths_requiring_new_object = file_path_metas
		.into_iter()
		.filter(|(_, (meta, _))| !existing_object_cas_ids.contains(&meta.cas_id))
		.collect::<Vec<_>>();

	let total_created = if !file_paths_requiring_new_object.is_empty() {
		let new_objects_cas_ids = file_paths_requiring_new_object
			.iter()
			.map(|(_, (meta, _))| &meta.cas_id)
			.collect::<HashSet<_>>();

		info!(
			"Creating {} new Objects in Library... {:#?}",
			file_paths_requiring_new_object.len(),
			new_objects_cas_ids
		);

		let (object_create_args, file_path_update_args): (Vec<_>, Vec<_>) =
			file_paths_requiring_new_object
				.iter()
				.map(|(id, (meta, fp))| {
					let pub_id = Uuid::new_v4();
					let pub_id_vec = pub_id.as_bytes().to_vec();

					let sync_id = || sync::object::SyncId {
						pub_id: pub_id_vec.clone(),
					};

					let size = meta.fs_metadata.len().to_string();
					let kind = meta.kind.int_value();

					let object_creation_args = (
						[sync.shared_create(sync_id())]
							.into_iter()
							.chain(
								[
									("date_created", json!(fp.date_created)),
									("kind", json!(kind)),
									("size_in_bytes", json!(size)),
								]
								.into_iter()
								.map(|(f, v)| sync.shared_update(sync_id(), f, v)),
							)
							.collect::<Vec<_>>(),
						object::create_unchecked(
							pub_id_vec.clone(),
							vec![
								object::date_created::set(fp.date_created),
								object::kind::set(kind),
								object::size_in_bytes::set(size),
							],
						),
					);

					(
						object_creation_args,
						file_path_object_connect_ops(*id, pub_id, location, sync, db),
					)
				})
				.unzip();

		// create new object records with assembled values
		let total_created_files = sync
			.write_ops(db, {
				let (sync, db_params): (Vec<_>, Vec<_>) = object_create_args.into_iter().unzip();

				(sync.concat(), db.object().create_many(db_params))
			})
			.await
			.unwrap_or_else(|e| {
				error!("Error inserting files: {:#?}", e);
				0
			});

		info!("Created {} new Objects in Library", total_created_files);

		if total_created_files > 0 {
			sync.write_ops(db, {
				let (sync, db): (Vec<_>, Vec<_>) = file_path_update_args.into_iter().unzip();

				(sync, db)
			})
			.await?;
		}

		total_created_files as usize
	} else {
		0
	};

	Ok((total_created, updated_file_paths.len()))
}

fn file_path_object_connect_ops<'db>(
	file_path_id: i32,
	object_id: Uuid,
	location: &location::Data,
	sync: &SyncManager,
	db: &'db PrismaClient,
) -> (
	CRDTOperation,
	prisma_client_rust::Select<'db, file_path_just_id::Data>,
) {
	info!("Connecting <FilePath id={file_path_id}> to <Object pub_id={object_id}'>");

	(
		sync.owned_update(
			sync::file_path::SyncId {
				id: file_path_id,
				location: sync::location::SyncId {
					pub_id: location.pub_id.clone(),
				},
			},
			[("object", json!({ "pub_id": object_id }))],
		),
		db.file_path()
			.update(
				file_path::location_id_id(location.id, file_path_id),
				vec![file_path::object::connect(object::pub_id::equals(
					object_id.as_bytes().to_vec(),
				))],
			)
			.select(file_path_just_id::select()),
	)
}

async fn process_identifier_file_paths(
	job_name: &str,
	location: &location::Data,
	file_paths: &[file_path_just_id_materialized_path_date_created::Data],
	step_number: usize,
	cursor: &mut FilePathIdAndLocationIdCursor,
	report: &mut FileIdentifierReport,
	ctx: WorkerContext,
) -> Result<(), JobError> {
	// if no file paths found, abort entire job early, there is nothing to do
	// if we hit this error, there is something wrong with the data/query
	if file_paths.is_empty() {
		return Err(JobError::EarlyFinish {
			name: job_name.to_string(),
			reason: "Expected orphan Paths not returned from database query for this chunk"
				.to_string(),
		});
	}

	info!(
		"Processing {:?} orphan Paths. ({} completed of {})",
		file_paths.len(),
		step_number,
		report.total_orphan_paths
	);

	let (total_objects_created, total_objects_linked) =
		identifier_job_step(&ctx.library_ctx, location, file_paths).await?;

	report.total_objects_created += total_objects_created;
	report.total_objects_linked += total_objects_linked;

	// set the step data cursor to the last row of this chunk
	if let Some(last_row) = file_paths.last() {
		cursor.file_path_id = last_row.id;
	}

	ctx.progress(vec![
		JobReportUpdate::CompletedTaskCount(step_number),
		JobReportUpdate::Message(format!(
			"Processed {} of {} orphan Paths",
			step_number * CHUNK_SIZE,
			report.total_orphan_paths
		)),
	]);

	Ok(())
}

fn finalize_file_identifier(report: &FileIdentifierReport, ctx: WorkerContext) -> JobResult {
	info!("Finalizing identifier job: {report:?}");

	if report.total_orphan_paths > 0 {
		invalidate_query!(ctx.library_ctx, "locations.getExplorerData");
	}

	Ok(Some(serde_json::to_value(report)?))
}
