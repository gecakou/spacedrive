use crate::{Error, JobContext};

use sd_prisma::prisma::location;
use sd_task_system::BaseTaskDispatcher;
use sd_utils::error::FileIOError;

use std::{
	collections::{hash_map::Entry, HashMap, HashSet},
	mem,
	path::Path,
	pin::pin,
	time::Duration,
};

use async_channel as chan;
use chrono::Utc;
use futures::StreamExt;
use futures_concurrency::{
	future::{Join, TryJoin},
	stream::Merge,
};
use tokio::{
	fs,
	sync::oneshot,
	time::{interval_at, Instant},
};
use tokio_stream::wrappers::IntervalStream;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use super::{
	job::{DynJob, JobHandle, JobName, JobOutput, OuterContext, ReturnStatus},
	report,
	store::{StoredJob, StoredJobEntry},
	Command, JobId, JobSystemError, SerializedTasks,
};

const JOBS_INITIAL_CAPACITY: usize = 32;
const FIVE_MINUTES: Duration = Duration::from_secs(5 * 60);

pub(super) enum RunnerMessage<OuterCtx: OuterContext, JobCtx: JobContext<OuterCtx>> {
	NewJob {
		id: JobId,
		location_id: location::id::Type,
		dyn_job: Box<dyn DynJob<OuterCtx, JobCtx>>,
		ctx: OuterCtx,
		ack_tx: oneshot::Sender<Result<(), JobSystemError>>,
	},
	ResumeStoredJob {
		id: JobId,
		location_id: location::id::Type,
		dyn_job: Box<dyn DynJob<OuterCtx, JobCtx>>,
		ctx: OuterCtx,
		serialized_tasks: Option<SerializedTasks>,
		ack_tx: oneshot::Sender<Result<(), JobSystemError>>,
	},
	Command {
		id: JobId,
		command: Command,
		ack_tx: oneshot::Sender<Result<(), JobSystemError>>,
	},
	GetActiveReports {
		ack_tx: oneshot::Sender<HashMap<JobId, report::Report>>,
	},
	CheckIfJobAreRunning {
		job_names: Vec<JobName>,
		location_id: location::id::Type,
		ack_tx: oneshot::Sender<bool>,
	},
	Shutdown,
}

struct JobsWorktables {
	job_hashes: HashMap<u64, JobId>,
	job_hashes_by_id: HashMap<JobId, u64>,
	running_jobs_by_job_id: HashMap<JobId, (JobName, location::id::Type)>,
	running_jobs_set: HashSet<(JobName, location::id::Type)>,
	jobs_to_store_by_ctx_id: HashMap<Uuid, Vec<StoredJobEntry>>,
}

pub(super) struct JobSystemRunner<OuterCtx: OuterContext, JobCtx: JobContext<OuterCtx>> {
	base_dispatcher: BaseTaskDispatcher<Error>,
	handles: HashMap<JobId, JobHandle<OuterCtx, JobCtx>>,
	worktables: JobsWorktables,
	job_return_status_tx: chan::Sender<(JobId, Result<ReturnStatus, Error>)>,
	job_outputs_tx: chan::Sender<(JobId, Result<JobOutput, JobSystemError>)>,
}

impl<OuterCtx: OuterContext, JobCtx: JobContext<OuterCtx>> JobSystemRunner<OuterCtx, JobCtx> {
	pub(super) fn new(
		base_dispatcher: BaseTaskDispatcher<Error>,
		job_return_status_tx: chan::Sender<(JobId, Result<ReturnStatus, Error>)>,
		job_outputs_tx: chan::Sender<(JobId, Result<JobOutput, JobSystemError>)>,
	) -> Self {
		Self {
			base_dispatcher,
			handles: HashMap::with_capacity(JOBS_INITIAL_CAPACITY),
			worktables: JobsWorktables {
				job_hashes: HashMap::with_capacity(JOBS_INITIAL_CAPACITY),
				job_hashes_by_id: HashMap::with_capacity(JOBS_INITIAL_CAPACITY),
				running_jobs_by_job_id: HashMap::with_capacity(JOBS_INITIAL_CAPACITY),
				running_jobs_set: HashSet::with_capacity(JOBS_INITIAL_CAPACITY),
				jobs_to_store_by_ctx_id: HashMap::new(),
			},
			job_return_status_tx,
			job_outputs_tx,
		}
	}

