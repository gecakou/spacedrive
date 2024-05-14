use std::{
	collections::{HashMap, VecDeque},
	future::pending,
	pin::pin,
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::Duration,
};

use async_channel as chan;
use futures::{FutureExt, StreamExt};
use futures_concurrency::future::Race;
use tokio::{
	spawn,
	sync::oneshot,
	task::{JoinError, JoinHandle},
	time::{sleep, timeout, Instant},
};
use tracing::{debug, error, trace, warn};

use super::{
	super::{
		error::{RunError, SystemError},
		system::SystemComm,
		task::{
			ExecStatus, InternalTaskExecStatus, Interrupter, Task, TaskId, TaskOutput, TaskStatus,
			TaskWorkState, TaskWorktable,
		},
	},
	RunnerMessage, TaskRunnerOutput, WorkStealer, WorkerId, ONE_SECOND,
};

const TEN_SECONDS: Duration = Duration::from_secs(10);
const ONE_MINUTE: Duration = Duration::from_secs(60);

const TASK_QUEUE_INITIAL_SIZE: usize = 64;
const PRIORITY_TASK_QUEUE_INITIAL_SIZE: usize = 32;
const ABORT_AND_SUSPEND_MAP_INITIAL_SIZE: usize = 8;

pub(super) enum TaskAddStatus {
	Running,
	Enqueued,
}

struct AbortAndSuspendSignalers {
	abort_tx: oneshot::Sender<oneshot::Sender<Result<(), SystemError>>>,
	suspend_tx: oneshot::Sender<()>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PendingTaskKind {
	Normal,
	Priority,
	Suspended,
}

impl PendingTaskKind {
	const fn with_priority(has_priority: bool) -> Self {
		if has_priority {
			Self::Priority
		} else {
			Self::Normal
		}
	}
}

struct RunningTask {
	task_id: TaskId,
	task_kind: PendingTaskKind,
	handle: JoinHandle<()>,
}

fn dispatch_steal_request<E: RunError>(
	worker_id: WorkerId,
	work_stealer: WorkStealer<E>,
	runner_tx: chan::Sender<RunnerMessage<E>>,
) -> JoinHandle<()> {
	spawn(async move {
		runner_tx
			.send(RunnerMessage::StoleTask(
				work_stealer.steal(worker_id).await,
			))
			.await
			.expect("runner channel closed before send stolen task");
	})
}

enum WaitingSuspendedTask {
	Task(TaskId),
	None,
}

impl WaitingSuspendedTask {
	const fn is_waiting(&self) -> bool {
		matches!(self, Self::Task(_))
	}
}

pub(super) struct Runner<E: RunError> {
	worker_id: WorkerId,
	system_comm: SystemComm,
	work_stealer: WorkStealer<E>,
	task_kinds: HashMap<TaskId, PendingTaskKind>,
	tasks: VecDeque<TaskWorkState<E>>,
	paused_tasks: HashMap<TaskId, TaskWorkState<E>>,
	suspended_task: Option<TaskWorkState<E>>,
	priority_tasks: VecDeque<TaskWorkState<E>>,
	last_requested_help: Instant,
	is_idle: bool,
	waiting_suspension: WaitingSuspendedTask,
	abort_and_suspend_map: HashMap<TaskId, AbortAndSuspendSignalers>,
	msgs_tx: chan::Sender<RunnerMessage<E>>,
	current_task_handle: Option<RunningTask>,
	suspend_on_shutdown_rx: chan::Receiver<RunnerMessage<E>>,
	current_steal_task_handle: Option<JoinHandle<()>>,
	last_steal_attempt_at: Instant,
	steal_attempts_count: u32,
}

impl<E: RunError> Runner<E> {
	pub(super) fn new(
		worker_id: WorkerId,
		work_stealer: WorkStealer<E>,
		system_comm: SystemComm,
	) -> (Self, chan::Receiver<RunnerMessage<E>>) {
		let (runner_tx, runner_rx) = chan::bounded(8);

		(
			Self {
				worker_id,
				system_comm,
				work_stealer,
				task_kinds: HashMap::with_capacity(TASK_QUEUE_INITIAL_SIZE),
				tasks: VecDeque::with_capacity(TASK_QUEUE_INITIAL_SIZE),
				paused_tasks: HashMap::new(),
				suspended_task: None,
				priority_tasks: VecDeque::with_capacity(PRIORITY_TASK_QUEUE_INITIAL_SIZE),
				last_requested_help: Instant::now(),
				is_idle: true,
				waiting_suspension: WaitingSuspendedTask::None,
				abort_and_suspend_map: HashMap::with_capacity(ABORT_AND_SUSPEND_MAP_INITIAL_SIZE),
				msgs_tx: runner_tx,
				current_task_handle: None,
				suspend_on_shutdown_rx: runner_rx.clone(),
				current_steal_task_handle: None,
				last_steal_attempt_at: Instant::now(),
				steal_attempts_count: 0,
			},
			runner_rx,
		)
	}

	pub(super) fn total_tasks(&self) -> usize {
		let priority_tasks_count = self.priority_tasks.len();
		let current_task_count = usize::from(self.current_task_handle.is_some());
		let suspended_task_count = usize::from(self.suspended_task.is_some());
		let tasks_count = self.tasks.len();

		trace!(
			"Task count: \
			<worker_id='{}', \
			priority_tasks_count={priority_tasks_count}, \
			current_task_count={current_task_count}, \
			suspended_task_count={suspended_task_count}, \
			tasks_count={tasks_count}>",
			self.worker_id
		);

		priority_tasks_count + current_task_count + suspended_task_count + tasks_count
	}

