use std::{
	collections::HashMap,
	sync::{
		atomic::{AtomicU16, Ordering},
		Arc, RwLock,
	},
};

use chrono::Utc;
use futures::channel::oneshot;
use sd_p2p::{spacetunnel::Identity, Manager, PeerId};

use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::{
	io::{AsyncRead, AsyncWrite, AsyncWriteExt},
	sync::broadcast,
};
use tracing::{debug, info};
use uuid::Uuid;

mod initial_sync;
mod proto;

pub use initial_sync::*;
use proto::*;

use crate::{
	library::{LibraryManager, LibraryName},
	node::{NodeConfig, Platform},
	p2p::Header,
};

use super::{P2PEvent, PeerMetadata};

pub struct PairingManager {
	id: AtomicU16,
	events_tx: broadcast::Sender<P2PEvent>,
	pairing_response: RwLock<HashMap<u16, oneshot::Sender<PairingDecision>>>,
	manager: Arc<Manager<PeerMetadata>>,
	library_manager: Arc<LibraryManager>,
}

impl PairingManager {
	pub fn new(
		manager: Arc<Manager<PeerMetadata>>,
		events_tx: broadcast::Sender<P2PEvent>,
		library_manager: Arc<LibraryManager>,
	) -> Arc<Self> {
		Arc::new(Self {
			id: AtomicU16::new(0),
			events_tx,
			pairing_response: RwLock::new(HashMap::new()),
			manager,
			library_manager,
		})
	}

	fn emit_progress(&self, id: u16, status: PairingStatus) {
		self.events_tx
			.send(P2PEvent::PairingProgress { id, status })
			.ok();
	}

	pub fn decision(&self, id: u16, decision: PairingDecision) {
		if let Some(tx) = self.pairing_response.write().unwrap().remove(&id) {
			tx.send(decision).ok();
		}
	}

	// TODO: Error handling

	pub async fn originator(self: Arc<Self>, peer_id: PeerId, node_config: NodeConfig) -> u16 {
		// TODO: Timeout for max number of pairings in a time period

		let pairing_id = self.id.fetch_add(1, Ordering::SeqCst);
		self.emit_progress(pairing_id, PairingStatus::EstablishingConnection);

		info!("Beginning pairing '{pairing_id}' as originator to remote peer '{peer_id}'");

		tokio::spawn(async move {
			let mut stream = self.manager.stream(peer_id).await.unwrap();
			stream.write_all(&Header::Pair.to_bytes()).await.unwrap();

			// TODO: Ensure both clients are on a compatible version cause Prisma model changes will cause issues

			// 1. Create new instance for originator and send it to the responder
			self.emit_progress(pairing_id, PairingStatus::PairingRequested);
			let now = Utc::now();
			let req = PairingRequest(Instance {
				id: Uuid::new_v4(),
				identity: Identity::new(), // TODO: Public key only
				node_id: node_config.id,
				node_name: node_config.name.clone(),
				node_platform: Platform::current(),
				last_seen: now,
				date_created: now,
			});
			stream.write_all(&req.to_bytes()).await.unwrap();

			// 2.
			match PairingResponse::from_stream(&mut stream).await.unwrap() {
				PairingResponse::Accepted {
					library_id,
					library_name,
					library_description,
					instances,
				} => {
					info!("Pairing '{pairing_id}' accepted by remote into library '{library_id}'");
					// TODO: Log all instances and library info
					self.emit_progress(
						pairing_id,
						PairingStatus::PairingInProgress {
							library_name: library_name.clone(),
							library_description: library_description.clone(),
						},
					);

					// TODO: Future - Library in pairing state
					// TODO: Create library

					if self
						.library_manager
						.get_all_libraries()
						.await
						.into_iter()
						.find(|i| i.id == library_id)
						.is_some()
					{
						self.emit_progress(pairing_id, PairingStatus::LibraryAlreadyExists);

						// TODO: Properly handle this at a protocol level so the error is on both sides

						return;
					}

					let library_config = self
						.library_manager
						.create_with_uuid(
							library_id,
							LibraryName::new(library_name).unwrap(),
							library_description,
							node_config,
							false, // We will sync everything which will conflict with the seeded stuff
						)
						.await
						.unwrap();
					let library = self
						.library_manager
						.get_library(library_config.uuid)
						.await
						.unwrap();

					library
						.db
						.instance()
						.create_many(instances.into_iter().map(|i| i.into()).collect())
						.exec()
						.await
						.unwrap();

					// 3.
					// TODO: Either rollback or update library out of pairing state

					// TODO: This should timeout if taking too long so it can't be used as a DOS style thing???
					let mut total = 0;
					let mut synced = 0;
					loop {
						match SyncData::from_stream(&mut stream).await.unwrap() {
							SyncData::Data { total_models, data } => {
								if let Some(total_models) = total_models {
									total = total_models;
								}
								synced += data.len();

								data.insert(&library.db).await.unwrap();

								// Prevent divide by zero
								if total != 0 {
									self.emit_progress(
										pairing_id,
										PairingStatus::InitialSyncProgress(
											((synced as f32 / total as f32) * 100.0) as u8,
										),
									);
								}
							}
							SyncData::Finished => break,
						}
					}

					// TODO: Done message to frontend
					self.emit_progress(pairing_id, PairingStatus::PairingComplete(library_id));
					stream.flush().await.unwrap();
				}
				PairingResponse::Rejected => {
					info!("Pairing '{pairing_id}' rejected by remote");
					self.emit_progress(pairing_id, PairingStatus::PairingRejected);
				}
			}
		});

		pairing_id
	}

