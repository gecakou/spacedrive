use crate::{
	invalidate_query,
	job::{
		JobError, JobInitData, JobReportUpdate, JobResult, JobState, StatefulJob, WorkerContext,
	},
	util::error::FileIOError,
};

use std::{hash::Hash, path::PathBuf};

use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::fs;
use tracing::{trace, warn};

use super::{context_menu_fs_info, get_location_path_from_location_id, osstr_to_string, FsInfo};

pub struct FileCopierJob {}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileCopierJobState {
	pub target_path: PathBuf, // target dir prefix too
	pub source_fs_info: FsInfo,
}

#[derive(Serialize, Deserialize, Hash, Type)]
pub struct FileCopierJobInit {
	pub source_location_id: i32,
	pub source_path_id: i32,
	pub target_location_id: i32,
	pub target_path: PathBuf,
	pub target_file_name_suffix: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileCopierJobStep {
	Directory { path: PathBuf },
	File { path: PathBuf },
}

impl From<FsInfo> for FileCopierJobStep {
	fn from(value: FsInfo) -> Self {
		if value.path_data.is_dir {
			Self::Directory {
				path: value.fs_path,
			}
		} else {
			Self::File {
				path: value.fs_path,
			}
		}
	}
}

impl JobInitData for FileCopierJobInit {
	type Job = FileCopierJob;
}

#[async_trait::async_trait]
impl StatefulJob for FileCopierJob {
	type Init = FileCopierJobInit;
	type Data = FileCopierJobState;
	type Step = FileCopierJobStep;

	const NAME: &'static str = "file_copier";

	fn new() -> Self {
		Self {}
	}

	async fn init(&self, ctx: WorkerContext, state: &mut JobState<Self>) -> Result<(), JobError> {
		let source_fs_info = context_menu_fs_info(
			&ctx.library.db,
			state.init.source_location_id,
			state.init.source_path_id,
		)
		.await?;

		let mut full_target_path =
			get_location_path_from_location_id(&ctx.library.db, state.init.target_location_id)
				.await?;

		// add the currently viewed subdirectory to the location root
		full_target_path.push(&state.init.target_path);

		// extension wizardry for cloning and such
		// if no suffix has been selected, just use the file name
		// if a suffix is provided and it's a directory, use the directory name + suffix
		// if a suffix is provided and it's a file, use the (file name + suffix).extension
		let file_name = osstr_to_string(source_fs_info.fs_path.file_name())?;

		let target_file_name = state.init.target_file_name_suffix.as_ref().map_or_else(
			|| Ok::<_, JobError>(file_name.clone()),
			|suffix| {
				Ok(if source_fs_info.path_data.is_dir {
					format!("{file_name}{suffix}")
				} else {
					osstr_to_string(source_fs_info.fs_path.file_stem())?
						+ suffix + &source_fs_info.fs_path.extension().map_or_else(
						|| Ok(String::new()),
						|ext| ext.to_str().map(|e| format!(".{e}")).ok_or(JobError::OsStr),
					)?
				})
			},
		)?;

		full_target_path.push(target_file_name);

		state.data = Some(FileCopierJobState {
			target_path: full_target_path,
			source_fs_info: source_fs_info.clone(),
		});

		state.steps.push_back(source_fs_info.into());

		ctx.progress(vec![JobReportUpdate::TaskCount(state.steps.len())]);

		Ok(())
	}

	async fn execute_step(
		&self,
		ctx: WorkerContext,
		state: &mut JobState<Self>,
	) -> Result<(), JobError> {
		let data = state
			.data
			.as_ref()
			.expect("critical error: missing data on job state");

		match &state.steps[0] {
			FileCopierJobStep::File { path } => {
				let mut target_path = data.target_path.clone();

				if data.source_fs_info.path_data.is_dir {
					// if root type is a dir, we need to preserve structure by making paths relative
					target_path.push(
						path.strip_prefix(&data.source_fs_info.fs_path)
							.map_err(|_| JobError::Path)?,
					);
				}

				let parent_path = path.parent().ok_or(JobError::Path)?;
				let parent_target_path = target_path.parent().ok_or(JobError::Path)?;

				if fs::canonicalize(parent_path)
					.await
					.map_err(|e| FileIOError::from((parent_path, e)))?
					== fs::canonicalize(parent_target_path)
						.await
						.map_err(|e| FileIOError::from((parent_target_path, e)))?
				{
					return Err(JobError::MatchingSrcDest(path.clone()));
				}

				if fs::metadata(&target_path).await.is_ok() {
					// only skip as it could be half way through a huge directory copy and run into an issue
					warn!(
						"Skipping {} as it would be overwritten",
						target_path.display()
					);
				// TODO(brxken128): could possibly return an error if the skipped file was the *only* file to be copied?
				} else {
					trace!(
						"Copying from {} to {}",
						path.display(),
						target_path.display()
					);

					fs::copy(&path, &target_path)
						.await
						.map_err(|e| FileIOError::from((&target_path, e)))?;
				}
			}
			FileCopierJobStep::Directory { path } => {
				// if this is the very first path, create the target dir
				// fixes copying dirs with no child directories
				if data.source_fs_info.path_data.is_dir && &data.source_fs_info.fs_path == path {
					fs::create_dir_all(&data.target_path)
						.await
						.map_err(|e| FileIOError::from((&data.target_path, e)))?;
				}

				let path = path.clone(); // To appease the borrowck

				let mut dir = fs::read_dir(&path)
					.await
					.map_err(|e| FileIOError::from((&path, e)))?;

				while let Some(entry) = dir
					.next_entry()
					.await
					.map_err(|e| FileIOError::from((&path, e)))?
				{
					let entry_path = entry.path();
					if entry
						.metadata()
						.await
						.map_err(|e| FileIOError::from((&entry_path, e)))?
						.is_dir()
					{
						state
							.steps
							.push_back(FileCopierJobStep::Directory { path: entry.path() });

						let full_path = data.target_path.join(
							entry_path
								.strip_prefix(&data.source_fs_info.fs_path)
								.map_err(|_| JobError::Path)?,
						);

						fs::create_dir_all(&full_path)
							.await
							.map_err(|e| FileIOError::from((full_path, e)))?;
					} else {
						state
							.steps
							.push_back(FileCopierJobStep::File { path: entry.path() });
					};

					ctx.progress(vec![JobReportUpdate::TaskCount(state.steps.len())]);
				}
			}
		};

		ctx.progress(vec![JobReportUpdate::CompletedTaskCount(
			state.step_number + 1,
		)]);
		Ok(())
	}

	async fn finalize(&mut self, ctx: WorkerContext, state: &mut JobState<Self>) -> JobResult {
		invalidate_query!(ctx.library, "search.paths");

		Ok(Some(serde_json::to_value(&state.init)?))
	}
}