	async fn new_job(
		&mut self,
		id: JobId,
		location_id: location::id::Type,
		dyn_job: Box<dyn DynJob<OuterCtx, JobCtx>>,
		ctx: OuterCtx,
		maybe_existing_tasks: Option<SerializedTasks>,
	) -> Result<(), JobSystemError> {
		let Self {
			base_dispatcher,
			handles,
			worktables:
				JobsWorktables {
					job_hashes,
					job_hashes_by_id,
					running_jobs_by_job_id,
					running_jobs_set,
					..
				},
			job_return_status_tx,
			..
		} = self;

		let db = ctx.db();
		let job_name = dyn_job.job_name();

		let job_hash = dyn_job.hash();
		if let Some(&already_running_id) = job_hashes.get(&job_hash) {
			return Err(JobSystemError::AlreadyRunning {
				new_id: id,
				already_running_id,
				job_name,
			});
		}

		running_jobs_by_job_id.insert(id, (job_name, location_id));
		running_jobs_set.insert((job_name, location_id));

		job_hashes.insert(job_hash, id);
		job_hashes_by_id.insert(id, job_hash);

		let start_time = Utc::now();

		let mut handle = if maybe_existing_tasks.is_some() {
			dyn_job.resume(
				base_dispatcher.clone(),
				ctx.clone(),
				maybe_existing_tasks,
				job_return_status_tx.clone(),
			)
		} else {
			dyn_job.dispatch(
				base_dispatcher.clone(),
				ctx.clone(),
				job_return_status_tx.clone(),
			)
		};

		handle.register_start(Utc::now()).await?;

		{
			let mut report = handle.ctx.report_mut().await;

			report.status = report::Status::Running;
			if report.started_at.is_none() {
				report.started_at = Some(start_time);
			}

			// If the report doesn't have a created_at date, it's a new report
			if report.created_at.is_none() {
				report.create(db).await?;
			} else {
				// Otherwise it can be a job being resumed or a children job that was already been created
				report.update(db).await?;
			}
		}

		// Registering children jobs
		handle
			.next_jobs
			.iter_mut()
			.map(|dyn_job| dyn_job.report_mut())
			.map(|next_job_report| async {
				if next_job_report.created_at.is_none() {
					next_job_report.create(ctx.db()).await
				} else {
					Ok(())
				}
			})
			.collect::<Vec<_>>()
			.try_join()
			.await?;

		handles.insert(id, handle);

		Ok(())
	}

	async fn get_active_reports(&self) -> HashMap<JobId, report::Report> {
		self.handles
			.iter()
			.map(|(job_id, handle)| async { (*job_id, handle.ctx.report().await.clone()) })
			.collect::<Vec<_>>()
			.join()
			.await
			.into_iter()
			.collect()
	}

	async fn process_command(&mut self, id: JobId, command: Command) -> Result<(), JobSystemError> {
		if let Some(handle) = self.handles.get_mut(&id) {
			handle.send_command(command).await?;
			Ok(())
		} else {
			Err(JobSystemError::NotFound(id))
		}
	}

	fn is_empty(&self) -> bool {
		self.handles.is_empty()
			&& self.worktables.job_hashes.is_empty()
			&& self.worktables.job_hashes_by_id.is_empty()
	}

	fn total_jobs(&self) -> usize {
		self.handles.len()
	}

	fn check_if_job_are_running(
		&self,
		job_names: Vec<JobName>,
		location_id: location::id::Type,
	) -> bool {
		job_names.into_iter().any(|job_name| {
			self.worktables
				.running_jobs_set
				.contains(&(job_name, location_id))
		})
	}

