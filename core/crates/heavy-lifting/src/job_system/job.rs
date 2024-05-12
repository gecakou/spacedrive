use crate::{Error, NonCriticalError, UpdateEvent};

use sd_core_sync::Manager as SyncManager;

use sd_prisma::prisma::PrismaClient;
use sd_task_system::{
	BaseTaskDispatcher, Task, TaskDispatcher, TaskHandle, TaskRemoteController, TaskSystemError,
};

use std::{
	collections::{hash_map::DefaultHasher, VecDeque},
	hash::{Hash, Hasher},
	marker::PhantomData,
	ops::{Deref, DerefMut},
	path::Path,
	pin::pin,
	sync::Arc,
};

use async_channel as chan;
use chrono::{DateTime, Utc};
use futures::{stream, Future, StreamExt};
use futures_concurrency::{
	future::{Join, TryJoin},
	stream::Merge,
};
use serde::{Deserialize, Serialize};
use specta::Type;
use strum::{Display, EnumString};
use tokio::{
	spawn,
	sync::{watch, Mutex},
};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use super::{
	report::{
		Report, ReportBuilder, ReportInputMetadata, ReportMetadata, ReportOutputMetadata, Status,
	},
	Command, JobId, JobSystemError, SerializableJob, SerializedTasks,
};

#[derive(
	Debug, Serialize, Deserialize, EnumString, Display, Clone, Copy, Type, Hash, PartialEq, Eq,
)]
#[strum(use_phf, serialize_all = "snake_case")]
pub enum JobName {
	Indexer,
	FileIdentifier,
	MediaProcessor,
	// TODO: Add more job names as needed
}

#[derive(Debug)]
pub enum ReturnStatus {
	Completed(JobReturn),
	Shutdown(Result<Option<Vec<u8>>, rmp_serde::encode::Error>),
	Canceled,
}

pub enum ProgressUpdate {
	TaskCount(u64),
	CompletedTaskCount(u64),
	Message(String),
	Phase(String),
}

impl ProgressUpdate {
	pub fn message(message: impl Into<String>) -> Self {
		Self::Message(message.into())
	}

	pub fn phase(phase: impl Into<String>) -> Self {
		Self::Phase(phase.into())
	}
}

pub trait OuterContext: Send + Sync + Clone + 'static {
	fn id(&self) -> Uuid;
	fn db(&self) -> &Arc<PrismaClient>;
	fn sync(&self) -> &Arc<SyncManager>;
	fn invalidate_query(&self, query: &'static str);
	fn query_invalidator(&self) -> impl Fn(&'static str) + Send + Sync;
	fn report_update(&self, update: UpdateEvent);
	fn get_data_directory(&self) -> &Path;
}

pub trait JobContext<OuterCtx: OuterContext>: OuterContext {
	fn new(report: Report, ctx: OuterCtx) -> Self;
	fn progress(&self, updates: Vec<ProgressUpdate>) -> impl Future<Output = ()> + Send;
	fn progress_msg(&self, msg: impl Into<String>) -> impl Future<Output = ()> + Send {
		let msg = msg.into();
		async move {
			self.progress(vec![ProgressUpdate::Message(msg)]).await;
		}
	}
	fn report(&self) -> impl Future<Output = impl Deref<Target = Report> + Send> + Send;
	fn report_mut(&self) -> impl Future<Output = impl DerefMut<Target = Report> + Send> + Send;
	fn get_outer_ctx(&self) -> OuterCtx;
}

pub trait Job: Send + Sync + Hash + 'static {
	const NAME: JobName;

	#[allow(unused_variables)]
	fn resume_tasks<OuterCtx: OuterContext>(
		&mut self,
		dispatcher: &JobTaskDispatcher,
		ctx: &impl JobContext<OuterCtx>,
		serialized_tasks: SerializedTasks,
	) -> impl Future<Output = Result<(), Error>> + Send {
		async move { Ok(()) }
	}

	fn run<OuterCtx: OuterContext>(
		self,
		dispatcher: JobTaskDispatcher,
		ctx: impl JobContext<OuterCtx>,
	) -> impl Future<Output = Result<ReturnStatus, Error>> + Send;
}

pub trait IntoJob<J, OuterCtx, JobCtx>
where
	J: Job + SerializableJob<OuterCtx>,
	OuterCtx: OuterContext,
	JobCtx: JobContext<OuterCtx>,
{
	fn into_job(self) -> Box<dyn DynJob<OuterCtx, JobCtx>>;
}

