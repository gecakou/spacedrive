use std::{ops::Deref, sync::Arc};

use sd_prisma::{
	prisma::{crdt_operation, SortOrder},
	prisma_sync::ModelSyncData,
};
use sd_sync::CRDTOperation;
use tokio::sync::{mpsc, oneshot, Mutex};
use tracing::debug;
use uhlc::{Timestamp, NTP64};
use uuid::Uuid;

use crate::{
	actor::{create_actor_io, ActorIO, ActorTypes},
	db_operation::write_crdt_op_to_db,
	wait, SharedState,
};

#[derive(Debug)]
#[must_use]
/// Stuff that can be handled outside the actor
pub enum Request {
	Messages {
		timestamps: Vec<(Uuid, NTP64)>,
		tx: oneshot::Sender<()>,
	},
	Ingested,
	FinishedIngesting,
}

/// Stuff that the actor consumes
#[derive(Debug)]
pub enum Event {
	Notification,
	Messages(MessagesEvent),
}

#[derive(Debug, Default)]
pub enum State {
	#[default]
	WaitingForNotification,
	RetrievingMessages,
	Ingesting(MessagesEvent),
}

/// The single entrypoint for sync operation ingestion.
/// Requests sync operations in a given timestamp range,
/// and attempts to write them to the syn coperations table along with
/// the actual cell that the operation points to.
///
/// If this actor stops running, no sync operations will
/// be applied to the database, independent of whether systems like p2p
/// or cloud are exchanging messages.
pub struct Actor {
	state: Option<State>,
	shared: Arc<SharedState>,
	io: ActorIO<Self>,
}

impl Actor {
	async fn tick(mut self) -> Option<Self> {
		let state = match self.state.take()? {
			State::WaitingForNotification => {
				wait!(self.io.event_rx, Event::Notification);

				State::RetrievingMessages
			}
			State::RetrievingMessages => {
				let (tx, mut rx) = oneshot::channel::<()>();

				self.io
					.send(Request::Messages {
						timestamps: self
							.timestamps
							.read()
							.await
							.iter()
							.map(|(&k, &v)| (k, v))
							.collect(),
						tx,
					})
					.await
					.ok();

				loop {
					tokio::select! {
						biased;
						res = self.io.event_rx.recv() => {
							if let Some(Event::Messages(event)) = res { break State::Ingesting(event) }
						}
						res = &mut rx => {
							if let Err(_) = res {
								debug!("messages request ignored");
								break State::WaitingForNotification
							 }
						},
					}
				}
			}
			State::Ingesting(event) => {
				if event.messages.len() > 0 {
					debug!(
						"ingesting {} operations: {} to {}",
						event.messages.len(),
						event.messages.first().unwrap().timestamp.as_u64(),
						event.messages.last().unwrap().timestamp.as_u64(),
					);

					for op in event.messages {
						self.receive_crdt_operation(op).await;
					}

					debug!("done ingesting");
				}

				match event.has_more {
					true => State::RetrievingMessages,
					false => {
						self.io.send(Request::FinishedIngesting).await.ok();

						State::WaitingForNotification
					}
				}
			}
		};

		debug!("new ingest actor state: {state:#?}");

		Some(Self {
			state: Some(state),
			..self
		})
	}

	pub fn spawn(shared: Arc<SharedState>) -> Handler {
		let (actor_io, handler_io) = create_actor_io::<Self>();

		tokio::spawn(async move {
			let mut this = Self {
				state: Some(Default::default()),
				io: actor_io,
				shared,
			};

			loop {
				this = match this.tick().await {
					Some(this) => this,
					None => break,
				};
			}
		});

		Handler {
			event_tx: handler_io.event_tx,
			req_rx: Arc::new(Mutex::new(handler_io.req_rx)),
		}
	}

	// where the magic happens
	async fn receive_crdt_operation(&mut self, op: CRDTOperation) {
		debug!("receiving operation {}", op.timestamp.as_u64());
		// first, we update the HLC's timestamp with the incoming one.
		// this involves a drift check + sets the last time of the clock
		self.clock
			.update_with_timestamp(&Timestamp::new(op.timestamp, op.instance.into()))
			.expect("timestamp has too much drift!");

		debug!("clock updateed");
		// read the timestamp for the operation's instance, or insert one if it doesn't exist
		let timestamp = self.timestamps.read().await.get(&op.instance).cloned();
		debug!("timestamp retrieved");

		// copy some fields bc rust ownership
		let op_instance = op.instance;
		let op_timestamp = op.timestamp;

		if !self.is_operation_old(&op).await {
			debug!("operation not old");
			// actually go and apply the operation in the db
			self.apply_op(op).await.ok();

			debug!("apply_op done");

			// update the stored timestamp for this instance - will be derived from the crdt operations table on restart
			self.timestamps.write().await.insert(
				op_instance,
				NTP64::max(timestamp.unwrap_or_default(), op_timestamp),
			);
			debug!("timestamp inserted	");
		}

		debug!("doen ingesting operation");
	}