	async fn process_return_status(
		&mut self,
		job_id: JobId,
		status: Result<ReturnStatus, Error>,
	) -> Result<(), JobSystemError> {
		let Self {
			handles,
			worktables,
			job_outputs_tx,
			job_return_status_tx,
			base_dispatcher,
			..
		} = self;

		let job_hash = worktables
			.job_hashes_by_id
			.remove(&job_id)
			.expect("it must be here");

		let (job_name, location_id) = worktables
			.running_jobs_by_job_id
			.remove(&job_id)
			.expect("a JobName and location_id must've been inserted in the map with the job id");

		assert!(worktables.running_jobs_set.remove(&(job_name, location_id)));

		assert!(worktables.job_hashes.remove(&job_hash).is_some());
		let mut handle = handles.remove(&job_id).expect("it must be here");

		let res = match status {
			Ok(ReturnStatus::Completed(job_return)) => {
				try_dispatch_next_job(
					&mut handle,
					location_id,
					base_dispatcher.clone(),
					worktables,
					handles,
					job_return_status_tx.clone(),
				)
				.await?;

				handle.complete_job(job_return).await
			}

			Ok(ReturnStatus::Shutdown(Ok(Some(serialized_job)))) => {
				let name = handle.ctx.report().await.name;

				let Some(next_jobs) =
					serialize_next_jobs_to_shutdown(job_id, job_name, handle.next_jobs).await
				else {
					return Ok(());
				};

				worktables
					.jobs_to_store_by_ctx_id
					.entry(handle.ctx.id())
					.or_default()
					.push(StoredJobEntry {
						location_id,
						root_job: StoredJob {
							id: job_id,
							name,
							serialized_job,
						},
						next_jobs,
					});

				debug!("Job was shutdown and serialized: <id='{job_id}', name='{name}'>");

				return Ok(());
			}

			Ok(ReturnStatus::Shutdown(Ok(None))) => {
				debug!(
					"Job was shutdown but didn't returned any serialized data, \
					probably it isn't resumable job: <id='{job_id}'>"
				);
				return Ok(());
			}

			Ok(ReturnStatus::Shutdown(Err(e))) => {
				error!("Failed to serialize job: {e:#?}");
				return Ok(());
			}

			Ok(ReturnStatus::Canceled) => handle
				.cancel_job()
				.await
				.and_then(|()| Err(JobSystemError::Canceled(job_id))),

			Err(e) => handle.failed_job(&e).await.and_then(|()| Err(e.into())),
		};

		job_outputs_tx
			.send((job_id, res))
			.await
			.expect("job outputs channel unexpectedly closed on job completion");

		Ok(())
	}

	fn clean_memory(&mut self) {
		if self.handles.capacity() > JOBS_INITIAL_CAPACITY
			&& self.handles.len() < JOBS_INITIAL_CAPACITY
		{
			self.handles.shrink_to(JOBS_INITIAL_CAPACITY);
		}

		if self.worktables.job_hashes.capacity() > JOBS_INITIAL_CAPACITY
			&& self.worktables.job_hashes.len() < JOBS_INITIAL_CAPACITY
		{
			self.worktables.job_hashes.shrink_to(JOBS_INITIAL_CAPACITY);
		}

		if self.worktables.job_hashes_by_id.capacity() > JOBS_INITIAL_CAPACITY
			&& self.worktables.job_hashes_by_id.len() < JOBS_INITIAL_CAPACITY
		{
			self.worktables
				.job_hashes_by_id
				.shrink_to(JOBS_INITIAL_CAPACITY);
		}

		if self.worktables.running_jobs_by_job_id.capacity() > JOBS_INITIAL_CAPACITY
			&& self.worktables.running_jobs_by_job_id.len() < JOBS_INITIAL_CAPACITY
		{
			self.worktables
				.running_jobs_by_job_id
				.shrink_to(JOBS_INITIAL_CAPACITY);
		}

		if self.worktables.running_jobs_set.capacity() > JOBS_INITIAL_CAPACITY
			&& self.worktables.running_jobs_set.len() < JOBS_INITIAL_CAPACITY
		{
			self.worktables
				.running_jobs_set
				.shrink_to(JOBS_INITIAL_CAPACITY);
		}
	}

