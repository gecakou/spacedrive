use crate::{
	finalize_indexer,
	job::{JobError, JobResult, JobState, StatefulJob, WorkerContext},
	location::file_path_helper::{get_max_file_path_id, set_max_file_path_id},
};

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

use chrono::Utc;
use itertools::Itertools;
use tokio::time::Instant;

use super::{
	ensure_sub_path_is_directory, ensure_sub_path_is_in_location,
	ensure_sub_path_parent_is_in_location, execute_indexer_step,
	rules::{IndexerRule, RuleKind},
	walk::{walk, WalkEntry},
	IndexerError, IndexerJobData, IndexerJobInit, IndexerJobStep, IndexerJobStepEntry,
	ScanProgress,
};

/// BATCH_SIZE is the number of files to index at each step, writing the chunk of files metadata in the database.
const BATCH_SIZE: usize = 1000;
pub const INDEXER_JOB_NAME: &str = "indexer";

/// A `IndexerJob` is a stateful job that walks a directory and indexes all files.
/// First it walks the directory and generates a list of files to index, chunked into
/// batches of [`BATCH_SIZE`]. Then for each chunk it write the file metadata to the database.
pub struct IndexerJob;

#[async_trait::async_trait]
impl StatefulJob for IndexerJob {
	type Init = IndexerJobInit;
	type Data = IndexerJobData;
	type Step = IndexerJobStep;

	fn name(&self) -> &'static str {
		INDEXER_JOB_NAME
	}

	/// Creates a vector of valid path buffers from a directory, chunked into batches of `BATCH_SIZE`.
	async fn init(&self, ctx: WorkerContext, state: &mut JobState<Self>) -> Result<(), JobError> {
		// grab the next id so we can increment in memory for batch inserting
		let first_file_id = get_max_file_path_id(&ctx.library_ctx)
			.await
			.map_err(IndexerError::from)?;

		let mut indexer_rules_by_kind: HashMap<RuleKind, Vec<IndexerRule>> =
			HashMap::with_capacity(state.init.location.indexer_rules.len());
		for location_rule in &state.init.location.indexer_rules {
			let indexer_rule = IndexerRule::try_from(&location_rule.indexer_rule)?;

			indexer_rules_by_kind
				.entry(indexer_rule.kind)
				.or_default()
				.push(indexer_rule);
		}

		let mut dirs_ids = HashMap::new();

		let to_walk_path = if let Some(ref sub_path) = state.init.sub_path {
			ensure_sub_path_is_in_location(&state.init.location.path, sub_path)?;
			ensure_sub_path_is_directory(sub_path).await?;

			let parent = ensure_sub_path_parent_is_in_location(
				state.init.location.id,
				sub_path,
				&ctx.library_ctx,
			)
			.await?;

			// If we're operating with a sub_path, then we have to put its parent on `dirs_ids` map
			dirs_ids.insert(
				PathBuf::from(&state.init.location.path).join(&parent.materialized_path),
				parent.id,
			);

			sub_path
		} else {
			Path::new(&state.init.location.path)
		};

		let scan_start = Instant::now();

		let paths = walk(
			to_walk_path,
			&indexer_rules_by_kind,
			|path, total_entries| {
				IndexerJobData::on_scan_progress(
					&ctx,
					vec![
						ScanProgress::Message(format!("Scanning {}", path.display())),
						ScanProgress::ChunkCount(total_entries / BATCH_SIZE),
					],
				);
			},
		)
		.await?;

		let total_paths = paths.len();
		let last_file_id = first_file_id + total_paths as i32;

		// Setting our global state for file_path ids
		set_max_file_path_id(last_file_id);

		let paths_entries = paths
			.into_iter()
			.zip(first_file_id..last_file_id)
			.map(
				|(
					WalkEntry {
						path,
						is_dir,
						created_at,
					},
					file_id,
				)| {
					let parent_id = if let Some(parent_dir) = path.parent() {
						dirs_ids.get(parent_dir).copied()
					} else {
						None
					};

					dirs_ids.insert(path.clone(), file_id);

					IndexerJobStepEntry {
						path,
						created_at,
						file_id,
						parent_id,
						is_dir,
					}
				},
			)
			.collect::<Vec<_>>();

		let total_entries = paths_entries.len();

		state.data = Some(IndexerJobData {
			db_write_start: Utc::now(),
			scan_read_time: scan_start.elapsed(),
			total_paths: total_entries,
			indexed_paths: 0,
		});

		state.steps = paths_entries
			.into_iter()
			.chunks(BATCH_SIZE)
			.into_iter()
			.enumerate()
			.map(|(i, chunk)| {
				let chunk_steps = chunk.collect::<Vec<_>>();
				IndexerJobData::on_scan_progress(
					&ctx,
					vec![
						ScanProgress::SavedChunks(i),
						ScanProgress::Message(format!(
							"Writing {} of {} to db",
							i * chunk_steps.len(),
							total_entries,
						)),
					],
				);
				chunk_steps
			})
			.collect();

		Ok(())
	}

	/// Process each chunk of entries in the indexer job, writing to the `file_path` table
	async fn execute_step(
		&self,
		ctx: WorkerContext,
		state: &mut JobState<Self>,
	) -> Result<(), JobError> {
		execute_indexer_step(&state.init.location, &state.steps[0], ctx)
			.await
			.map(|indexed_paths| {
				state
					.data
					.as_mut()
					.expect("critical error: missing data on job state")
					.indexed_paths = indexed_paths;
			})
	}

	/// Logs some metadata about the indexer job
	async fn finalize(&mut self, ctx: WorkerContext, state: &mut JobState<Self>) -> JobResult {
		finalize_indexer!(state, ctx)
	}
}