	pub(super) fn spawn_task_runner(
		&mut self,
		task_id: TaskId,
		task_work_state: TaskWorkState<E>,
	) -> JoinHandle<()> {
		let (abort_tx, abort_rx) = oneshot::channel();
		let (suspend_tx, suspend_rx) = oneshot::channel();

		self.abort_and_suspend_map.insert(
			task_id,
			AbortAndSuspendSignalers {
				abort_tx,
				suspend_tx,
			},
		);

		let handle = spawn(run_single_task(
			self.worker_id,
			task_work_state,
			self.msgs_tx.clone(),
			suspend_rx,
			abort_rx,
		));

		trace!(
			"Task runner spawned: <worker_id='{}', task_id='{task_id}'>",
			self.worker_id
		);

		handle
	}

	pub(super) async fn new_task(&mut self, task_work_state: TaskWorkState<E>) {
		let task_id = task_work_state.task.id();
		let new_kind = PendingTaskKind::with_priority(task_work_state.task.with_priority());

		trace!(
			"Received new task: <worker_id='{}', task_id='{task_id}', kind='{new_kind:#?}'>",
			self.worker_id
		);

		self.task_kinds.insert(task_id, new_kind);

		match self
			.inner_add_task(task_id, new_kind, task_work_state)
			.await
		{
			TaskAddStatus::Running => trace!(
				"Task running: <worker_id='{}', task_id='{task_id}'>",
				self.worker_id
			),
			TaskAddStatus::Enqueued => trace!(
				"Task enqueued: <worker_id='{}', task_id='{task_id}'>",
				self.worker_id
			),
		}
	}

	pub(super) async fn resume_task(&mut self, task_id: TaskId) -> Result<(), SystemError> {
		trace!(
			"Resume task request: <worker_id='{}', task_id='{task_id}'>",
			self.worker_id
		);
		if let Some(task_work_state) = self.paused_tasks.remove(&task_id) {
			task_work_state.worktable.set_unpause();

			match self
				.inner_add_task(
					task_id,
					*self
						.task_kinds
						.get(&task_id)
						.expect("we added the task kind before pausing it"),
					task_work_state,
				)
				.await
			{
				TaskAddStatus::Running => trace!(
					"Resumed task is running: <worker_id='{}', task_id='{task_id}'>",
					self.worker_id
				),
				TaskAddStatus::Enqueued => trace!(
					"Resumed task was enqueued: <worker_id='{}', task_id='{task_id}'>",
					self.worker_id
				),
			}

			Ok(())
		} else {
			trace!(
				"Task not found: <worker_id='{}', task_id='{task_id}'>",
				self.worker_id
			);
			Err(SystemError::TaskNotFound(task_id))
		}
	}

	pub(super) fn pause_not_running_task(&mut self, task_id: TaskId) -> Result<(), SystemError> {
		trace!(
			"Pause not running task request: <worker_id='{}', task_id='{task_id}'>",
			self.worker_id
		);

		if self.paused_tasks.contains_key(&task_id) {
			trace!(
				"Task is already paused: <worker_id='{}', task_id='{task_id}'>",
				self.worker_id
			);
			return Ok(());
		}

		if let Some(current_task) = &self.current_task_handle {
			if current_task.task_id == task_id {
				trace!(
					"Task began to run before we managed to pause it, run function will pause it: \
					<worker_id='{}', task_id='{task_id}'>",
					self.worker_id
				);
				return Ok(()); // The task will pause itself
			}
		}

		if self.pause_suspended_task(task_id) || self.pause_task_from_queues(task_id) {
			return Ok(());
		}

		Err(SystemError::TaskNotFound(task_id))
	}

	fn pause_suspended_task(&mut self, task_id: TaskId) -> bool {
		if let Some(suspended_task) = &self.suspended_task {
			if suspended_task.task.id() == task_id {
				trace!(
					"Task is already suspended but will be paused: <worker_id='{}', task_id='{task_id}'>",
					self.worker_id
				);

				self.paused_tasks.insert(
					task_id,
					self.suspended_task.take().expect("we just checked it"),
				);

				return true;
			}
		}

		false
	}

	fn pause_task_from_queues(&mut self, task_id: TaskId) -> bool {
		if let Some(index) = self
			.priority_tasks
			.iter()
			.position(|task_work_state| task_work_state.task.id() == task_id)
		{
			self.paused_tasks.insert(
				task_id,
				self.priority_tasks
					.remove(index)
					.expect("we just checked it"),
			);

			return true;
		}

		if let Some(index) = self
			.tasks
			.iter()
			.position(|task_work_state| task_work_state.task.id() == task_id)
		{
			self.paused_tasks.insert(
				task_id,
				self.tasks.remove(index).expect("we just checked it"),
			);

			return true;
		}

		false
	}

	pub(super) fn cancel_not_running_task(&mut self, task_id: TaskId) {
		trace!(
			"Cancel not running task request: <worker_id='{}', task_id='{task_id}'>",
			self.worker_id
		);

		if let Some(current_task) = &self.current_task_handle {
			if current_task.task_id == task_id {
				trace!(
					"Task began to run before we managed to cancel it, run function will cancel it: \
					<worker_id='{}', task_id='{task_id}'>",
					self.worker_id
				);
				return; // The task will cancel itself
			}
		}

		if let Some(suspended_task) = &self.suspended_task {
			if suspended_task.task.id() == task_id {
				trace!(
					"Task is already suspended but will be paused: <worker_id='{}', task_id='{task_id}'>",
					self.worker_id
				);

				send_cancel_task_response(
					self.worker_id,
					task_id,
					self.suspended_task.take().expect("we just checked it"),
				);

				return;
			}
		}

		self.cancel_task_from_queues(task_id);

		// If the task is not found, then it's possible that the user already canceled it but still have the handle
	}