impl<J, OuterCtx, JobCtx> IntoJob<J, OuterCtx, JobCtx> for J
where
	J: Job + SerializableJob<OuterCtx>,
	OuterCtx: OuterContext,
	JobCtx: JobContext<OuterCtx>,
{
	fn into_job(self) -> Box<dyn DynJob<OuterCtx, JobCtx>> {
		let id = JobId::new_v4();

		Box::new(JobHolder {
			id,
			job: self,
			report: ReportBuilder::new(id, J::NAME).build(),
			next_jobs: VecDeque::new(),
			_ctx: PhantomData,
		})
	}
}

impl<J, OuterCtx, JobCtx> IntoJob<J, OuterCtx, JobCtx> for JobEnqueuer<J, OuterCtx, JobCtx>
where
	J: Job + SerializableJob<OuterCtx>,
	OuterCtx: OuterContext,
	JobCtx: JobContext<OuterCtx>,
{
	fn into_job(self) -> Box<dyn DynJob<OuterCtx, JobCtx>> {
		self.build()
	}
}

#[derive(Debug)]
pub struct JobReturn {
	data: JobOutputData,
	metadata: Option<ReportOutputMetadata>,
	non_critical_errors: Vec<NonCriticalError>,
}

impl JobReturn {
	#[must_use]
	pub fn builder() -> JobReturnBuilder {
		JobReturnBuilder {
			job_return: Self::default(),
		}
	}
}

impl Default for JobReturn {
	fn default() -> Self {
		Self {
			data: JobOutputData::Empty,
			metadata: None,
			non_critical_errors: vec![],
		}
	}
}

#[derive(Debug, Default)]
pub struct JobReturnBuilder {
	job_return: JobReturn,
}

impl JobReturnBuilder {
	#[must_use]
	pub const fn with_data(mut self, data: JobOutputData) -> Self {
		self.job_return.data = data;
		self
	}

	#[must_use]
	pub fn with_metadata(mut self, metadata: impl Into<ReportOutputMetadata>) -> Self {
		self.job_return.metadata = Some(metadata.into());
		self
	}

	#[must_use]
	pub fn with_non_critical_errors(mut self, errors: Vec<NonCriticalError>) -> Self {
		if self.job_return.non_critical_errors.is_empty() {
			self.job_return.non_critical_errors = errors;
		} else {
			self.job_return.non_critical_errors.extend(errors);
		}
		self
	}

	#[must_use]
	pub fn build(self) -> JobReturn {
		self.job_return
	}
}

#[derive(Serialize, Type)]
pub struct JobOutput {
	id: JobId,
	status: Status,
	job_name: JobName,
	data: JobOutputData,
	metadata: Vec<ReportMetadata>,
	non_critical_errors: Vec<NonCriticalError>,
}

impl JobOutput {
	pub fn prepare_output_and_report(
		JobReturn {
			data,
			metadata,
			non_critical_errors,
		}: JobReturn,
		report: &mut Report,
	) -> Self {
		if non_critical_errors.is_empty() {
			report.status = Status::Completed;
			debug!("Job<id='{}', name='{}'> completed", report.id, report.name);
		} else {
			report.status = Status::CompletedWithErrors;
			report.non_critical_errors = non_critical_errors
				.iter()
				.map(ToString::to_string)
				.collect();

			warn!(
				"Job<id='{}', name='{}'> completed with errors: {non_critical_errors:#?}",
				report.id, report.name
			);
		}

		if let Some(metadata) = metadata {
			report.metadata.push(ReportMetadata::Output(metadata));
		}

		report.completed_at = Some(Utc::now());

		Self {
			id: report.id,
			status: report.status,
			job_name: report.name,
			data,
			metadata: report.metadata.clone(),
			non_critical_errors,
		}
	}
}

#[derive(Debug, Serialize, Type)]
pub enum JobOutputData {
	Empty,
	// TODO: Add more types
}

pub struct JobEnqueuer<J, OuterCtx, JobCtx>
where
	J: Job + SerializableJob<OuterCtx>,
	OuterCtx: OuterContext,
	JobCtx: JobContext<OuterCtx>,
{
	id: JobId,
	job: J,
	report_builder: ReportBuilder,
	next_jobs: VecDeque<Box<dyn DynJob<OuterCtx, JobCtx>>>,
	_ctx: PhantomData<OuterCtx>,
}

