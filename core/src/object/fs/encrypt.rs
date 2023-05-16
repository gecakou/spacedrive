use crate::{invalidate_query, job::*, library::Library, util::error::FileIOError};

use std::path::PathBuf;

use chrono::FixedOffset;
use sd_crypto::{
	crypto::Encryptor,
	header::{file::FileHeader, keyslot::Keyslot},
	primitives::{LATEST_FILE_HEADER, LATEST_KEYSLOT, LATEST_METADATA, LATEST_PREVIEW_MEDIA},
	types::{Algorithm, Key},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::{fs::File, io::AsyncReadExt};
use tracing::{error, warn};
use uuid::Uuid;

use super::{context_menu_fs_info, FsInfo, BYTES_EXT};

pub struct FileEncryptorJob;

#[derive(Serialize, Deserialize, Type, Hash)]
pub struct FileEncryptorJobInit {
	pub location_id: i32,
	pub path_id: i32,
	pub key_uuid: Uuid,
	pub algorithm: Algorithm,
	pub metadata: bool,
	pub preview_media: bool,
	pub output_path: Option<PathBuf>,
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
	pub path_id: i32,
	pub name: String,
	pub hidden: bool,
	pub favorite: bool,
	pub important: bool,
	pub note: Option<String>,
	pub date_created: chrono::DateTime<FixedOffset>,
}

impl JobInitData for FileEncryptorJobInit {
	type Job = FileEncryptorJob;
}

#[async_trait::async_trait]
impl StatefulJob for FileEncryptorJob {
	type Init = FileEncryptorJobInit;
	type Data = ();
	type Step = FsInfo;

	const NAME: &'static str = "file_encryptor";

	fn new() -> Self {
		Self {}
	}

	async fn init(&self, ctx: WorkerContext, state: &mut JobState<Self>) -> Result<(), JobError> {
		state.steps.push_back(
			context_menu_fs_info(&ctx.library.db, state.init.location_id, state.init.path_id)
				.await
				.map_err(|_| JobError::MissingData {
					value: String::from("file_path that matches both location id and path id"),
				})?,
		);

		ctx.progress(vec![JobReportUpdate::TaskCount(state.steps.len())]);

		Ok(())
	}

	async fn execute_step(
		&self,
		ctx: WorkerContext,
		state: &mut JobState<Self>,
	) -> Result<(), JobError> {
		let info = &state.steps[0];

		let Library { key_manager, .. } = &ctx.library;

		if !info.path_data.is_dir {
			// handle overwriting checks, and making sure there's enough available space

			let user_key = key_manager
				.access_keymount(state.init.key_uuid)
				.await?
				.hashed_key;

			let user_key_details = key_manager.access_keystore(state.init.key_uuid).await?;

			let output_path = state.init.output_path.clone().map_or_else(
				|| {
					let mut path = info.fs_path.clone();
					let extension = path.extension().map_or_else(
						|| Ok("bytes".to_string()),
						|extension| {
							Ok::<String, JobError>(
								extension
									.to_str()
									.ok_or(JobError::MissingData {
										value: String::from(
											"path contents when converted to string",
										),
									})?
									.to_string() + BYTES_EXT,
							)
						},
					)?;

					path.set_extension(extension);
					Ok::<PathBuf, JobError>(path)
				},
				Ok,
			)?;

			let _guard = ctx
				.library
				.location_manager()
				.temporary_ignore_events_for_path(
					state.init.location_id,
					ctx.library.clone(),
					&output_path,
				)
				.await
				.map_or_else(
					|e| {
						error!(
							"Failed to make location manager ignore the path {}; Error: {e:#?}",
							output_path.display()
						);
						None
					},
					Some,
				);

			let mut reader = File::open(&info.fs_path)
				.await
				.map_err(|e| FileIOError::from((&info.fs_path, e)))?;
			let mut writer = File::create(&output_path)
				.await
				.map_err(|e| FileIOError::from((output_path, e)))?;

			let master_key = Key::generate();

			let mut header = FileHeader::new(
				LATEST_FILE_HEADER,
				state.init.algorithm,
				vec![
					Keyslot::new(
						LATEST_KEYSLOT,
						state.init.algorithm,
						user_key_details.hashing_algorithm,
						user_key_details.content_salt,
						user_key,
						master_key.clone(),
					)
					.await?,
				],
			)?;

			if state.init.metadata || state.init.preview_media {
				// if any are requested, we can make the query as it'll be used at least once
				if let Some(ref object) = info.path_data.object {
					if state.init.metadata {
						let metadata = Metadata {
							path_id: state.init.path_id,
							name: info.path_data.materialized_path.clone(),
							hidden: object.hidden,
							favorite: object.favorite,
							important: object.important,
							note: object.note.clone(),
							date_created: object.date_created,
						};

						header
							.add_metadata(
								LATEST_METADATA,
								state.init.algorithm,
								master_key.clone(),
								&metadata,
							)
							.await?;
					}

					// if state.init.preview_media
					// 	&& (object.has_thumbnail
					// 		|| object.has_video_preview || object.has_thumbstrip)

					// may not be the best - pvm isn't guaranteed to be webp
					let pvm_path = ctx
						.library
						.config()
						.data_directory()
						.join("thumbnails")
						.join(info.path_data.cas_id.as_ref().unwrap())
						.with_extension("wepb");

					if tokio::fs::metadata(&pvm_path).await.is_ok() {
						let mut pvm_bytes = Vec::new();
						let mut pvm_file = File::open(&pvm_path)
							.await
							.map_err(|e| FileIOError::from((&pvm_path, e)))?;
						pvm_file
							.read_to_end(&mut pvm_bytes)
							.await
							.map_err(|e| FileIOError::from((pvm_path, e)))?;

						header
							.add_preview_media(
								LATEST_PREVIEW_MEDIA,
								state.init.algorithm,
								master_key.clone(),
								&pvm_bytes,
							)
							.await?;
					}
				} else {
					// should use container encryption if it's a directory
					warn!("skipping metadata/preview media inclusion, no associated object found")
				}
			}

			header.write(&mut writer).await?;

			let encryptor = Encryptor::new(master_key, header.nonce, header.algorithm)?;

			encryptor
				.encrypt_streams(&mut reader, &mut writer, &header.generate_aad())
				.await?;
		} else {
			warn!(
				"encryption is skipping {} as it isn't a file",
				info.path_data.materialized_path
			)
		}

		ctx.progress(vec![JobReportUpdate::CompletedTaskCount(
			state.step_number + 1,
		)]);

		Ok(())
	}

	async fn finalize(&mut self, ctx: WorkerContext, state: &mut JobState<Self>) -> JobResult {
		invalidate_query!(ctx.library, "search.paths");

		// mark job as successful
		Ok(Some(serde_json::to_value(&state.init)?))
	}
}