	fn cancel_task_from_queues(&mut self, task_id: TaskId) {
		if let Some(index) = self
			.priority_tasks
			.iter()
			.position(|task_work_state| task_work_state.task.id() == task_id)
		{
			send_cancel_task_response(
				self.worker_id,
				task_id,
				self.priority_tasks
					.remove(index)
					.expect("we just checked it"),
			);

			return;
		}

		if let Some(index) = self
			.tasks
			.iter()
			.position(|task_work_state| task_work_state.task.id() == task_id)
		{
			send_cancel_task_response(
				self.worker_id,
				task_id,
				self.tasks.remove(index).expect("we just checked it"),
			);
		}
	}

	#[inline]
	fn add_task_when_idle(
		&mut self,
		task_id: TaskId,
		task_kind: PendingTaskKind,
		task_work_state: TaskWorkState<E>,
	) {
		trace!(
			"Idle worker will process the new task: <worker_id='{}', task_id='{task_id}'>",
			self.worker_id
		);
		let handle = self.spawn_task_runner(task_id, task_work_state);

		self.current_task_handle = Some(RunningTask {
			task_id,
			task_kind,
			handle,
		});

		// Doesn't need to report working back to system as it already registered
		// that we're not idle anymore when it dispatched the task to this worker
		self.is_idle = false;
	}

	#[inline]
	pub(super) async fn inner_add_task(
		&mut self,
		task_id: TaskId,
		task_kind: PendingTaskKind,
		task_work_state: TaskWorkState<E>,
	) -> TaskAddStatus {
		if self.is_idle {
			self.add_task_when_idle(task_id, task_kind, task_work_state);
			TaskAddStatus::Running
		} else {
			let RunningTask {
				task_id: old_task_id,
				task_kind: old_kind,
				..
			} = self
				.current_task_handle
				.as_ref()
				.expect("Worker isn't idle, but no task is running");

			trace!(
				"Worker is busy: \
				<worker_id='{}', task_id='{task_id}', current_task_kind='{old_kind:#?}'>",
				self.worker_id,
			);

			let add_status = match (task_kind, old_kind) {
				(PendingTaskKind::Priority, PendingTaskKind::Priority) => {
					trace!(
						"Old and new tasks have priority, will put new task on priority queue: \
						<worker_id='{}', task_id='{task_id}'>",
						self.worker_id
					);
					self.priority_tasks.push_front(task_work_state);

					TaskAddStatus::Enqueued
				}
				(PendingTaskKind::Priority, PendingTaskKind::Normal) => {
					if self.waiting_suspension.is_waiting() {
						trace!(
							"Worker is already waiting for a task to be suspended, will enqueue new task: \
							<worker_id='{}', task_id='{task_id}'>",
							self.worker_id
						);

						self.priority_tasks.push_front(task_work_state);
					} else {
						trace!(
							"Old task will be suspended: \
						<worker_id='{}', new_task_id='{task_id}', old_task_id='{old_task_id}'>",
							self.worker_id
						);

						// We put the query at the top of the priority queue, so it will be
						// dispatched by the run function as soon as the current task is suspended
						self.priority_tasks.push_front(task_work_state);

						if self
							.abort_and_suspend_map
							.remove(old_task_id)
							.expect("we always store the abort and suspend signalers")
							.suspend_tx
							.send(())
							.is_err()
						{
							warn!(
							"Task <id='{old_task_id}'> suspend channel closed before receiving suspend signal. \
							This probably happened because the task finished before we could suspend it."
						);
						}

						self.waiting_suspension = WaitingSuspendedTask::Task(*old_task_id);
					}

					TaskAddStatus::Running
				}
				(_, _) => {
					trace!(
						"New task doesn't have priority and will be enqueued: \
						<worker_id='{}', task_id='{task_id}'>",
						self.worker_id,
					);

					self.tasks.push_back(task_work_state);

					TaskAddStatus::Enqueued
				}
			};

			let task_count = self.total_tasks();

			trace!(
				"Worker with {task_count} pending tasks: <worker_id='{}'>",
				self.worker_id
			);

			if task_count > self.work_stealer.workers_count()
				&& self.last_requested_help.elapsed() > ONE_SECOND
			{
				trace!(
					"Worker requesting help from the system: \
					<worker_id='{}', task_count='{task_count}'>",
					self.worker_id
				);

				self.system_comm
					.request_help(self.worker_id, task_count)
					.await;

				self.last_requested_help = Instant::now();
			}

			add_status
		}
	}