	pub async fn responder(
		self: Arc<Self>,
		peer_id: PeerId,
		mut stream: impl AsyncRead + AsyncWrite + Unpin,
	) {
		let pairing_id = self.id.fetch_add(1, Ordering::SeqCst);
		self.emit_progress(pairing_id, PairingStatus::EstablishingConnection);

		info!("Beginning pairing '{pairing_id}' as responder to remote peer '{peer_id}'");

		// let inner = || async move {
		let remote_instance = PairingRequest::from_stream(&mut stream).await.unwrap().0;
		self.emit_progress(pairing_id, PairingStatus::PairingDecisionRequest);
		self.events_tx
			.send(P2PEvent::PairingRequest {
				id: pairing_id,
				name: remote_instance.node_name,
				os: remote_instance.node_platform.into(),
			})
			.ok();

		// Prompt the user and wait
		// TODO: After 1 minute remove channel from map and assume it was rejected
		let (tx, rx) = oneshot::channel();
		self.pairing_response
			.write()
			.unwrap()
			.insert(pairing_id, tx);
		let PairingDecision::Accept(library_id) = rx.await.unwrap() else {
    			info!("The user rejected pairing '{pairing_id}'!");
    			// self.emit_progress(pairing_id, PairingStatus::PairingRejected); // TODO: Event to remove from frontend index
    			stream.write_all(&PairingResponse::Rejected.to_bytes()).await.unwrap();
    			return;
    		};
		info!("The user accepted pairing '{pairing_id}' for library '{library_id}'!");

		let library = self.library_manager.get_library(library_id).await.unwrap();
		stream
			.write_all(
				&PairingResponse::Accepted {
					library_id: library.id,
					library_name: library.config.name.into(),
					library_description: library.config.description.clone(),
					instances: library
						.db
						.instance()
						.find_many(vec![])
						.exec()
						.await
						.unwrap()
						.into_iter()
						.map(|i| Instance {
							id: Uuid::from_slice(&i.pub_id).unwrap(),
							// TODO: If `i.identity` contains a public/private keypair replace it with the public key
							identity: Identity::from_bytes(&i.identity).unwrap(),
							node_id: Uuid::from_slice(&i.node_id).unwrap(),
							node_name: i.node_name,
							node_platform: Platform::try_from(i.node_platform as u8)
								.unwrap_or(Platform::Unknown),
							last_seen: i.last_seen.into(),
							date_created: i.date_created.into(),
						})
						.collect(),
				}
				.to_bytes(),
			)
			.await
			.unwrap();

		// TODO: Pairing confirmation + rollback

		let total = ModelData::total_count(&library.db).await.unwrap();
		let mut synced = 0;
		info!("Starting sync of {} rows", total);

		let mut cursor = ModelSyncCursor::new();
		while let Some(data) = cursor.next(&library.db).await {
			let data = data.unwrap();
			let total_models = match synced {
				0 => Some(total),
				_ => None,
			};
			synced += data.len();
			self.emit_progress(
				pairing_id,
				PairingStatus::InitialSyncProgress((synced as f32 / total as f32 * 100.0) as u8), // SAFETY: It's a percentage
			);
			debug!(
				"Initial library sync cursor={:?} items={}",
				cursor,
				data.len()
			);

			stream
				.write_all(&SyncData::Data { total_models, data }.to_bytes().unwrap())
				.await
				.unwrap();
		}

		stream
			.write_all(&SyncData::Finished.to_bytes().unwrap())
			.await
			.unwrap();

		self.emit_progress(pairing_id, PairingStatus::PairingComplete(library_id));
		stream.flush().await.unwrap();
	}
}

#[derive(Debug, Type, Serialize, Deserialize)]
#[serde(tag = "decision", content = "libraryId", rename_all = "camelCase")]
pub enum PairingDecision {
	Accept(Uuid),
	Reject,
}

#[derive(Debug, Hash, Clone, Serialize, Type)]
#[serde(tag = "type", content = "data")]
pub enum PairingStatus {
	EstablishingConnection,
	PairingRequested,
	LibraryAlreadyExists,
	PairingDecisionRequest,
	PairingInProgress {
		library_name: String,
		library_description: Option<String>,
	},
	InitialSyncProgress(u8),
	PairingComplete(Uuid),
	PairingRejected,
}

// TODO: Unit tests
