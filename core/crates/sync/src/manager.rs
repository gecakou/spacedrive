use crate::{crdt_op_db, db_operation::*, ingest, SharedState, SyncMessage, NTP64};

use sd_prisma::prisma::{cloud_crdt_operation, crdt_operation, instance, PrismaClient, SortOrder};
use sd_sync::{CRDTOperation, OperationFactory};
use sd_utils::uuid_to_bytes;

use std::{
	cmp::Ordering,
	collections::HashMap,
	ops::Deref,
	sync::{
		atomic::{self, AtomicBool},
		Arc,
	},
};

use tokio::sync::{broadcast, RwLock};
use uhlc::{HLCBuilder, HLC};
use uuid::Uuid;

pub struct Manager {
	pub tx: broadcast::Sender<SyncMessage>,
	pub ingest: ingest::Handler,
	shared: Arc<SharedState>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq)]
pub struct GetOpsArgs {
	pub clocks: Vec<(Uuid, NTP64)>,
	pub count: u32,
}

pub struct New {
	pub manager: Manager,
	pub rx: broadcast::Receiver<SyncMessage>,
}

impl Manager {
	#[allow(clippy::new_ret_no_self)]
	pub fn new(
		db: &Arc<PrismaClient>,
		instance: Uuid,
		emit_messages_flag: &Arc<AtomicBool>,
		timestamps: HashMap<Uuid, NTP64>,
	) -> New {
		let (tx, rx) = broadcast::channel(64);

		let clock = HLCBuilder::new().with_id(instance.into()).build();

		let shared = Arc::new(SharedState {
			db: db.clone(),
			instance,
			clock,
			timestamps: Arc::new(RwLock::new(timestamps)),
			emit_messages_flag: emit_messages_flag.clone(),
		});

		let ingest = ingest::Actor::spawn(shared.clone());

		New {
			manager: Self { tx, ingest, shared },
			rx,
		}
	}

	pub fn subscribe(&self) -> broadcast::Receiver<SyncMessage> {
		self.tx.subscribe()
	}

	pub async fn write_ops<'item, I: prisma_client_rust::BatchItem<'item>>(
		&self,
		tx: &PrismaClient,
		(_ops, queries): (Vec<CRDTOperation>, I),
	) -> prisma_client_rust::Result<<I as prisma_client_rust::BatchItemParent>::ReturnValue> {
		let ret = if self.emit_messages_flag.load(atomic::Ordering::Relaxed) {
			let (res, _) = tx
				._batch((
					queries,
					_ops.iter()
						.map(|op| crdt_op_db(op).to_query(tx))
						.collect::<Vec<_>>(),
				))
				.await?;

			self.tx.send(SyncMessage::Created).ok();

			res
		} else {
			tx._batch([queries]).await?.remove(0)
		};

		Ok(ret)
	}

	#[allow(unused_variables)]
	pub async fn write_op<'item, Q: prisma_client_rust::BatchItem<'item>>(
		&self,
		tx: &PrismaClient,
		op: CRDTOperation,
		query: Q,
	) -> prisma_client_rust::Result<<Q as prisma_client_rust::BatchItemParent>::ReturnValue> {
		let ret = if self.emit_messages_flag.load(atomic::Ordering::Relaxed) {
			let ret = tx._batch((crdt_op_db(&op).to_query(tx), query)).await?.1;

			self.tx.send(SyncMessage::Created).ok();

			ret
		} else {
			tx._batch(vec![query]).await?.remove(0)
		};

		Ok(ret)
	}

	pub async fn get_ops(
		&self,
		args: GetOpsArgs,
	) -> prisma_client_rust::Result<Vec<CRDTOperation>> {
		let db = &self.db;

		macro_rules! db_args {
			($args:ident, $op:ident) => {
				vec![prisma_client_rust::operator::or(
					$args
						.clocks
						.iter()
						.map(|(instance_id, timestamp)| {
							prisma_client_rust::and![
								$op::instance::is(vec![instance::pub_id::equals(uuid_to_bytes(
									*instance_id
								))]),
								$op::timestamp::gt(timestamp.as_u64() as i64)
							]
						})
						.chain([
							$op::instance::is_not(vec![
								instance::pub_id::in_vec(
									$args
										.clocks
										.iter()
										.map(|(instance_id, _)| {
											uuid_to_bytes(*instance_id)
										})
										.collect()
								)
							])
						])
						.collect(),
				)]
			};
		}

		let mut ops = db
			.crdt_operation()
			.find_many(db_args!(args, crdt_operation))
			.take(i64::from(args.count))
			.order_by(crdt_operation::timestamp::order(SortOrder::Asc))
			.include(crdt_include::include())
			.exec()
			.await?;

		ops.sort_by(|a, b| match a.timestamp().cmp(&b.timestamp()) {
			Ordering::Equal => a.instance().cmp(&b.instance()),
			o => o,
		});

		Ok(ops
			.into_iter()
			.take(args.count as usize)
			.map(|o| o.into_operation())
			.collect())
	}

	pub async fn get_cloud_ops(
		&self,
		args: GetOpsArgs,
	) -> prisma_client_rust::Result<Vec<CRDTOperation>> {
		let db = &self.db;

		macro_rules! db_args {
			($args:ident, $op:ident) => {
				vec![prisma_client_rust::operator::or(
					$args
						.clocks
						.iter()
						.map(|(instance_id, timestamp)| {
							prisma_client_rust::and![
								$op::instance::is(vec![instance::pub_id::equals(uuid_to_bytes(
									*instance_id
								))]),
								$op::timestamp::gt(timestamp.as_u64() as i64)
							]
						})
						.chain([
							$op::instance::is_not(vec![
								instance::pub_id::in_vec(
									$args
										.clocks
										.iter()
										.map(|(instance_id, _)| {
											uuid_to_bytes(*instance_id)
										})
										.collect()
								)
							])
						])
						.collect(),
				)]
			};
		}

		let mut ops = db
			.cloud_crdt_operation()
			.find_many(db_args!(args, cloud_crdt_operation))
			.take(i64::from(args.count))
			.order_by(cloud_crdt_operation::timestamp::order(SortOrder::Asc))
			.include(cloud_crdt_include::include())
			.exec()
			.await?;

		ops.sort_by(|a, b| match a.timestamp().cmp(&b.timestamp()) {
			Ordering::Equal => a.instance().cmp(&b.instance()),
			o => o,
		});

		Ok(ops
			.into_iter()
			.take(args.count as usize)
			.map(|o| o.into_operation())
			.collect())
	}
}

impl OperationFactory for Manager {
	fn get_clock(&self) -> &HLC {
		&self.clock
	}

	fn get_instance(&self) -> Uuid {
		self.instance
	}
}

impl Deref for Manager {
	type Target = SharedState;

	fn deref(&self) -> &Self::Target {
		&self.shared
	}
}