	pub(super) async fn force_task_abortion(
		&mut self,
		task_id: uuid::Uuid,
	) -> Result<(), SystemError> {
		if let Some(AbortAndSuspendSignalers { abort_tx, .. }) =
			self.abort_and_suspend_map.remove(&task_id)
		{
			let (tx, rx) = oneshot::channel();

			if abort_tx.send(tx).is_err() {
				debug!(
					"Failed to send force abortion request, the task probably finished before we could abort it: \
					<worker_id='{}', task_id='{task_id}'>",
					self.worker_id
				);

				Ok(())
			} else {
				match timeout(ONE_SECOND, rx).await {
					Ok(Ok(res)) => res,
					// If the sender was dropped, then the task finished before we could
					// abort it which is fine
					Ok(Err(_)) => Ok(()),
					Err(_) => Err(SystemError::TaskForcedAbortTimeout(task_id)),
				}
			}
		} else {
			trace!(
				"Forced abortion of a not running task request: <worker_id='{}', task_id='{task_id}'>",
				self.worker_id
			);

			if let Some(current_task) = &self.current_task_handle {
				if current_task.task_id == task_id {
					trace!(
					"Task began to run before we managed to abort it, run function will abort it: \
					<worker_id='{}', task_id='{task_id}'>",
					self.worker_id
				);
					return Ok(()); // The task will abort itself
				}
			}

			if let Some(suspended_task) = &self.suspended_task {
				if suspended_task.task.id() == task_id {
					trace!(
					"Task is already suspended but will be paused: <worker_id='{}', task_id='{task_id}'>",
					self.worker_id
				);

					send_forced_abortion_task_response(
						self.worker_id,
						task_id,
						self.suspended_task.take().expect("we just checked it"),
					);

					return Ok(());
				}
			}

			if let Some(index) = self
				.priority_tasks
				.iter()
				.position(|task_work_state| task_work_state.task.id() == task_id)
			{
				send_forced_abortion_task_response(
					self.worker_id,
					task_id,
					self.priority_tasks
						.remove(index)
						.expect("we just checked it"),
				);

				return Ok(());
			}

			if let Some(index) = self
				.tasks
				.iter()
				.position(|task_work_state| task_work_state.task.id() == task_id)
			{
				send_forced_abortion_task_response(
					self.worker_id,
					task_id,
					self.tasks.remove(index).expect("we just checked it"),
				);

				return Ok(());
			}

			// If the task is not found, then it's possible that the user already aborted it but still have the handle
			Ok(())
		}
	}

	pub(super) async fn shutdown(mut self, tx: oneshot::Sender<()>) {
		trace!(
			"Worker beginning shutdown process: <worker_id='{}'>",
			self.worker_id
		);

		trace!(
			"Aborting steal task for shutdown if there is one running: <worker_id='{}'>",
			self.worker_id
		);

		self.abort_steal_task();

		let Self {
			worker_id,
			tasks,
			paused_tasks,
			priority_tasks,
			is_idle,
			abort_and_suspend_map,
			msgs_tx: runner_tx,
			mut current_task_handle,
			suspend_on_shutdown_rx,
			..
		} = self;

		if is_idle {
			trace!("Worker is idle, no tasks to shutdown: <worker_id='{worker_id}'>");
		} else {
			trace!("Worker is busy, will shutdown tasks: <worker_id='{worker_id}'>");

			if let Some(RunningTask {
				task_id, handle, ..
			}) = current_task_handle.take()
			{
				for (task_id, AbortAndSuspendSignalers { suspend_tx, .. }) in abort_and_suspend_map
				{
					if suspend_tx.send(()).is_err() {
						warn!(
							"Shutdown request channel closed before sending abort signal: \
								<worker_id='{worker_id}', task_id='{task_id}'>"
						);
					} else {
						trace!(
							"Sent suspend signal for task on shutdown: \
								<worker_id='{worker_id}', task_id='{task_id}'>"
						);
					}
				}

				if let Err(e) = handle.await {
					error!("Task <worker_id='{worker_id}', task_id='{task_id}'> failed to join: {e:#?}");
				}

				runner_tx.close();

				Self::process_tasks_being_suspended_on_shutdown(worker_id, suspend_on_shutdown_rx)
					.await;
			}

			priority_tasks
				.into_iter()
				.chain(paused_tasks.into_values())
				.chain(tasks.into_iter())
				.for_each(|task_work_state| {
					send_shutdown_task_response(
						worker_id,
						task_work_state.task.id(),
						task_work_state,
					);
				});
		}

		trace!("Worker shutdown process completed: <worker_id='{worker_id}'>");

		if tx.send(()).is_err() {
			warn!("Shutdown request channel closed before sending ack");
		}
	}

	async fn process_tasks_being_suspended_on_shutdown(
		worker_id: WorkerId,
		suspend_on_shutdown_rx: chan::Receiver<RunnerMessage<E>>,
	) {
		let mut suspend_on_shutdown_rx = pin!(suspend_on_shutdown_rx);

		while let Some(runner_msg) = suspend_on_shutdown_rx.next().await {
			match runner_msg {
				RunnerMessage::TaskOutput(task_id, res) => match res {
					Ok(TaskRunnerOutput {
						task_work_state,
						status,
					}) => match status {
						InternalTaskExecStatus::Done(out) => {
							send_complete_task_response(worker_id, task_id, task_work_state, out);
						}

						InternalTaskExecStatus::Canceled => {
							send_cancel_task_response(worker_id, task_id, task_work_state);
						}

						InternalTaskExecStatus::Suspend | InternalTaskExecStatus::Paused => {
							send_shutdown_task_response(worker_id, task_id, task_work_state);
						}

						InternalTaskExecStatus::Error(e) => {
							send_error_task_response(worker_id, task_id, task_work_state, e);
						}
					},
					Err(()) => {
						error!(
							"Task <worker_id='{worker_id}', task_id='{task_id}'> failed to suspend on shutdown"
						);
					}
				},

				RunnerMessage::StoleTask(Some(task_work_state)) => {
					send_shutdown_task_response(
						worker_id,
						task_work_state.task.id(),
						task_work_state,
					);
				}

				RunnerMessage::StoleTask(None) => {}
			}
		}
	}

	pub(super) fn get_next_task(&mut self) -> Option<(PendingTaskKind, TaskWorkState<E>)> {
		if let Some(task) = self.priority_tasks.pop_front() {
			return Some((PendingTaskKind::Priority, task));
		}

		if let Some(task) = self.suspended_task.take() {
			task.interrupter.reset();
			task.worktable.set_unpause();
			return Some((PendingTaskKind::Suspended, task));
		}

		self.tasks
			.pop_front()
			.map(|task| (PendingTaskKind::Normal, task))
	}