	async fn save_jobs(
		self,
		store_jobs_file: impl AsRef<Path> + Send,
	) -> Result<(), JobSystemError> {
		let store_jobs_file = store_jobs_file.as_ref();

		let Self {
			handles,
			worktables:
				JobsWorktables {
					job_hashes,
					job_hashes_by_id,
					jobs_to_store_by_ctx_id,
					..
				},
			..
		} = self;

		assert!(
			handles.is_empty() && job_hashes.is_empty() && job_hashes_by_id.is_empty(),
			"All jobs must be completed before saving"
		);

		if jobs_to_store_by_ctx_id.is_empty() {
			info!("No jobs to store in disk for job system shutdown!");
			return Ok(());
		}

		fs::write(
			store_jobs_file,
			rmp_serde::to_vec_named(&jobs_to_store_by_ctx_id)?,
		)
		.await
		.map_err(|e| JobSystemError::StoredJobs(FileIOError::from((store_jobs_file, e))))
	}
}

async fn serialize_next_jobs_to_shutdown<OuterCtx: OuterContext, JobCtx: JobContext<OuterCtx>>(
	parent_job_id: JobId,
	parent_job_name: JobName,
	next_jobs: impl IntoIterator<Item = Box<dyn DynJob<OuterCtx, JobCtx>>> + Send,
) -> Option<Vec<StoredJob>> {
	next_jobs
		.into_iter()
		.map(|next_job| async move {
			let next_id = next_job.id();
			let next_name = next_job.job_name();
			next_job
				.serialize()
				.await
				.map(|maybe_serialized_job| {
					maybe_serialized_job.map(|serialized_job| StoredJob {
						id: next_id,
						name: next_name,
						serialized_job,
					})
				})
				.map_err(|e| {
					error!(
						"Failed to serialize next job: <parent_id='{parent_job_id}', parent_name='{parent_job_name}', \
						next_id='{next_id}', next_name='{next_name}'>: {e:#?}"
					);
				})
		})
		.collect::<Vec<_>>()
		.try_join()
		.await
		.map(|maybe_serialized_next_jobs| {
			maybe_serialized_next_jobs.into_iter().flatten().collect()
		})
		.ok()
}

async fn try_dispatch_next_job<OuterCtx: OuterContext, JobCtx: JobContext<OuterCtx>>(
	handle: &mut JobHandle<OuterCtx, JobCtx>,
	location_id: location::id::Type,
	base_dispatcher: BaseTaskDispatcher<Error>,
	JobsWorktables {
		job_hashes,
		job_hashes_by_id,
		running_jobs_by_job_id,
		running_jobs_set,
		..
	}: &mut JobsWorktables,
	handles: &mut HashMap<JobId, JobHandle<OuterCtx, JobCtx>>,
	job_return_status_tx: chan::Sender<(JobId, Result<ReturnStatus, Error>)>,
) -> Result<(), JobSystemError> {
	if let Some(next) = handle.next_jobs.pop_front() {
		let next_id = next.id();
		let next_hash = next.hash();
		let next_name = next.job_name();

		if let Entry::Vacant(e) = job_hashes.entry(next_hash) {
			e.insert(next_id);
			trace!(
				"Dispatching next job: <id='{next_id}', name='{}'>",
				next.job_name()
			);
			job_hashes_by_id.insert(next_id, next_hash);
			running_jobs_by_job_id.insert(next_id, (next_name, location_id));
			running_jobs_set.insert((next_name, location_id));
			let mut next_handle = next.dispatch(
				base_dispatcher,
				handle.ctx.get_outer_ctx(),
				job_return_status_tx,
			);
			next_handle.register_start(Utc::now()).await?;

			assert!(
				next_handle.next_jobs.is_empty(),
				"Only the root job will have next jobs, the rest will be empty and \
				we will swap with remaining ones from the previous job"
			);

			next_handle.next_jobs = mem::take(&mut handle.next_jobs);

			handles.insert(next_id, next_handle);
		} else {
			warn!("Unexpectedly found a job with the same hash as the next job: <id='{next_id}', name='{}'>", next.job_name());
		}
	} else {
		trace!("No next jobs to dispatch");
	}

	Ok(())
}

