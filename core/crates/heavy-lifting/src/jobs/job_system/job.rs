use crate::{jobs::JobId, Error, NonCriticalJobError};

use sd_prisma::prisma::PrismaClient;
use sd_task_system::{IntoTask, Task, TaskHandle, TaskRemoteController, TaskSystemError};

use std::{
	collections::VecDeque,
	hash::{DefaultHasher, Hash, Hasher},
	pin::pin,
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
use tokio::spawn;
use tracing::{debug, error, info, warn};

use super::{
	report::{
		Report, ReportBuilder, ReportInputMetadata, ReportMetadata, ReportOutputMetadata, Status,
	},
	Command, JobSystemError, SerializableJob,
};

#[derive(Debug, Serialize, Deserialize, EnumString, Display, Clone, Copy, Type, Hash)]
#[strum(use_phf, serialize_all = "snake_case")]
pub enum JobName {
	Indexer,
	// TODO: Add more job names as needed
}

pub enum ReturnStatus {
	Completed(JobReturn),
	Failed(Error),
	Shutdown(Option<Result<Vec<u8>, rmp_serde::encode::Error>>),
	Canceled,
}

pub trait Job: Send + Sync + Hash + 'static {
	const NAME: JobName;

	fn resume(&mut self, dispatched_tasks: Vec<TaskHandle<Error>>);

	fn run(self, dispatcher: TaskDispatcher) -> impl Future<Output = ReturnStatus> + Send;
}

pub trait IntoJob<J: Job + SerializableJob> {
	fn into_job(self) -> Box<dyn DynJob>;
}

impl<J: Job + SerializableJob> IntoJob<J> for J {
	fn into_job(self) -> Box<dyn DynJob> {
		let id = JobId::new_v4();

		Box::new(JobHolder {
			id,
			job: self,
			report: ReportBuilder::new(id, J::NAME).build(),
			next_jobs: VecDeque::new(),
		})
	}
}

impl<J: Job + SerializableJob> IntoJob<J> for JobBuilder<J> {
	fn into_job(self) -> Box<dyn DynJob> {
		self.build()
	}
}

pub struct JobReturn {
	data: JobOutputData,
	metadata: Option<ReportOutputMetadata>,
	non_critical_errors: Vec<NonCriticalJobError>,
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

#[derive(Serialize, Type)]
pub struct JobOutput {
	id: JobId,
	status: Status,
	job_name: JobName,
	data: JobOutputData,
	metadata: Vec<ReportMetadata>,
	non_critical_errors: Vec<NonCriticalJobError>,
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

#[derive(Serialize, Type)]
pub enum JobOutputData {
	Empty,
	// TODO: Add more types
}

pub struct JobBuilder<J: Job + SerializableJob> {
	id: JobId,
	job: J,
	report_builder: ReportBuilder,
	next_jobs: VecDeque<Box<dyn DynJob>>,
}

impl<J: Job + SerializableJob> JobBuilder<J> {
	pub fn build(self) -> Box<JobHolder<J>> {
		Box::new(JobHolder::<J> {
			id: self.id,
			job: self.job,
			report: self.report_builder.build(),
			next_jobs: VecDeque::new(),
		})
	}

