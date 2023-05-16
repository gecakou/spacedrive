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

use super::{context_menu_fs_info, get_location_path_from_location_id, FsInfo};

pub struct FileCutterJob {}

#[derive(Serialize, Deserialize, Hash, Type)]
pub struct FileCutterJobInit {
	pub source_location_id: i32,
	pub source_path_id: i32,
	pub target_location_id: i32,
	pub target_path: PathBuf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileCutterJobStep {
	pub source_fs_info: FsInfo,
	pub target_directory: PathBuf,
}

impl JobInitData for FileCutterJobInit {
	type Job = FileCutterJob;
}

#[async_trait::async_trait]
impl StatefulJob for FileCutterJob {
	type Init = FileCutterJobInit;
	type Data = ();
	type Step = FileCutterJobStep;

	const NAME: &'static str = "file_cutter";

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
		full_target_path.push(&state.init.target_path);

		state.steps.push_back(FileCutterJobStep {
			source_fs_info,
			target_directory: full_target_path,
		});

		ctx.progress(vec![JobReportUpdate::TaskCount(state.steps.len())]);

		Ok(())
	}

	async fn execute_step(
		&self,
		ctx: WorkerContext,
		state: &mut JobState<Self>,
	) -> Result<(), JobError> {
		let step = &state.steps[0];
		let source_info = &step.source_fs_info;

		let full_output = step
			.target_directory
			.join(source_info.fs_path.file_name().ok_or(JobError::OsStr)?);

		let parent_source = source_info.fs_path.parent().ok_or(JobError::Path)?;

		let parent_output = full_output.parent().ok_or(JobError::Path)?;

		if fs::canonicalize(parent_source)
			.await
			.map_err(|e| FileIOError::from((parent_source, e)))?
			== fs::canonicalize(parent_output)
				.await
				.map_err(|e| FileIOError::from((parent_output, e)))?
		{
			return Err(JobError::MatchingSrcDest(source_info.fs_path.clone()));
		}

		if fs::metadata(&full_output).await.is_ok() {
			warn!(
				"Skipping {} as it would be overwritten",
				full_output.display()
			);

			return Err(JobError::WouldOverwrite(full_output));
		}

		trace!(
			"Cutting {} to {}",
			source_info.fs_path.display(),
			full_output.display()
		);

		fs::rename(&source_info.fs_path, &full_output)
			.await
			.map_err(|e| FileIOError::from((&source_info.fs_path, e)))?;

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