pub(super) async fn run<OuterCtx: OuterContext, JobCtx: JobContext<OuterCtx>>(
	mut runner: JobSystemRunner<OuterCtx, JobCtx>,
	store_jobs_file: impl AsRef<Path> + Send,
	msgs_rx: chan::Receiver<RunnerMessage<OuterCtx, JobCtx>>,
	job_return_status_rx: chan::Receiver<(JobId, Result<ReturnStatus, Error>)>,
) {
	enum StreamMessage<OuterCtx: OuterContext, JobCtx: JobContext<OuterCtx>> {
		ReturnStatus((JobId, Result<ReturnStatus, Error>)),
		RunnerMessage(RunnerMessage<OuterCtx, JobCtx>),
		CleanMemoryTick,
	}

	let memory_cleanup_interval = interval_at(Instant::now() + FIVE_MINUTES, FIVE_MINUTES);

	let job_return_status_rx_to_shutdown = job_return_status_rx.clone();

	let mut msg_stream = pin!((
		msgs_rx.map(StreamMessage::RunnerMessage),
		job_return_status_rx.map(StreamMessage::ReturnStatus),
		IntervalStream::new(memory_cleanup_interval).map(|_| StreamMessage::CleanMemoryTick),
	)
		.merge());

	while let Some(msg) = msg_stream.next().await {
		match msg {
			// Job return status messages
			StreamMessage::ReturnStatus((job_id, status)) => {
				trace!("Received return status for job: <id='{job_id}', status='{status:#?}'>");
				if let Err(e) = runner.process_return_status(job_id, status).await {
					error!("Failed to process return status: {e:#?}");
				}
			}

			// Runner messages
			StreamMessage::RunnerMessage(RunnerMessage::NewJob {
				id,
				location_id,
				dyn_job,
				ctx,
				ack_tx,
			}) => {
				ack_tx
					.send(runner.new_job(id, location_id, dyn_job, ctx, None).await)
					.expect("ack channel closed before sending new job response");
			}

			StreamMessage::RunnerMessage(RunnerMessage::GetActiveReports { ack_tx }) => {
				ack_tx
					.send(runner.get_active_reports().await)
					.expect("ack channel closed before sending active reports response");
			}
			StreamMessage::RunnerMessage(RunnerMessage::ResumeStoredJob {
				id,
				location_id,
				dyn_job,
				ctx,
				serialized_tasks,
				ack_tx,
			}) => {
				ack_tx
					.send(
						runner
							.new_job(id, location_id, dyn_job, ctx, serialized_tasks)
							.await,
					)
					.expect("ack channel closed before sending resume job response");
			}

			StreamMessage::RunnerMessage(RunnerMessage::Command {
				id,
				command,
				ack_tx,
			}) => {
				ack_tx
					.send(runner.process_command(id, command).await)
					.unwrap_or_else(|_| {
						panic!("ack channel closed before sending {command:?} response")
					});
			}

			StreamMessage::RunnerMessage(RunnerMessage::Shutdown) => {
				// Consuming all pending return status messages
				loop {
					while let Ok((job_id, status)) = job_return_status_rx_to_shutdown.try_recv() {
						if let Err(e) = runner.process_return_status(job_id, status).await {
							error!("Failed to process return status before shutting down: {e:#?}");
						}
					}

					if runner.is_empty() {
						break;
					}

					debug!(
						"Waiting for {} jobs to shutdown before shutting down the job system...",
						runner.total_jobs()
					);
				}

				// Now the runner can shutdown
				if let Err(e) = runner.save_jobs(store_jobs_file).await {
					error!("Failed to save jobs before shutting down: {e:#?}");
				}

				return;
			}

			StreamMessage::RunnerMessage(RunnerMessage::CheckIfJobAreRunning {
				job_names,
				location_id,
				ack_tx,
			}) => {
				ack_tx
					.send(runner.check_if_job_are_running(job_names, location_id))
					.expect("ack channel closed before sending resume job response");
			}

			// Memory cleanup tick
			StreamMessage::CleanMemoryTick => {
				runner.clean_memory();
			}
		}
	}
}
