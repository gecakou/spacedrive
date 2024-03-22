use crate::Error;

use chrono::Utc;
use sd_core_file_path_helper::IsolatedFilePathDataParts;
use sd_core_prisma_helpers::location_with_indexer_rules;
use sd_core_sync::Manager as SyncManager;

use sd_prisma::{
	prisma::{file_path, PrismaClient},
	prisma_sync,
};
use sd_sync::{sync_db_entry, OperationFactory};
use sd_task_system::{ExecStatus, Interrupter, IntoAnyTaskOutput, Task, TaskId};

use sd_utils::db::inode_to_db;
use serde_json::json;
use tracing::trace;

use std::sync::Arc;

use super::{walker::WalkedEntry, IndexerError};

#[derive(Debug)]
pub struct SaveTask {
	id: TaskId,
	location: Arc<location_with_indexer_rules::Data>,
	walked_entries: Vec<WalkedEntry>,
	db: Arc<PrismaClient>,
	sync: Arc<SyncManager>,
}

impl SaveTask {
	#[must_use]
	pub fn new(
		location: Arc<location_with_indexer_rules::Data>,
		walked_entries: Vec<WalkedEntry>,
		db: Arc<PrismaClient>,
		sync: Arc<SyncManager>,
	) -> Self {
		Self {
			id: TaskId::new_v4(),
			location,
			walked_entries,
			db,
			sync,
		}
	}
}

#[derive(Debug)]
pub struct SaveBatchTaskOutput {
	pub saved_count: i64,
}

#[async_trait::async_trait]
impl Task<Error> for SaveTask {
	fn id(&self) -> TaskId {
		self.id
	}

	async fn run(&mut self, _: &Interrupter) -> Result<ExecStatus, Error> {
		use file_path::{
			create_unchecked, date_created, date_indexed, date_modified, extension, hidden, inode,
			is_dir, location, location_id, materialized_path, name, size_in_bytes_bytes,
		};

		let Self {
			location,
			walked_entries,
			db,
			sync,
			..
		} = self;

		let (sync_stuff, paths): (Vec<_>, Vec<_>) = walked_entries
			.drain(..)
			.map(|entry| {
				let IsolatedFilePathDataParts {
					materialized_path,
					is_dir,
					name,
					extension,
					..
				} = entry.iso_file_path.to_parts();

				let pub_id = sd_utils::uuid_to_bytes(entry.pub_id);

				let (sync_params, db_params): (Vec<_>, Vec<_>) = [
					(
						(
							location::NAME,
							json!(prisma_sync::location::SyncId {
								pub_id: location.pub_id.clone()
							}),
						),
						location_id::set(Some(location.id)),
					),
					sync_db_entry!(materialized_path.to_string(), materialized_path),
					sync_db_entry!(name.to_string(), name),
					sync_db_entry!(is_dir, is_dir),
					sync_db_entry!(extension.to_string(), extension),
					sync_db_entry!(
						entry.metadata.size_in_bytes.to_be_bytes().to_vec(),
						size_in_bytes_bytes
					),
					sync_db_entry!(inode_to_db(entry.metadata.inode), inode),
					{
						let v = entry.metadata.created_at.into();
						sync_db_entry!(v, date_created)
					},
					{
						let v = entry.metadata.modified_at.into();
						sync_db_entry!(v, date_modified)
					},
					{
						let v = Utc::now().into();
						sync_db_entry!(v, date_indexed)
					},
					sync_db_entry!(entry.metadata.hidden, hidden),
				]
				.into_iter()
				.unzip();

				(
					sync.shared_create(
						prisma_sync::file_path::SyncId {
							pub_id: sd_utils::uuid_to_bytes(entry.pub_id),
						},
						sync_params,
					),
					create_unchecked(pub_id, db_params),
				)
			})
			.unzip();

		let saved_count = sync
			.write_ops(
				db,
				(
					sync_stuff.into_iter().flatten().collect(),
					db.file_path().create_many(paths).skip_duplicates(),
				),
			)
			.await
			.map_err(IndexerError::from)?;

		trace!("Inserted {saved_count} records");

		Ok(ExecStatus::Done(
			SaveBatchTaskOutput { saved_count }.into_output(),
		))
	}
}