impl<J, OuterCtx, JobCtx> JobEnqueuer<J, OuterCtx, JobCtx>
where
	J: Job + SerializableJob<OuterCtx>,
	OuterCtx: OuterContext,
	JobCtx: JobContext<OuterCtx>,
{
	fn build(self) -> Box<dyn DynJob<OuterCtx, JobCtx>> {
		Box::new(JobHolder {
			id: self.id,
			job: self.job,
			report: self.report_builder.build(),
			next_jobs: self.next_jobs,
			_ctx: self._ctx,
		})
	}

	pub fn new(job: J) -> Self {
		let id = JobId::new_v4();
		Self {
			id,
			job,
			report_builder: ReportBuilder::new(id, J::NAME),
			next_jobs: VecDeque::new(),
			_ctx: PhantomData,
		}
	}

	#[must_use]
	pub fn with_action(mut self, action: impl Into<String>) -> Self {
		self.report_builder = self.report_builder.with_action(action);
		self
	}

	#[must_use]
	pub fn with_parent_id(mut self, parent_id: JobId) -> Self {
		self.report_builder = self.report_builder.with_parent_id(parent_id);
		self
	}

	#[must_use]
	pub fn with_metadata(mut self, metadata: ReportInputMetadata) -> Self {
		self.report_builder = self.report_builder.with_metadata(metadata);
		self
	}

	#[must_use]
	pub fn enqueue_next(mut self, next: impl Job + SerializableJob<OuterCtx>) -> Self {
		let next_job_order = self.next_jobs.len() + 1;

		let mut child_job_builder = JobEnqueuer::new(next).with_parent_id(self.id);

		if let Some(parent_action) = &self.report_builder.action {
			child_job_builder =
				child_job_builder.with_action(format!("{parent_action}-{next_job_order}"));
		}

		self.next_jobs.push_back(child_job_builder.build());

		self
	}
}

pub struct JobHolder<J, OuterCtx, JobCtx>
where
	J: Job + SerializableJob<OuterCtx>,
	OuterCtx: OuterContext,
	JobCtx: JobContext<OuterCtx>,
{
	pub(super) id: JobId,
	pub(super) job: J,
	pub(super) report: Report,
	pub(super) next_jobs: VecDeque<Box<dyn DynJob<OuterCtx, JobCtx>>>,
	pub(super) _ctx: PhantomData<OuterCtx>,
}

pub struct JobHandle<OuterCtx: OuterContext, JobCtx: JobContext<OuterCtx>> {
	pub(crate) id: JobId,
	pub(crate) next_jobs: VecDeque<Box<dyn DynJob<OuterCtx, JobCtx>>>,
	pub(crate) ctx: JobCtx,
	pub(crate) commands_tx: chan::Sender<Command>,
}

impl<OuterCtx: OuterContext, JobCtx: JobContext<OuterCtx>> JobHandle<OuterCtx, JobCtx> {
	pub async fn send_command(&mut self, command: Command) -> Result<(), JobSystemError> {
		trace!(
			"Handle sending command {command:?} to <job_id='{}'>",
			self.id
		);
		if self.commands_tx.send(command).await.is_err() {
			warn!("Tried to send a {command:?} to a job that was already completed");

			Ok(())
		} else {
			self.command_children(command).await
		}
	}

	pub async fn command_children(&mut self, command: Command) -> Result<(), JobSystemError> {
		let (new_status, completed_at) = match command {
			Command::Pause => (Status::Paused, None),
			Command::Resume => return Ok(()),
			Command::Cancel => (Status::Canceled, Some(Utc::now())),
		};

		self.next_jobs
			.iter_mut()
			.map(|dyn_job| dyn_job.report_mut())
			.map(|next_job_report| async {
				next_job_report.status = new_status;
				next_job_report.completed_at = completed_at;

				trace!(
					"Parent job sent command {command:?} to <job_id='{}'>",
					next_job_report.id
				);

				next_job_report.update(self.ctx.db()).await
			})
			.collect::<Vec<_>>()
			.try_join()
			.await
			.map(|_| ())
			.map_err(Into::into)
	}

