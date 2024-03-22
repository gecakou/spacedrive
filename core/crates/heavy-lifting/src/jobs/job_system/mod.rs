use crate::Error;

use sd_prisma::prisma::PrismaClient;
use sd_task_system::TaskDispatcher;
use sd_utils::error::FileIOError;

use std::{cell::RefCell, collections::hash_map::HashMap, path::Path, sync::Arc};

use async_channel as chan;
use futures::Stream;
use futures_concurrency::future::{Join, TryJoin};
use tokio::{fs, spawn, sync::oneshot, task::JoinHandle};
use tracing::{error, info, trace, warn};
use uuid::Uuid;

use super::JobId;

pub(super) mod error;
pub(super) mod job;
pub(super) mod report;
mod runner;
mod store;

use error::JobSystemError;
use job::{IntoJob, Job, JobOutput};
use runner::{run, JobSystemRunner, RunnerMessage};
use store::{load_jobs, StoredJobEntry};

pub use store::SerializableJob;

const PENDING_JOBS_FILE: &str = "pending_jobs.bin";

#[derive(Debug, Clone, Copy)]
pub enum Command {
	Pause,
	Resume,
	Cancel,
}

pub struct JobSystem {
	msgs_tx: chan::Sender<RunnerMessage>,
	job_outputs_rx: chan::Receiver<(JobId, Result<JobOutput, JobSystemError>)>,
	runner_handle: RefCell<Option<JoinHandle<()>>>,
}

impl JobSystem {
	pub async fn new(
		dispatcher: TaskDispatcher<Error>,
		data_directory: impl AsRef<Path> + Send,
		dbs: &HashMap<Uuid, Arc<PrismaClient>>,
	) -> Result<Self, JobSystemError> {
		let (job_outputs_tx, job_outputs_rx) = chan::unbounded();
		let (job_return_status_tx, job_return_status_rx) = chan::bounded(16);
		let (msgs_tx, msgs_rx) = chan::bounded(8);

		let store_jobs_file = Arc::new(data_directory.as_ref().join(PENDING_JOBS_FILE));

		let runner_handle = RefCell::new(Some(spawn({
			let store_jobs_file = Arc::clone(&store_jobs_file);
			async move {
				trace!("Job System Runner starting...");
				while let Err(e) = spawn({
					let store_jobs_file = Arc::clone(&store_jobs_file);
					let dispatcher = dispatcher.clone();
					let job_return_status_tx = job_return_status_tx.clone();
					let job_return_status_rx = job_return_status_rx.clone();
					let job_outputs_tx = job_outputs_tx.clone();
					let msgs_rx = msgs_rx.clone();

					async move {
						run(
							JobSystemRunner::new(dispatcher, job_return_status_tx, job_outputs_tx),
							store_jobs_file.as_ref(),
							msgs_rx,
							job_return_status_rx,
						)
						.await;
					}
				})
				.await
				{
					if e.is_panic() {
						error!("Job system panicked: {e:#?}");
					} else {
						trace!("JobSystemRunner received shutdown signal and will exit...");
						break;
					}
					trace!("Restarting JobSystemRunner processing task...");
				}

				info!("JobSystemRunner gracefully shutdown");
			}
		})));

		load_stored_job_entries(store_jobs_file.as_ref(), dbs, &msgs_tx).await?;

		Ok(Self {
			msgs_tx,
			job_outputs_rx,
			runner_handle,
		})
	}

	/// Shutdown the job system
	/// # Panics
	/// Panics only happen if internal channels are unexpectedly closed
	pub async fn shutdown(&self) {
		if let Some(handle) = self
			.runner_handle
			.try_borrow_mut()
			.ok()
			.and_then(|mut maybe_handle| maybe_handle.take())
		{
			self.msgs_tx
				.send(RunnerMessage::Shutdown)
				.await
				.expect("runner msgs channel unexpectedly closed on shutdown request");

			if let Err(e) = handle.await {
				if e.is_panic() {
					error!("JobSystem panicked: {e:#?}");
				}
			}
			info!("JobSystem gracefully shutdown");
		} else {
			warn!("JobSystem already shutdown");
		}
	}