	pub(super) fn steal_request(&mut self, tx: oneshot::Sender<Option<TaskWorkState<E>>>) {
		trace!("Steal request: <worker_id='{}'>", self.worker_id);
		if let Some((kind, task_work_state)) = self.get_next_task() {
			self.proceed_with_task_to_be_stolen(kind, task_work_state, tx);
		} else {
			trace!("No task to steal: <worker_id='{}'>", self.worker_id);
			if tx.send(None).is_err() {
				warn!(
					"Steal request channel closed before sending no task response: \
					<worker_id='{}'>",
					self.worker_id
				);
			}
		}
	}

	fn proceed_with_task_to_be_stolen(
		&mut self,
		kind: PendingTaskKind,
		task_work_state: TaskWorkState<E>,
		tx: oneshot::Sender<Option<TaskWorkState<E>>>,
	) {
		let task_id = task_work_state.task.id();
		self.task_kinds.remove(&task_id);

		trace!(
			"Stealing task: <worker_id='{}', task_id='{task_id}', kind='{kind:#?}'>",
			self.worker_id
		);

		if let Err(Some(task_work_state)) = tx.send(Some(task_work_state)) {
			self.put_back_failed_to_stole_task(task_id, kind, task_work_state);
		}
	}

	fn put_back_failed_to_stole_task(
		&mut self,
		id: TaskId,
		kind: PendingTaskKind,
		task_work_state: TaskWorkState<E>,
	) {
		warn!(
			"Steal request channel closed before sending task: <worker_id='{}'>",
			self.worker_id
		);
		match kind {
			PendingTaskKind::Normal => self.tasks.push_front(task_work_state),
			PendingTaskKind::Priority => self.priority_tasks.push_front(task_work_state),
			PendingTaskKind::Suspended => self.suspended_task = Some(task_work_state),
		}

		self.task_kinds.insert(id, kind);
	}

	pub(super) fn wake_up(&mut self) {
		if self.is_idle {
			trace!(
				"Worker is idle, waking up: <worker_id='{}'>",
				self.worker_id
			);

			if self.current_steal_task_handle.is_none() {
				self.current_steal_task_handle = Some(dispatch_steal_request(
					self.worker_id,
					self.work_stealer.clone(),
					self.msgs_tx.clone(),
				));
			} else {
				trace!(
					"Steal task already running, ignoring wake up request: <worker_id='{}'>",
					self.worker_id
				);
			}
		} else {
			trace!(
				"Worker already working, ignoring wake up request: <worker_id='{}'>",
				self.worker_id
			);
		}
	}

	#[inline]
	pub(super) async fn dispatch_next_task(&mut self, finished_task_id: TaskId) {
		trace!(
			"Task finished and will try to process a new task: \
			<worker_id='{}', finished_task_id='{finished_task_id}'>",
			self.worker_id
		);

		self.abort_and_suspend_map.remove(&finished_task_id);

		let RunningTask {
			task_id: old_task_id,

			handle,
			..
		} = self
			.current_task_handle
			.take()
			.expect("Task handle missing, but task output received");

		assert_eq!(finished_task_id, old_task_id, "Task output id mismatch");

		trace!(
			"Waiting task handle: <worker_id='{}', task_id='{old_task_id}'>",
			self.worker_id
		);
		if let Err(e) = handle.await {
			error!("Task <id='{old_task_id}'> failed to join: {e:#?}");
		}
		trace!(
			"Waited task handle: <worker_id='{}', task_id='{old_task_id}'>",
			self.worker_id
		);

		if let Some((task_kind, task_work_state)) = self.get_next_task() {
			let task_id = task_work_state.task.id();

			trace!(
				"Dispatching next task: <worker_id='{}', task_id='{task_id}', kind='{task_kind:#?}'>",
				self.worker_id
			);

			let handle = self.spawn_task_runner(task_id, task_work_state);

			self.current_task_handle = Some(RunningTask {
				task_id,
				task_kind,
				handle,
			});
		} else {
			trace!(
				"No task to dispatch, worker is now idle and will dispatch a steal request: <worker_id='{}'>",
				self.worker_id
			);

			self.is_idle = true;
			self.system_comm.idle_report(self.worker_id).await;

			if self.current_steal_task_handle.is_none() {
				self.current_steal_task_handle = Some(dispatch_steal_request(
					self.worker_id,
					self.work_stealer.clone(),
					self.msgs_tx.clone(),
				));
			} else {
				trace!(
					"Steal task already running: <worker_id='{}'>",
					self.worker_id
				);
			}
		}
	}

	pub(super) async fn process_task_output(
		&mut self,
		task_id: TaskId,
		TaskRunnerOutput {
			task_work_state,
			status,
		}: TaskRunnerOutput<E>,
	) {
		match status {
			InternalTaskExecStatus::Done(out) => {
				self.task_kinds.remove(&task_id);
				send_complete_task_response(self.worker_id, task_id, task_work_state, out);
			}

			InternalTaskExecStatus::Paused => {
				self.paused_tasks.insert(task_id, task_work_state);
				trace!(
					"Task paused: <worker_id='{}', task_id='{task_id}'>",
					self.worker_id
				);
			}

			InternalTaskExecStatus::Canceled => {
				self.task_kinds.remove(&task_id);
				send_cancel_task_response(self.worker_id, task_id, task_work_state);
			}

			InternalTaskExecStatus::Error(e) => {
				self.task_kinds.remove(&task_id);
				send_error_task_response(self.worker_id, task_id, task_work_state, e);
			}

			InternalTaskExecStatus::Suspend => {
				self.suspended_task = Some(task_work_state);
				trace!(
					"Task suspended: <worker_id='{}', task_id='{task_id}'>",
					self.worker_id
				);

				self.clean_suspended_task(task_id);
			}
		}

		trace!(
			"Processing task output completed and will try to dispatch a new task: \
			<worker_id='{}', task_id='{task_id}'>",
			self.worker_id
		);

		self.dispatch_next_task(task_id).await;
	}