	pub fn new(job: J) -> Self {
		let id = JobId::new_v4();
		Self {
			id,
			job,
			report_builder: ReportBuilder::new(id, J::NAME),
			next_jobs: VecDeque::new(),
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
	pub fn enqueue_next(mut self, next: impl Job + SerializableJob) -> Self {
		let next_job_order = self.next_jobs.len() + 1;

		let mut child_job_builder = JobBuilder::new(next).with_parent_id(self.id);

		if let Some(parent_action) = &self.report_builder.action {
			child_job_builder =
				child_job_builder.with_action(format!("{parent_action}-{next_job_order}"));
		}

		self.next_jobs.push_back(child_job_builder.build());

		self
	}
}

pub struct JobHolder<J: Job + SerializableJob> {
	pub(super) id: JobId,
	pub(super) job: J,
	pub(super) report: Report,
	pub(super) next_jobs: VecDeque<Box<dyn DynJob>>,
}

pub struct JobHandle {
	pub(crate) next_jobs: VecDeque<Box<dyn DynJob>>,
	pub(crate) report: Report,
	pub(crate) commands_tx: chan::Sender<Command>,
}

impl JobHandle {
	pub async fn send_command(
		&mut self,
		command: Command,
		db: &PrismaClient,
	) -> Result<(), JobSystemError> {
		if self.commands_tx.send(command).await.is_err() {
			warn!("Tried to send a {command:?} to a job that was already completed");

			Ok(())
		} else {
			self.command_children(command, db).await
		}
	}

	pub async fn command_children(
		&mut self,
		command: Command,
		db: &PrismaClient,
	) -> Result<(), JobSystemError> {
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

				next_job_report.update(db).await
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
		db: &PrismaClient,
	) -> Result<(), JobSystemError> {
		let Self {
			next_jobs, report, ..
		} = self;

		report.status = Status::Running;
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

		// Registering children jobs
		next_jobs
			.iter_mut()
			.map(|dyn_job| dyn_job.report_mut())
			.map(|next_job_report| async {
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
		db: &PrismaClient,
	) -> Result<JobOutput, JobSystemError> {
		let Self { report, .. } = self;

		let output = JobOutput::prepare_output_and_report(job_return, report);

		report.update(db).await?;

		Ok(output)
	}

	pub async fn failed_job(&mut self, e: &Error, db: &PrismaClient) -> Result<(), JobSystemError> {
		let Self { report, .. } = self;
		error!(
			"Job<id='{}', name='{}'> failed with a critical error: {e:#?};",
			report.id, report.name
		);

		report.status = Status::Failed;
		report.critical_error = Some(e.to_string());
		report.completed_at = Some(Utc::now());

		report.update(db).await?;

		self.command_children(Command::Cancel, db).await
	}

	pub async fn shutdown_pause_job(&mut self, db: &PrismaClient) -> Result<(), JobSystemError> {
		let Self { report, .. } = self;
		info!(
			"Job<id='{}', name='{}'> paused due to system shutdown, we will pause all children jobs",
			report.id, report.name
		);

		report.status = Status::Paused;

		report.update(db).await?;

		self.command_children(Command::Pause, db).await
	}

	pub async fn cancel_job(&mut self, db: &PrismaClient) -> Result<(), JobSystemError> {
		let Self { report, .. } = self;
		info!(
			"Job<id='{}', name='{}'> canceled, we will cancel all children jobs",
			report.id, report.name
		);

		report.status = Status::Canceled;
		report.completed_at = Some(Utc::now());

		report.update(db).await?;

		self.command_children(Command::Cancel, db).await
	}
}

pub trait DynJob: Send + Sync + 'static {
	fn id(&self) -> JobId;

	fn job_name(&self) -> JobName;

	fn hash(&self) -> u64;

	fn report_mut(&mut self) -> &mut Report;

	fn set_next_jobs(&mut self, next_jobs: VecDeque<Box<dyn DynJob>>);

	fn next_jobs(&self) -> &VecDeque<Box<dyn DynJob>>;

	fn serialize(&self) -> Option<Result<Vec<u8>, rmp_serde::encode::Error>>;

	fn dispatch(
		self: Box<Self>,
		dispatcher: sd_task_system::TaskDispatcher<Error>,
		done_tx: chan::Sender<(JobId, ReturnStatus)>,
	) -> JobHandle;

	fn resume(
		self: Box<Self>,
		dispatcher: sd_task_system::TaskDispatcher<Error>,
		existing_tasks: Vec<Box<dyn Task<Error>>>,
		done_tx: chan::Sender<(JobId, ReturnStatus)>,
	) -> JobHandle;
}

impl<J: Job + SerializableJob> DynJob for JobHolder<J> {
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

	fn set_next_jobs(&mut self, next_jobs: VecDeque<Box<dyn DynJob>>) {
		self.next_jobs = next_jobs;
	}

	fn next_jobs(&self) -> &VecDeque<Box<dyn DynJob>> {
		&self.next_jobs
	}

	fn serialize(&self) -> Option<Result<Vec<u8>, rmp_serde::encode::Error>> {
		self.job.serialize()
	}

	fn dispatch(
		self: Box<Self>,
		dispatcher: sd_task_system::TaskDispatcher<Error>,
		done_tx: chan::Sender<(JobId, ReturnStatus)>,
	) -> JobHandle {
		let (commands_tx, commands_rx) = chan::bounded(8);

		spawn(to_spawn_job(
			self.id,
			self.job,
			None,
			dispatcher,
			commands_rx,
			done_tx,
		));

		JobHandle {
			next_jobs: self.next_jobs,
			report: self.report,
			commands_tx,
		}
	}

	fn resume(
		self: Box<Self>,
		dispatcher: sd_task_system::TaskDispatcher<Error>,
		existing_tasks: Vec<Box<dyn Task<Error>>>,
		done_tx: chan::Sender<(JobId, ReturnStatus)>,
	) -> JobHandle {
		let (commands_tx, commands_rx) = chan::bounded(8);

		spawn(to_spawn_job(
			self.id,
			self.job,
			Some(existing_tasks),
			dispatcher,
			commands_rx,
			done_tx,
		));

		JobHandle {
			next_jobs: self.next_jobs,
			report: self.report,
			commands_tx,
		}
	}
}

async fn to_spawn_job(
	id: JobId,
	mut job: impl Job,
	existing_tasks: Option<Vec<Box<dyn Task<Error>>>>,
	dispatcher: sd_task_system::TaskDispatcher<Error>,
	commands_rx: chan::Receiver<Command>,
	done_tx: chan::Sender<(JobId, ReturnStatus)>,
) {
	enum StreamMessage {
		Commands(Command),
		NewRemoteController(TaskRemoteController),
		Done(ReturnStatus),
	}

	let mut remote_controllers = vec![];

	let (dispatcher, remote_controllers_rx) = TaskDispatcher::new(dispatcher);

	if let Some(existing_tasks) = existing_tasks {
		job.resume(
			existing_tasks
				.into_iter()
				.map(|task| dispatcher.dispatch_boxed(task))
				.collect::<Vec<_>>()
				.join()
				.await,
		);
	}

	let mut msgs_stream = pin!((
		commands_rx.map(StreamMessage::Commands),
		remote_controllers_rx.map(StreamMessage::NewRemoteController),
		stream::once(job.run(dispatcher)).map(StreamMessage::Done),
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
						remote_controllers
							.iter()
							.map(TaskRemoteController::cancel)
							.collect::<Vec<_>>()
							.join()
							.await;

						return done_tx
							.send((id, ReturnStatus::Canceled))
							.await
							.expect("jobs done tx closed");
					}
				}
			}

			StreamMessage::Done(res) => {
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

pub struct TaskDispatcher {
	dispatcher: sd_task_system::TaskDispatcher<Error>,
	remote_controllers_tx: chan::Sender<TaskRemoteController>,
}

impl TaskDispatcher {
	fn new(
		dispatcher: sd_task_system::TaskDispatcher<Error>,
	) -> (Self, chan::Receiver<TaskRemoteController>) {
		let (remote_controllers_tx, remote_controllers_rx) = chan::unbounded();

		(
			Self {
				dispatcher,
				remote_controllers_tx,
			},
			remote_controllers_rx,
		)
	}

	pub async fn dispatch(&self, into_task: impl IntoTask<Error>) -> TaskHandle<Error> {
		self.dispatch_boxed(into_task.into_task()).await
	}

	pub async fn dispatch_boxed(&self, task: Box<dyn Task<Error>>) -> TaskHandle<Error> {
		let handle = self.dispatcher.dispatch_boxed(task).await;

		self.remote_controllers_tx
			.send(handle.remote_controller())
			.await
			.expect("remote controllers tx closed");

		handle
	}

	pub async fn dispatch_many(
		&self,
		into_tasks: Vec<impl IntoTask<Error>>,
	) -> Vec<TaskHandle<Error>> {
		let handles = self.dispatcher.dispatch_many(into_tasks).await;

		for handle in &handles {
			self.remote_controllers_tx
				.send(handle.remote_controller())
				.await
				.expect("remote controllers tx closed");
		}

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