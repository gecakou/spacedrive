use crate::{utils::sub_path::get_full_path_from_sub_path, Error, NonCriticalJobError};

use sd_core_indexer_rules::{IndexerRule, IndexerRuler};
use sd_core_prisma_helpers::location_with_indexer_rules;
use sd_core_sync::Manager as SyncManager;

use sd_prisma::prisma::PrismaClient;
use sd_task_system::{BaseTaskDispatcher, CancelTaskOnDrop, IntoTask, TaskDispatcher, TaskOutput};
use sd_utils::db::maybe_missing;

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::Arc,
};

use futures_concurrency::future::TryJoin;
use itertools::Itertools;
use tracing::{debug, warn};

use super::{
	remove_non_existing_file_paths, reverse_update_directories_sizes,
	tasks::{
		saver::{SaveTask, SaveTaskOutput},
		updater::{UpdateTask, UpdateTaskOutput},
		walker::{ToWalkEntry, WalkDirTask, WalkTaskOutput, WalkedEntry},
	},
	update_directory_sizes, update_location_size, IndexerError, IsoFilePathFactory, WalkerDBProxy,
	BATCH_SIZE,
};

pub async fn shallow(
	location: location_with_indexer_rules::Data,
	sub_path: impl AsRef<Path> + Send,
	dispatcher: BaseTaskDispatcher<Error>,
	db: Arc<PrismaClient>,
	sync: Arc<SyncManager>,
	invalidate_query: impl Fn(&'static str) + Send + Sync,
) -> Result<Vec<NonCriticalJobError>, Error> {
	let sub_path = sub_path.as_ref();

	let location_path = maybe_missing(&location.path, "location.path")
		.map(PathBuf::from)
		.map(Arc::new)
		.map_err(IndexerError::from)?;

	let to_walk_path = Arc::new(
		get_full_path_from_sub_path(location.id, &Some(sub_path), &*location_path, &db)
			.await
			.map_err(IndexerError::from)?,
	);

	let Some(WalkTaskOutput {
		to_create,
		to_update,
		to_remove,
		mut errors,
		directory_iso_file_path,
		total_size,
		..
	}) = walk(
		&location,
		Arc::clone(&location_path),
		Arc::clone(&to_walk_path),
		Arc::clone(&db),
		&dispatcher,
	)
	.await?
	else {
		return Ok(vec![]);
	};

	let removed_count = remove_non_existing_file_paths(to_remove, &db, &sync).await?;

	let Some(Metadata {
		indexed_count,
		updated_count,
	}) = save_and_update(
		&location,
		to_create,
		to_update,
		Arc::clone(&db),
		Arc::clone(&sync),
		&dispatcher,
	)
	.await?
	else {
		return Ok(errors);
	};

	if indexed_count > 0 || removed_count > 0 || updated_count > 0 {
		update_directory_sizes(
			HashMap::from([(directory_iso_file_path, total_size)]),
			&db,
			&sync,
		)
		.await?;

		if to_walk_path != location_path {
			reverse_update_directories_sizes(
				&*to_walk_path,
				location.id,
				&*location_path,
				&db,
				&sync,
				&mut errors,
			)
			.await?;
		}

		update_location_size(location.id, &db, &invalidate_query).await?;
	}

	if indexed_count > 0 || removed_count > 0 {
		invalidate_query("search.paths");
	}

	Ok(errors)
}

async fn walk(
	location: &location_with_indexer_rules::Data,
	location_path: Arc<PathBuf>,
	to_walk_path: Arc<PathBuf>,
	db: Arc<PrismaClient>,
	dispatcher: &BaseTaskDispatcher<Error>,
) -> Result<Option<WalkTaskOutput>, Error> {
	match dispatcher
		.dispatch(WalkDirTask::new_shallow(
			ToWalkEntry::from(&*to_walk_path),
			to_walk_path,
			location
				.indexer_rules
				.iter()
				.map(|rule| IndexerRule::try_from(&rule.indexer_rule))
				.collect::<Result<Vec<_>, _>>()
				.map(IndexerRuler::new)
				.map_err(IndexerError::from)?,
			IsoFilePathFactory {
				location_id: location.id,
				location_path,
			},
			WalkerDBProxy {
				location_id: location.id,
				db,
			},
		)?)
		.await
		.await?
	{
		sd_task_system::TaskStatus::Done((_, TaskOutput::Out(data))) => Ok(Some(
			*data
				.downcast::<WalkTaskOutput>()
				.expect("we just dispatched this task"),
		)),
		sd_task_system::TaskStatus::Done((_, TaskOutput::Empty)) => {
			warn!("Shallow indexer's walker task finished without any output");
			Ok(None)
		}
		sd_task_system::TaskStatus::Error(e) => Err(e),

		sd_task_system::TaskStatus::Shutdown(_) => {
			debug!("Spacedrive is shuting down while a shallow indexer was in progress");
			Ok(None)
		}
		sd_task_system::TaskStatus::Canceled | sd_task_system::TaskStatus::ForcedAbortion => {
			unreachable!("WalkDirTask on shallow indexer can never be canceled or aborted")
		}
	}
}

struct Metadata {
	indexed_count: u64,
	updated_count: u64,
}

async fn save_and_update(
	location: &location_with_indexer_rules::Data,
	to_create: Vec<WalkedEntry>,
	to_update: Vec<WalkedEntry>,
	db: Arc<PrismaClient>,
	sync: Arc<SyncManager>,
	dispatcher: &BaseTaskDispatcher<Error>,
) -> Result<Option<Metadata>, Error> {
	let save_and_update_tasks = to_create
		.into_iter()
		.chunks(BATCH_SIZE)
		.into_iter()
		.map(|chunk| {
			SaveTask::new_shallow(
				location.id,
				location.pub_id.clone(),
				chunk.collect::<Vec<_>>(),
				Arc::clone(&db),
				Arc::clone(&sync),
			)
		})
		.map(IntoTask::into_task)
		.chain(
			to_update
				.into_iter()
				.chunks(BATCH_SIZE)
				.into_iter()
				.map(|chunk| {
					UpdateTask::new_shallow(
						chunk.collect::<Vec<_>>(),
						Arc::clone(&db),
						Arc::clone(&sync),
					)
				})
				.map(IntoTask::into_task),
		)
		.collect::<Vec<_>>();

	let mut metadata = Metadata {
		indexed_count: 0,
		updated_count: 0,
	};

	for task_status in dispatcher
		.dispatch_many_boxed(save_and_update_tasks)
		.await
		.into_iter()
		.map(CancelTaskOnDrop)
		.collect::<Vec<_>>()
		.try_join()
		.await?
	{
		match task_status {
			sd_task_system::TaskStatus::Done((_, TaskOutput::Out(data))) => {
				if data.is::<SaveTaskOutput>() {
					metadata.indexed_count += data
						.downcast::<SaveTaskOutput>()
						.expect("just checked")
						.saved_count;
				} else {
					metadata.updated_count += data
						.downcast::<UpdateTaskOutput>()
						.expect("just checked")
						.updated_count;
				}
			}
			sd_task_system::TaskStatus::Done((_, TaskOutput::Empty)) => {
				warn!("Shallow indexer's saver or updater task finished without any output");
				return Ok(None);
			}
			sd_task_system::TaskStatus::Error(e) => return Err(e),

			sd_task_system::TaskStatus::Shutdown(_) => {
				debug!("Spacedrive is shuting down while a shallow indexer was in progress");
				return Ok(None);
			}
			sd_task_system::TaskStatus::Canceled | sd_task_system::TaskStatus::ForcedAbortion => {
				unreachable!(
					"Save or Updater tasks on shallow indexer can never be canceled or aborted"
				);
			}
		}
	}

	Ok(Some(metadata))
}