	/// Dispatch a new job to the system
	/// # Panics
	/// Panics only happen if internal channels are unexpectedly closed
	pub async fn dispatch<J: Job + SerializableJob>(
		&mut self,
		job: impl IntoJob<J> + Send,
		(db_id, db): (Uuid, Arc<PrismaClient>),
	) -> Result<JobId, JobSystemError> {
		let dyn_job = job.into_job();
		let id = dyn_job.id();

		let (ack_tx, ack_rx) = oneshot::channel();
		self.msgs_tx
			.send(RunnerMessage::NewJob {
				id,
				dyn_job,
				db_id,
				db,
				ack_tx,
			})
			.await
			.expect("runner msgs channel unexpectedly closed on new job request");

		ack_rx
			.await
			.expect("ack channel closed before receiving new job request")
			.map(|()| id)
	}

	pub fn receive_job_outputs(
		&self,
	) -> impl Stream<Item = (JobId, Result<JobOutput, JobSystemError>)> {
		self.job_outputs_rx.clone()
	}

	async fn send_command(&self, id: JobId, command: Command) -> Result<(), JobSystemError> {
		let (ack_tx, ack_rx) = oneshot::channel();
		self.msgs_tx
			.send(RunnerMessage::Command {
				id,
				command,
				ack_tx,
			})
			.await
			.unwrap_or_else(|_| {
				panic!("runner msgs channel unexpectedly closed on {command:?} request")
			});

		ack_rx
			.await
			.unwrap_or_else(|_| panic!("ack channel closed before receiving {command:?} response"))
	}

	pub async fn pause(&self, id: JobId) -> Result<(), JobSystemError> {
		self.send_command(id, Command::Pause).await
	}

	pub async fn resume(&self, id: JobId) -> Result<(), JobSystemError> {
		self.send_command(id, Command::Resume).await
	}

	pub async fn cancel(&self, id: JobId) -> Result<(), JobSystemError> {
		self.send_command(id, Command::Cancel).await
	}
}

/// SAFETY: Due to usage of refcell we lost `Sync` impl, but we only use it to have a shutdown method
/// receiving `&self` which is called once, and we also use `try_borrow_mut` so we never panic
unsafe impl Sync for JobSystem {}

async fn load_stored_job_entries(
	store_jobs_file: impl AsRef<Path> + Send,
	dbs: &HashMap<Uuid, Arc<PrismaClient>>,
	msgs_tx: &chan::Sender<RunnerMessage>,
) -> Result<(), JobSystemError> {
	let store_jobs_file = store_jobs_file.as_ref();

	let stores_jobs_by_db = rmp_serde::from_slice::<HashMap<Uuid, Vec<StoredJobEntry>>>(
		&fs::read(store_jobs_file).await.map_err(|e| {
			JobSystemError::StoredJobs(FileIOError::from((
				store_jobs_file,
				e,
				"Failed to load jobs from disk",
			)))
		})?,
	)?;

	stores_jobs_by_db
		.into_iter()
		.filter_map(|(db_id, entries)| {
			dbs.get(&db_id).map_or_else(
				|| {
					warn!("Found stored jobs for a database that doesn't exist anymore: <db_id='{db_id}'>");
					None
				},
				|db| Some((entries, db_id, Arc::clone(db))),
			)
		})
		.map(|(entries, db_id, db)| async move {
			load_jobs(entries, &db)
				.await
				.map(|stored_jobs| (stored_jobs, db_id, db))
		})
		.collect::<Vec<_>>()
		.join()
		.await
		.into_iter()
		.filter_map(|res| {
			res.map_err(|e| error!("Failed to load stored jobs: {e:#?}"))
				.ok()
		})
		.flat_map(|(stored_jobs, db_id, db)| {
			stored_jobs.into_iter().map(move |(dyn_job, dyn_tasks)| {
				let db = Arc::clone(&db);
				async move {
					let (ack_tx, ack_rx) = oneshot::channel();
					msgs_tx
						.send(RunnerMessage::ResumeStoredJob {
							id: dyn_job.id(),
							dyn_job,
							dyn_tasks,
							db_id,
							db,
							ack_tx,
						})
						.await
						.expect("runner msgs channel unexpectedly closed on stored job resume");

					ack_rx
						.await
						.expect("ack channel closed before receiving stored job resume response")
				}
			})
		})
		.collect::<Vec<_>>()
		.try_join()
		.await?;

	fs::remove_file(store_jobs_file).await.map_err(|e| {
		JobSystemError::StoredJobs(FileIOError::from((
			store_jobs_file,
			e,
			"Failed to clean stored jobs file",
		)))
	})
}