	pub(super) fn idle_check(&mut self) {
		if self.is_idle {
			trace!(
				"Worker is idle for some time and will try to steal a task: <worker_id='{}'>",
				self.worker_id
			);

			if self.current_steal_task_handle.is_none() {
				self.steal_attempt();
			} else {
				trace!(
					"Steal task already running, ignoring on this idle check: <worker_id='{}'>",
					self.worker_id
				);
			}

			self.idle_memory_cleanup();
		}
	}

	fn steal_attempt(&mut self) {
		let elapsed = self.last_steal_attempt_at.elapsed();
		let required = (TEN_SECONDS * self.steal_attempts_count).min(ONE_MINUTE);
		trace!(
			"Steal attempt required cool down: \
			<worker_id='{}', elapsed='{elapsed:?}', required='{required:?}', steal_attempts_count={}>",
			self.worker_id,
			self.steal_attempts_count
		);
		if elapsed > required {
			self.current_steal_task_handle = Some(dispatch_steal_request(
				self.worker_id,
				self.work_stealer.clone(),
				self.msgs_tx.clone(),
			));
			self.last_steal_attempt_at = Instant::now();
		} else {
			trace!(
				"Steal attempt still cooling down: <worker_id='{}', steal_attempts_count={}>",
				self.worker_id,
				self.steal_attempts_count
			);
		}
	}

	fn idle_memory_cleanup(&mut self) {
		// As we're idle, let's check if we need to do some memory cleanup
		if self.tasks.capacity() > TASK_QUEUE_INITIAL_SIZE {
			assert_eq!(self.tasks.len(), 0);
			self.tasks.shrink_to(TASK_QUEUE_INITIAL_SIZE);
		}

		if self.task_kinds.capacity() > TASK_QUEUE_INITIAL_SIZE {
			assert_eq!(self.task_kinds.len(), self.paused_tasks.len());
			self.task_kinds.shrink_to(TASK_QUEUE_INITIAL_SIZE);
		}

		if self.priority_tasks.capacity() > PRIORITY_TASK_QUEUE_INITIAL_SIZE {
			assert_eq!(self.priority_tasks.len(), 0);
			self.priority_tasks
				.shrink_to(PRIORITY_TASK_QUEUE_INITIAL_SIZE);
		}

		if self.paused_tasks.capacity() != self.paused_tasks.len() {
			self.paused_tasks.shrink_to_fit();
		}

		if self.abort_and_suspend_map.capacity() > ABORT_AND_SUSPEND_MAP_INITIAL_SIZE {
			assert!(self.abort_and_suspend_map.len() < ABORT_AND_SUSPEND_MAP_INITIAL_SIZE);
			self.abort_and_suspend_map
				.shrink_to(ABORT_AND_SUSPEND_MAP_INITIAL_SIZE);
		}
	}

	pub(super) fn abort_steal_task(&mut self) {
		if let Some(steal_task_handle) = self.current_steal_task_handle.take() {
			steal_task_handle.abort();
			trace!("Aborted steal task: <worker_id='{}'>", self.worker_id);
		} else {
			trace!("No steal task to abort: <worker_id='{}'>", self.worker_id);
		}
	}

	pub(super) async fn process_stolen_task(&mut self, maybe_new_task: Option<TaskWorkState<E>>) {
		if let Some(steal_task_handle) = self.current_steal_task_handle.take() {
			if let Err(e) = steal_task_handle.await {
				error!("Steal task failed to join: {e:#?}");
			}
		}

		if let Some(task_work_state) = maybe_new_task {
			self.system_comm.working_report(self.worker_id).await;
			trace!(
				"Stolen task: <worker_id='{}', task_id='{}'>",
				self.worker_id,
				task_work_state.task.id()
			);
			self.steal_attempts_count = 0;
			self.new_task(task_work_state).await;
		} else {
			self.steal_attempts_count += 1;
		}
	}

