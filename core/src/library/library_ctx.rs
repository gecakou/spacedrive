use crate::{job::DynJob, node::NodeConfigManager, prisma::PrismaClient, CoreEvent, NodeContext};
use std::sync::Arc;
use uuid::Uuid;

use super::LibraryConfig;

/// LibraryContext holds context for a library which can be passed around the application.
#[derive(Clone)]
pub struct LibraryContext {
	/// id holds the ID of the current library.
	pub id: Uuid,
	/// config holds the configuration of the current library.
	pub config: LibraryConfig,
	/// db holds the database client for the current library.
	pub db: Arc<PrismaClient>,
	/// node_local_id holds the local ID of the node which is running the library.
	pub node_local_id: i32,
	/// node_context holds the node context for the node which this library is running on.
	pub(super) node_context: NodeContext,
}

impl LibraryContext {
	pub(crate) async fn spawn_job(&self, job: Box<dyn DynJob>) {
		self.node_context.jobs.clone().ingest(self, job).await;
	}

	pub(crate) async fn queue_job(&self, job: Box<dyn DynJob>) {
		self.node_context.jobs.ingest_queue(self, job).await;
	}

	pub(crate) async fn emit(&self, event: CoreEvent) {
		self.node_context
			.event_sender
			.send(event)
			.await
			.unwrap_or_else(|e| {
				println!("Failed to emit event. {:?}", e);
			});
	}

	pub(crate) fn config(&self) -> Arc<NodeConfigManager> {
		self.node_context.config.clone()
	}
}