	pub async fn register_start(
		&mut self,
		start_time: DateTime<Utc>,
	) -> Result<(), JobSystemError> {
		trace!("Handle registering start of <job_id='{}'>", self.id);

		let Self { next_jobs, ctx, .. } = self;

		let mut report = ctx.report_mut().await;

		report.status = Status::Running;
		if report.started_at.is_none() {
			report.started_at = Some(start_time);
		}

		let db = ctx.db();

		// If the report doesn't have a created_at date, it's a new report
		if report.created_at.is_none() {
			report.create(db).await?;
		} else {
			// Otherwise it can be a job being resumed or a children job that was already been created
			report.update(db).await?;
		}

		// Registering children jobs
		next_jobs
			.iter_mut()
			.map(|dyn_job| dyn_job.report_mut())
			.map(|next_job_report| async {
				trace!(
					"Parent job registering children <job_id='{}'>",
					next_job_report.id
				);
				if next_job_report.created_at.is_none() {
					next_job_report.create(db).await
				} else {
					Ok(())
				}
			})
			.collect::<Vec<_>>()
			.try_join()
			.await
			.map(|_| ())
			.map_err(Into::into)
	}

	pub async fn complete_job(
		&mut self,
		job_return: JobReturn,
	) -> Result<JobOutput, JobSystemError> {
		let Self { ctx, .. } = self;

		let mut report = ctx.report_mut().await;

		trace!("Handle completing <job_id='{}'>", self.id);

		let output = JobOutput::prepare_output_and_report(job_return, &mut report);

		report.update(ctx.db()).await?;

		trace!("Handle completed <job_id='{}'>", self.id);

		Ok(output)
	}

	pub async fn failed_job(&mut self, e: &Error) -> Result<(), JobSystemError> {
		trace!("Handle registering failed job <job_id='{}'>", self.id);

		let db = self.ctx.db();
		{
			let mut report = self.ctx.report_mut().await;

			error!(
				"Job<id='{}', name='{}'> failed with a critical error: {e:#?};",
				report.id, report.name
			);

			report.status = Status::Failed;
			report.critical_error = Some(e.to_string());
			report.completed_at = Some(Utc::now());

			report.update(db).await?;
		}

		trace!(
			"Handle sending cancel command to children due to failure: <job_id='{}'>",
			self.id
		);

		self.command_children(Command::Cancel).await
	}

	pub async fn shutdown_pause_job(&mut self) -> Result<(), JobSystemError> {
		trace!("Handle pausing job on shutdown: <job_id='{}'>", self.id);

		let db = self.ctx.db();

		{
			let mut report = self.ctx.report_mut().await;

			info!(
				"Job<id='{}', name='{}'> paused due to system shutdown, we will pause all children jobs",
				report.id, report.name
			);

			report.status = Status::Paused;

			report.update(db).await?;
		}

		self.command_children(Command::Pause).await
	}

	pub async fn cancel_job(&mut self) -> Result<(), JobSystemError> {
		trace!("Handle canceling job: <job_id='{}'>", self.id);
		let db = self.ctx.db();

		{
			let mut report = self.ctx.report_mut().await;

			info!(
				"Job<id='{}', name='{}'> canceled, we will cancel all children jobs",
				report.id, report.name
			);

			report.status = Status::Canceled;
			report.completed_at = Some(Utc::now());

			report.update(db).await?;
		}

		trace!(
			"Handle sending cancel command to children: <job_id='{}'>",
			self.id
		);

		self.command_children(Command::Cancel).await
	}
}

#[async_trait::async_trait]
pub trait DynJob<OuterCtx: OuterContext, JobCtx: JobContext<OuterCtx>>:
	Send + Sync + 'static
{
	fn id(&self) -> JobId;

	fn job_name(&self) -> JobName;

	fn hash(&self) -> u64;

	fn report_mut(&mut self) -> &mut Report;

	fn set_next_jobs(&mut self, next_jobs: VecDeque<Box<dyn DynJob<OuterCtx, JobCtx>>>);

	fn next_jobs(&self) -> &VecDeque<Box<dyn DynJob<OuterCtx, JobCtx>>>;

	async fn serialize(self: Box<Self>) -> Result<Option<Vec<u8>>, rmp_serde::encode::Error>;

	fn dispatch(
		self: Box<Self>,
		base_dispatcher: BaseTaskDispatcher<Error>,
		ctx: OuterCtx,
		done_tx: chan::Sender<(JobId, Result<ReturnStatus, Error>)>,
	) -> JobHandle<OuterCtx, JobCtx>;

	fn resume(
		self: Box<Self>,
		base_dispatcher: BaseTaskDispatcher<Error>,
		ctx: OuterCtx,
		serialized_tasks: Option<SerializedTasks>,
		done_tx: chan::Sender<(JobId, Result<ReturnStatus, Error>)>,
	) -> JobHandle<OuterCtx, JobCtx>;
}

