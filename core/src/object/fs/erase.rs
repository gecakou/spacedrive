use super::{context_menu_fs_info, FsInfo, ObjectType};
use crate::job::{JobError, JobReportUpdate, JobResult, JobState, StatefulJob, WorkerContext};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::{collections::VecDeque, fs::OpenOptions, hash::Hash, io::Write};

pub struct FileEraserJob {}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileEraserJobState {}

#[derive(Serialize, Deserialize, Hash, Type)]
pub struct FileEraserJobInit {
	pub location_id: i32,
	pub path_id: i32,
	pub passes: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileEraserJobStep {
	pub fs_info: FsInfo,
}

const JOB_NAME: &str = "file_eraser";

#[async_trait::async_trait]
impl StatefulJob for FileEraserJob {
	type Data = FileEraserJobState;
	type Init = FileEraserJobInit;
	type Step = FileEraserJobStep;

	fn name(&self) -> &'static str {
		JOB_NAME
	}

	async fn init(&self, ctx: WorkerContext, state: &mut JobState<Self>) -> Result<(), JobError> {
		let fs_info = context_menu_fs_info(
			&ctx.library_ctx.db,
			state.init.location_id,
			state.init.path_id,
		)
		.await?;

		state.steps = VecDeque::new();
		state.steps.push_back(FileEraserJobStep { fs_info });

		ctx.progress(vec![JobReportUpdate::TaskCount(state.steps.len())]);

		Ok(())
	}

	async fn execute_step(
		&self,
		ctx: WorkerContext,
		state: &mut JobState<Self>,
	) -> Result<(), JobError> {
		let step = &state.steps[0];
		let info = &step.fs_info;

		// need to handle stuff such as querying prisma for all paths of a file, and deleting all of those if requested (with a checkbox in the ui)
		// maybe a files.countOccurances/and or files.getPath(location_id, path_id) to show how many of these files would be erased (and where?)

		// will need to handle recursion here
		// if the match below is a directory, get all files within it, add them to the job steps queue, emit a job report update, repeat until all avenues are exhausted

		match info.obj_type {
			ObjectType::File => {
				let mut file = OpenOptions::new()
					.read(true)
					.write(true)
					.open(info.obj_path.clone())?;
				let file_len = file.metadata()?.len();

				sd_crypto::fs::erase::erase(&mut file, file_len as usize, 2)?;
				file.set_len(1)?;
				file.flush();
				drop(file);

				std::fs::remove_file(info.obj_path.clone())?;
			}
			ObjectType::Directory => todo!(),
		};

		ctx.progress(vec![JobReportUpdate::CompletedTaskCount(
			state.step_number + 1,
		)]);
		Ok(())
	}

	async fn finalize(&self, _ctx: WorkerContext, state: &mut JobState<Self>) -> JobResult {
		Ok(Some(serde_json::to_value(&state.init)?))
	}
}