	pub(crate) fn clean_suspended_task(&mut self, task_id: uuid::Uuid) {
		match self.waiting_suspension {
			WaitingSuspendedTask::Task(waiting_task_id) if waiting_task_id == task_id => {
				trace!(
					"Task was suspended and will be cleaned: <worker_id='{}', task_id='{task_id}'>",
					self.worker_id
				);
				self.waiting_suspension = WaitingSuspendedTask::None;
			}
			WaitingSuspendedTask::Task(_) => {
				trace!(
					"Task wasn't suspended, ignoring: <worker_id='{}', task_id='{task_id}'>",
					self.worker_id
				);
			}
			WaitingSuspendedTask::None => {}
		}
	}
}

type RunTaskOutput<E> = (Box<dyn Task<E>>, Result<Result<ExecStatus, E>, SystemError>);

fn handle_run_task_attempt<E: RunError>(
	worker_id: WorkerId,
	task_id: TaskId,
	mut task: Box<dyn Task<E>>,
	worktable: &TaskWorktable,
	interrupter: Arc<Interrupter>,
) -> JoinHandle<RunTaskOutput<E>> {
	spawn({
		let already_paused = worktable.is_paused();
		let already_canceled = worktable.is_canceled();
		let already_aborted = worktable.is_aborted();

		async move {
			if already_paused {
				trace!(
					"Task was paused before running: <worker_id='{worker_id}', task_id='{task_id}'>"
				);

				(task, Ok(Ok(ExecStatus::Paused)))
			} else if already_canceled {
				trace!(
					"Task was canceled before running: <worker_id='{worker_id}', task_id='{task_id}'>"
				);

				(task, Ok(Ok(ExecStatus::Canceled)))
			} else if already_aborted {
				trace!(
					"Task was aborted before running: <worker_id='{worker_id}', task_id='{task_id}'>"
				);

				(task, Err(SystemError::TaskAborted(task_id)))
			} else {
				let run_result = if let Some(timeout_duration) = task.with_timeout() {
					(task.run(&interrupter).map(Ok), async move {
						sleep(timeout_duration)
							.map(|()| Err(SystemError::TaskTimeout(task_id)))
							.await
					})
						.race()
						.await
				} else {
					task.run(&interrupter).map(Ok).await
				};

				match run_result {
					Ok(res) => {
						trace!("Ran task: <worker_id='{worker_id}', task_id='{task_id}'>: {res:?}");

						(task, Ok(res))
					}
					Err(e) => (task, Err(e)),
				}
			}
		}
	})
}

fn handle_task_suspension(
	worker_id: WorkerId,
	task_id: TaskId,
	has_suspended: Arc<AtomicBool>,
	worktable: Arc<TaskWorktable>,
	suspend_rx: oneshot::Receiver<()>,
) -> JoinHandle<()> {
	spawn(async move {
		if suspend_rx.await.is_ok() {
			let (tx, rx) = oneshot::channel();

			trace!("Suspend signal received: <worker_id='{worker_id}', task_id='{task_id}'>");

			// The interrupter only knows about Pause and Cancel commands, we use pause as
			// the suspend task feature should be invisible to the user
			worktable.pause(tx).await;

			match rx.await {
				Ok(()) => {
					trace!("Suspending: <worker_id='{worker_id}', task_id='{task_id}'>");
					has_suspended.store(true, Ordering::Relaxed);
				}
				Err(_) => {
					// The task probably finished before we could suspend it so the channel was dropped
					trace!(
						"Suspend channel closed: <worker_id='{worker_id}', task_id='{task_id}'>"
					);
				}
			}
		} else {
			trace!(
				"Suspend channel closed, task probably finished before we could suspend it: \
					<worker_id='{worker_id}', task_id='{task_id}'>"
			);
		}
	})
}

type PartialTaskWorkState<E> = (
	TaskId,
	Arc<TaskWorktable>,
	oneshot::Sender<Result<TaskStatus<E>, SystemError>>,
	Arc<Interrupter>,
);

async fn emit_task_completed_message<E: RunError>(
	worker_id: WorkerId,
	run_task_output: RunTaskOutput<E>,
	has_suspended: Arc<AtomicBool>,
	(task_id, worktable, done_tx, interrupter): PartialTaskWorkState<E>,
	runner_tx: chan::Sender<RunnerMessage<E>>,
) {
	match run_task_output {
		(task, Ok(res)) => {
			trace!(
				"Task completed ok: <worker_id='{worker_id}', task_id='{task_id}', result={res:?}>"
			);
			runner_tx
				.send(RunnerMessage::TaskOutput(task_id, {
					let mut internal_status = res.into();

					if matches!(internal_status, InternalTaskExecStatus::Paused)
						&& has_suspended.load(Ordering::Relaxed)
					{
						internal_status = InternalTaskExecStatus::Suspend;
					}

					Ok(TaskRunnerOutput {
						task_work_state: TaskWorkState {
							task,
							worktable,
							done_tx,
							interrupter,
						},
						status: internal_status,
					})
				}))
				.await
				.expect("Task runner channel closed while sending task output");
		}

		(_, Err(e)) => {
			trace!("Task had an error: <worker_id='{worker_id}', task_id='{task_id}'>");

			if done_tx
				.send(if matches!(e, SystemError::TaskAborted(_)) {
					Ok(TaskStatus::ForcedAbortion)
				} else {
					Err(e)
				})
				.is_err()
			{
				error!("Task done channel closed while sending error response");
			}

			runner_tx
				.send(RunnerMessage::TaskOutput(task_id, Err(())))
				.await
				.expect("Task runner channel closed while sending task output");
		}
	}
}

async fn run_single_task<E: RunError>(
	worker_id: WorkerId,
	TaskWorkState {
		task,
		worktable,
		interrupter,
		done_tx,
	}: TaskWorkState<E>,
	runner_tx: chan::Sender<RunnerMessage<E>>,
	suspend_rx: oneshot::Receiver<()>,
	abort_rx: oneshot::Receiver<oneshot::Sender<Result<(), SystemError>>>,
) {
	enum RaceOutput<E: RunError> {
		Completed(Result<RunTaskOutput<E>, JoinError>),
		Abort(oneshot::Sender<Result<(), SystemError>>),
	}

	let task_id = task.id();

	worktable.set_started();

	trace!("Running task: <worker_id='{worker_id}', task_id='{task_id}'>");

	let handle = handle_run_task_attempt(
		worker_id,
		task_id,
		task,
		&worktable,
		Arc::clone(&interrupter),
	);

	let task_abort_handle = handle.abort_handle();

	let has_suspended = Arc::new(AtomicBool::new(false));

	let suspender_handle = handle_task_suspension(
		worker_id,
		task_id,
		Arc::clone(&has_suspended),
		Arc::clone(&worktable),
		suspend_rx,
	);

	match (async { RaceOutput::Completed(handle.await) }, async move {
		if let Ok(tx) = abort_rx.await {
			trace!("Aborting task: <worker_id='{worker_id}', task_id='{task_id}'>");
			RaceOutput::Abort(tx)
		} else {
			// If the abort channel is closed, we should just ignore it and keep waiting for the task to finish
			// as we're being suspended by the worker
			trace!(
				"Abort channel closed, will wait for task to finish: <worker_id='{worker_id}', task_id='{task_id}'>"
			);
			pending().await
		}
	})
		.race()
		.await
	{
		RaceOutput::Completed(Ok(run_task_output)) => {
			emit_task_completed_message(
				worker_id,
				run_task_output,
				has_suspended,
				(task_id, worktable, done_tx, interrupter),
				runner_tx,
			)
			.await;
		}

		RaceOutput::Completed(Err(join_error)) => {
			error!("Task <id='{task_id}'> failed to join: {join_error:#?}",);
			if done_tx.send(Err(SystemError::TaskJoin(task_id))).is_err() {
				error!("Task done channel closed while sending join error response");
			}

			if runner_tx
				.send(RunnerMessage::TaskOutput(task_id, Err(())))
				.await
				.is_err()
			{
				error!("Task runner channel closed while sending join error response");
			}
		}

		RaceOutput::Abort(tx) => {
			task_abort_handle.abort();

			trace!("Task aborted: <worker_id='{worker_id}', task_id='{task_id}'>");

			if done_tx.send(Ok(TaskStatus::ForcedAbortion)).is_err() {
				error!("Task done channel closed while sending abort error response");
			}

			if runner_tx
				.send(RunnerMessage::TaskOutput(task_id, Err(())))
				.await
				.is_err()
			{
				error!("Task runner channel closed while sending abort error response");
			}

			if tx.send(Ok(())).is_err() {
				error!("Task abort channel closed while sending abort error response");
			}
		}
	}

	if !suspender_handle.is_finished() {
		trace!(
			"Aborting suspender handler as it isn't needed anymore: <worker_id='{worker_id}', task_id='{task_id}'>"
		);
		// if we received a suspend signal this abort will do nothing, as the task finished already
		suspender_handle.abort();
	}

	trace!("Run single task finished: <worker_id='{worker_id}', task_id='{task_id}'>");
}

fn send_complete_task_response<E: RunError>(
	worker_id: WorkerId,
	task_id: TaskId,
	TaskWorkState {
		done_tx, worktable, ..
	}: TaskWorkState<E>,
	out: TaskOutput,
) {
	worktable.set_completed();
	if done_tx.send(Ok(TaskStatus::Done((task_id, out)))).is_err() {
		warn!(
			"Task done channel closed before sending done response for task: \
		<worker_id='{worker_id}', task_id='{task_id}'>"
		);
	} else {
		trace!(
			"Emitted task done signal on shutdown: \
		<worker_id='{worker_id}', task_id='{task_id}'>"
		);
	}
}

fn send_cancel_task_response<E: RunError>(
	worker_id: WorkerId,
	task_id: TaskId,
	TaskWorkState {
		done_tx, worktable, ..
	}: TaskWorkState<E>,
) {
	worktable.set_completed();
	if done_tx.send(Ok(TaskStatus::Canceled)).is_err() {
		warn!(
			"Task done channel closed before sending canceled response for task: \
			<worker_id='{worker_id}', task_id='{task_id}'>",
		);
	} else {
		trace!(
			"Emitted task canceled signal on cancel not running task: \
			<worker_id='{worker_id}', task_id='{task_id}'>",
		);
	}
}

fn send_shutdown_task_response<E: RunError>(
	worker_id: WorkerId,
	task_id: TaskId,
	TaskWorkState { task, done_tx, .. }: TaskWorkState<E>,
) {
	if done_tx.send(Ok(TaskStatus::Shutdown(task))).is_err() {
		warn!(
			"Task done channel closed before sending shutdown response for task: \
			<worker_id='{worker_id}', task_id='{task_id}'>"
		);
	} else {
		trace!(
			"Successfully suspended and sent back DynTask on worker shutdown: \
			<worker_id='{worker_id}', task_id='{task_id}'>"
		);
	}
}

fn send_error_task_response<E: RunError>(
	worker_id: usize,
	task_id: uuid::Uuid,
	TaskWorkState {
		done_tx, worktable, ..
	}: TaskWorkState<E>,
	e: E,
) {
	worktable.set_completed();
	if done_tx.send(Ok(TaskStatus::Error(e))).is_err() {
		warn!(
			"Task done channel closed before sending error response for task: \
			<worker_id='{worker_id}', task_id='{task_id}'>"
		);
	} else {
		trace!(
			"Emitted task error signal on shutdown: \
			<worker_id='{worker_id}', task_id='{task_id}'>"
		);
	}
}

fn send_forced_abortion_task_response<E: RunError>(
	worker_id: WorkerId,
	task_id: TaskId,
	TaskWorkState {
		done_tx, worktable, ..
	}: TaskWorkState<E>,
) {
	worktable.set_completed();
	if done_tx.send(Ok(TaskStatus::ForcedAbortion)).is_err() {
		warn!(
			"Task done channel closed before sending forced abortion response for task: \
			<worker_id='{worker_id}', task_id='{task_id}'>",
		);
	} else {
		trace!(
			"Emitted task forced abortion signal on cancel not running task: \
			<worker_id='{worker_id}', task_id='{task_id}'>",
		);
	}
}