#[async_trait::async_trait]
impl<J, OuterCtx, JobCtx> DynJob<OuterCtx, JobCtx> for JobHolder<J, OuterCtx, JobCtx>
where
	J: Job + SerializableJob<OuterCtx>,
	OuterCtx: OuterContext,
	JobCtx: JobContext<OuterCtx>,
{
	fn id(&self) -> JobId {
		self.id
	}

	fn job_name(&self) -> JobName {
		J::NAME
	}

	fn hash(&self) -> u64 {
		let mut hasher = DefaultHasher::new();
		J::NAME.hash(&mut hasher);
		self.job.hash(&mut hasher);
		hasher.finish()
	}

	fn report_mut(&mut self) -> &mut Report {
		&mut self.report
	}

	fn set_next_jobs(&mut self, next_jobs: VecDeque<Box<dyn DynJob<OuterCtx, JobCtx>>>) {
		self.next_jobs = next_jobs;
	}

	fn next_jobs(&self) -> &VecDeque<Box<dyn DynJob<OuterCtx, JobCtx>>> {
		&self.next_jobs
	}

	async fn serialize(self: Box<Self>) -> Result<Option<Vec<u8>>, rmp_serde::encode::Error> {
		self.job.serialize().await
	}

	fn dispatch(
		self: Box<Self>,
		base_dispatcher: BaseTaskDispatcher<Error>,
		ctx: OuterCtx,
		done_tx: chan::Sender<(JobId, Result<ReturnStatus, Error>)>,
	) -> JobHandle<OuterCtx, JobCtx> {
		let (commands_tx, commands_rx) = chan::bounded(8);

		let ctx = JobCtx::new(self.report, ctx);

		trace!("Dispatching job <job_id='{}'>", self.id);

		spawn(to_spawn_job::<OuterCtx, _>(
			self.id,
			self.job,
			ctx.clone(),
			None,
			base_dispatcher,
			commands_rx,
			done_tx,
		));

		JobHandle {
			id: self.id,
			next_jobs: self.next_jobs,
			ctx,
			commands_tx,
		}
	}

	fn resume(
		self: Box<Self>,
		base_dispatcher: BaseTaskDispatcher<Error>,
		ctx: OuterCtx,
		serialized_tasks: Option<SerializedTasks>,
		done_tx: chan::Sender<(JobId, Result<ReturnStatus, Error>)>,
	) -> JobHandle<OuterCtx, JobCtx> {
		let (commands_tx, commands_rx) = chan::bounded(8);

		let ctx = JobCtx::new(self.report, ctx);

		trace!("Resuming job <job_id='{}'>", self.id);

		spawn(to_spawn_job::<OuterCtx, _>(
			self.id,
			self.job,
			ctx.clone(),
			serialized_tasks,
			base_dispatcher,
			commands_rx,
			done_tx,
		));

		JobHandle {
			id: self.id,
			next_jobs: self.next_jobs,
			ctx,
			commands_tx,
		}
	}
}