	async fn apply_op(&mut self, op: CRDTOperation) -> prisma_client_rust::Result<()> {
		// self.db
		// 	._transaction()
		// 	.with_timeout(30 * 1000)
		// 	.run(|db| async move {
		debug!("transaction start");

		// apply the operation to the actual record
		let sync_data = ModelSyncData::from_op(op.clone());

		debug!("model sync data exists: {}", sync_data.is_some());

		sync_data.unwrap().exec(&self.db).await?;

		debug!("operation applied");

		// write the operation to the operations table
		write_crdt_op_to_db(&op, &self.db).await?;

		debug!("operation written");

		// Ok(())
		// })
		// .await?;

		debug!("transaction done");

		self.io.req_tx.send(Request::Ingested).await.ok();

		debug!("notification sent");

		Ok(())
	}

	// determines if an operation is old and shouldn't be applied
	async fn is_operation_old(&mut self, op: &CRDTOperation) -> bool {
		let db = &self.db;

		let old_timestamp = {
			let newer_op = db
				.crdt_operation()
				.find_first(vec![
					crdt_operation::timestamp::gte(op.timestamp.as_u64() as i64),
					crdt_operation::model::equals(op.model.to_string()),
					crdt_operation::record_id::equals(serde_json::to_vec(&op.record_id).unwrap()),
					crdt_operation::kind::equals(op.kind().to_string()),
				])
				.order_by(crdt_operation::timestamp::order(SortOrder::Desc))
				.exec()
				.await
				.unwrap();

			newer_op.map(|newer_op| newer_op.timestamp)
		};

		old_timestamp
			.map(|old| old != op.timestamp.as_u64() as i64)
			.unwrap_or_default()
	}
}

impl Deref for Actor {
	type Target = SharedState;

	fn deref(&self) -> &Self::Target {
		&self.shared
	}
}

pub struct Handler {
	pub event_tx: mpsc::Sender<Event>,
	pub req_rx: Arc<Mutex<mpsc::Receiver<Request>>>,
}

#[derive(Debug)]
pub struct MessagesEvent {
	pub instance_id: Uuid,
	pub messages: Vec<CRDTOperation>,
	pub has_more: bool,
}

impl ActorTypes for Actor {
	type Event = Event;
	type Request = Request;
	type Handler = Handler;
}

#[cfg(test)]
mod test {
	use std::sync::atomic::AtomicBool;

	use uhlc::HLCBuilder;

	use super::*;

	async fn new_actor() -> (Handler, Arc<SharedState>) {
		let instance = uuid::Uuid::new_v4();
		let shared = Arc::new(SharedState {
			db: sd_prisma::test_db().await,
			instance,
			clock: HLCBuilder::new().with_id(instance.into()).build(),
			timestamps: Default::default(),
			emit_messages_flag: Arc::new(AtomicBool::new(true)),
		});

		(Actor::spawn(shared.clone()), shared)
	}

	/// If messages tx is dropped, actor should reset and assume no further messages
	/// will be sent
	#[tokio::test]
	async fn messages_request_drop() -> Result<(), ()> {
		let (ingest, _) = new_actor().await;

		for _ in [(), ()] {
			let mut rx = ingest.req_rx.lock().await;

			println!("lock acquired");

			ingest.event_tx.send(Event::Notification).await.unwrap();

			println!("notificaton sent");

			let Some(Request::Messages { .. }) = rx.recv().await else {
				panic!("bruh")
			};

			println!("message received")
		}

		Ok(())
	}

	// /// If messages tx is dropped, actor should reset and assume no further messages
	// /// will be sent
	// #[tokio::test]
	// async fn retrieve_wait() -> Result<(), ()> {
	// 	let (ingest, _) = new_actor().await;

	// 	for _ in [(), ()] {
	// 		let mut rx = ingest.req_rx.lock().await;

	// 		ingest.event_tx.send(Event::Notification).await.unwrap();

	// 		let Some(Request::Messages { .. }) = rx.recv().await else {
	// 			panic!("bruh")
	// 		};
	// 	}

	// 	Ok(())
	// }
}