async fn to_spawn_job<OuterCtx: OuterContext, JobCtx: JobContext<OuterCtx>>(
	id: JobId,
	mut job: impl Job,
	ctx: JobCtx,
	existing_tasks: Option<SerializedTasks>,
	base_dispatcher: BaseTaskDispatcher<Error>,
	commands_rx: chan::Receiver<Command>,
	done_tx: chan::Sender<(JobId, Result<ReturnStatus, Error>)>,
) {
	enum StreamMessage {
		Commands(Command),
		NewRemoteController(TaskRemoteController),
		Done(Result<ReturnStatus, Error>),
	}

	let mut remote_controllers = vec![];

	let (running_state_tx, running_state_rx) = watch::channel(JobRunningState::Running);

	let (dispatcher, remote_controllers_rx) =
		JobTaskDispatcher::new(base_dispatcher, running_state_rx);

	if let Some(existing_tasks) = existing_tasks {
		if let Err(e) = job.resume_tasks(&dispatcher, &ctx, existing_tasks).await {
			done_tx
				.send((id, Err(e)))
				.await
				.expect("jobs done tx closed on error at resume_tasks");

			return;
		}
	}

	let mut msgs_stream = pin!((
		commands_rx.map(StreamMessage::Commands),
		remote_controllers_rx.map(StreamMessage::NewRemoteController),
		stream::once(job.run::<OuterCtx>(dispatcher, ctx)).map(StreamMessage::Done),
	)
		.merge());

	while let Some(msg) = msgs_stream.next().await {
		match msg {
			StreamMessage::NewRemoteController(remote_controller) => {
				remote_controllers.push(remote_controller);
			}
			StreamMessage::Commands(command) => {
				remote_controllers.retain(|controller| !controller.is_done());

				match command {
					Command::Pause => {
						trace!("Pausing job <job_id='{}'>", id);
						running_state_tx.send_modify(|state| *state = JobRunningState::Paused);
						remote_controllers
							.iter()
							.map(TaskRemoteController::pause)
							.collect::<Vec<_>>()
							.join()
							.await
							.into_iter()
							.for_each(|res| {
								if let Err(e) = res {
									assert!(matches!(e, TaskSystemError::TaskNotFound(_)));

									warn!("Tried to pause a task that was already completed");
								}
							});
					}
					Command::Resume => {
						trace!("Resuming job <job_id='{}'>", id);
						running_state_tx.send_modify(|state| *state = JobRunningState::Running);

						remote_controllers
							.iter()
							.map(TaskRemoteController::resume)
							.collect::<Vec<_>>()
							.join()
							.await
							.into_iter()
							.for_each(|res| {
								if let Err(e) = res {
									assert!(matches!(e, TaskSystemError::TaskNotFound(_)));

									warn!("Tried to pause a task that was already completed");
								}
							});
					}
					Command::Cancel => {
						trace!("Canceling job <job_id='{}'>", id);
						remote_controllers
							.iter()
							.map(TaskRemoteController::cancel)
							.collect::<Vec<_>>()
							.join()
							.await;

						return done_tx
							.send((id, Ok(ReturnStatus::Canceled)))
							.await
							.expect("jobs done tx closed");
					}
				}
			}

			StreamMessage::Done(res) => {
				trace!("Job <job_id='{}'> done", id);
				#[cfg(debug_assertions)]
				{
					// Just a sanity check to make sure we don't have any pending tasks left
					remote_controllers.retain(|controller| !controller.is_done());
					assert!(remote_controllers.is_empty());
					// Using #[cfg(debug_assertions)] to don't pay this retain cost in release builds
				}

				return done_tx.send((id, res)).await.expect("jobs done tx closed");
			}
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JobRunningState {
	Running,
	Paused,
}

impl Default for JobRunningState {
	fn default() -> Self {
		Self::Running
	}
}

#[derive(Debug, Clone)]
pub struct JobTaskDispatcher {
	dispatcher: BaseTaskDispatcher<Error>,
	remote_controllers_tx: chan::Sender<TaskRemoteController>,
	running_state: Arc<Mutex<watch::Receiver<JobRunningState>>>,
}

impl TaskDispatcher<Error> for JobTaskDispatcher {
	async fn dispatch_boxed(&self, boxed_task: Box<dyn Task<Error>>) -> TaskHandle<Error> {
		self.wait_for_dispatch_approval().await;

		let handle = self.dispatcher.dispatch_boxed(boxed_task).await;

		self.remote_controllers_tx
			.send(handle.remote_controller())
			.await
			.expect("remote controllers tx closed");

		handle
	}

	async fn dispatch_many_boxed(
		&self,
		boxed_tasks: impl IntoIterator<Item = Box<dyn Task<Error>>> + Send,
	) -> Vec<TaskHandle<Error>> {
		self.wait_for_dispatch_approval().await;

		let handles = self.dispatcher.dispatch_many_boxed(boxed_tasks).await;

		handles
			.iter()
			.map(|handle| self.remote_controllers_tx.send(handle.remote_controller()))
			.collect::<Vec<_>>()
			.try_join()
			.await
			.expect("remote controllers tx closed");

		handles
	}
}

impl JobTaskDispatcher {
	fn new(
		dispatcher: BaseTaskDispatcher<Error>,
		running_state_rx: watch::Receiver<JobRunningState>,
	) -> (Self, chan::Receiver<TaskRemoteController>) {
		let (remote_controllers_tx, remote_controllers_rx) = chan::unbounded();

		(
			Self {
				dispatcher,
				remote_controllers_tx,
				running_state: Arc::new(Mutex::new(running_state_rx)),
			},
			remote_controllers_rx,
		)
	}

	async fn wait_for_dispatch_approval(&self) {
		self.running_state
			.lock()
			.await
			.wait_for(|state| *state == JobRunningState::Running)
			.await
			.expect("job running state watch channel unexpectedly closed");
	}
}
