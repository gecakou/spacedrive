use std::{
	borrow::Cow,
	collections::HashMap,
	path::PathBuf,
	sync::Arc,
	time::{Duration, Instant},
};

use futures::Stream;
use sd_p2p::{
	spaceblock::{BlockSize, SpaceblockRequest, Transfer},
	spacetunnel::{Identity, Tunnel},
	Event, Manager, ManagerError, MetadataManager, PeerId,
};
use sd_sync::CRDTOperation;
use serde::Serialize;
use specta::Type;
use tokio::{
	fs::File,
	io::{AsyncReadExt, AsyncWriteExt, BufReader},
	sync::{broadcast, oneshot, Mutex},
	time::sleep,
};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
	library::{LibraryManager, SubscriberEvent},
	node::{NodeConfig, NodeConfigManager},
	p2p::{OperatingSystem, SPACEDRIVE_APP_ID},
	sync::SyncMessage,
};

use super::{Header, PairingManager, PairingStatus, PeerMetadata};

/// The amount of time to wait for a Spacedrop request to be accepted or rejected before it's automatically rejected
const SPACEDROP_TIMEOUT: Duration = Duration::from_secs(60);

/// TODO: P2P event for the frontend
#[derive(Debug, Clone, Type, Serialize)]
#[serde(tag = "type")]
pub enum P2PEvent {
	DiscoveredPeer {
		peer_id: PeerId,
		metadata: PeerMetadata,
	},
	SpacedropRequest {
		id: Uuid,
		peer_id: PeerId,
		name: String,
	},
	// Pairing was reuqest has come in.
	// This will fire on the responder only.
	PairingRequest {
		id: u16,
		name: String,
		os: OperatingSystem,
	},
	PairingProgress {
		id: u16,
		status: PairingStatus,
	}, // TODO: Expire peer + connection/disconnect
}

pub struct P2PManager {
	pub events: (broadcast::Sender<P2PEvent>, broadcast::Receiver<P2PEvent>),
	pub manager: Arc<Manager<PeerMetadata>>,
	spacedrop_pairing_reqs: Arc<Mutex<HashMap<Uuid, oneshot::Sender<Option<String>>>>>,
	pub metadata_manager: Arc<MetadataManager<PeerMetadata>>,
	pub spacedrop_progress: Arc<Mutex<HashMap<Uuid, broadcast::Sender<u8>>>>,
	pub pairing: Arc<PairingManager>,
	library_manager: Arc<LibraryManager>,
}

impl P2PManager {
	pub async fn new(
		node_config: Arc<NodeConfigManager>,
		library_manager: Arc<LibraryManager>,
	) -> Result<Arc<Self>, ManagerError> {
		let (config, keypair) = {
			let config = node_config.get().await;
			(
				Self::config_to_metadata(&config, &library_manager).await,
				config.keypair,
			)
		};

		let metadata_manager = MetadataManager::new(config);

		let (manager, mut stream) =
			Manager::new(SPACEDRIVE_APP_ID, &keypair, metadata_manager.clone()).await?;

		info!(
			"Node '{}' is now online listening at addresses: {:?}",
			manager.peer_id(),
			manager.listen_addrs().await
		);

		// need to keep 'rx' around so that the channel isn't dropped
		let (tx, rx) = broadcast::channel(100);

		let spacedrop_pairing_reqs = Arc::new(Mutex::new(HashMap::new()));
		let spacedrop_progress = Arc::new(Mutex::new(HashMap::new()));

		let pairing = PairingManager::new(manager.clone(), tx.clone(), library_manager.clone());
		tokio::spawn({
			let events = tx.clone();
			let spacedrop_pairing_reqs = spacedrop_pairing_reqs.clone();
			let spacedrop_progress = spacedrop_progress.clone();
			let library_manager = library_manager.clone();
			let pairing = pairing.clone();

			async move {
				let mut shutdown = false;
				while let Some(event) = stream.next().await {
					match event {
						Event::PeerDiscovered(event) => {
							debug!(
								"Discovered peer by id '{}' with address '{:?}' and metadata: {:?}",
								event.peer_id, event.addresses, event.metadata
							);

							events
								.send(P2PEvent::DiscoveredPeer {
									peer_id: event.peer_id,
									metadata: event.metadata.clone(),
								})
								.map_err(|_| error!("Failed to send event to p2p event stream!"))
								.ok();

							// TODO: Don't just connect to everyone when we find them. We should only do it if we know them.
							// TODO(Spacedrop): Disable Spacedrop for now
							// event.dial().await;
						}
						Event::PeerMessage(event) => {
							let events = events.clone();
							let spacedrop_pairing_reqs = spacedrop_pairing_reqs.clone();
							let spacedrop_progress = spacedrop_progress.clone();
							let library_manager = library_manager.clone();
							let pairing = pairing.clone();

							tokio::spawn(async move {
								let mut stream = event.stream;
								let header = Header::from_stream(&mut stream).await.unwrap();

								match header {
									Header::Ping => {
										debug!("Received ping from peer '{}'", event.peer_id);
									}
									Header::Spacedrop(req) => {
										let id = Uuid::new_v4();
										let (tx, rx) = oneshot::channel();

										info!("spacedrop({id}): received from peer '{}' for file '{}' with file length '{}'", event.peer_id, req.name, req.size);

										spacedrop_pairing_reqs.lock().await.insert(id, tx);

										let (process_tx, _) = broadcast::channel(100);
										spacedrop_progress
											.lock()
											.await
											.insert(id, process_tx.clone());

										if events
											.send(P2PEvent::SpacedropRequest {
												id,
												peer_id: event.peer_id,
												name: req.name.clone(),
											})
											.is_err()
										{
											// No frontend's are active

											todo!("Outright reject Spacedrop");
										}

										tokio::select! {
											_ = sleep(SPACEDROP_TIMEOUT) => {
												info!("spacedrop({id}): timeout, rejecting!");
											}
											file_path = rx => {
												match file_path {
													Ok(Some(file_path)) => {
														info!("spacedrop({id}): accepted saving to '{:?}'", file_path);

														stream.write_all(&[1]).await.unwrap();

														let f = File::create(file_path).await.unwrap();

														Transfer::new(&req, |percent| {
															process_tx.send(percent).ok();
														}).receive(&mut stream, f).await;

														info!("spacedrop({id}): complete");
													}
													Ok(None) => {
														info!("spacedrop({id}): rejected");
													}
													Err(_) => {
														info!("spacedrop({id}): error with Spacedrop pairing request receiver!");
													}
												}
											}
										};
									}
									Header::Pair => {
										pairing.responder(event.peer_id, stream).await;
									}
									Header::Sync(library_id) => {
										let mut stream = Tunnel::from_stream(stream).await.unwrap();

										let mut len = [0; 4];
										stream.read_exact(&mut len).await.unwrap();
										let len = u32::from_le_bytes(len);

										let mut buf = vec![0; len as usize]; // TODO: Designed for easily being able to be DOS the current Node
										stream.read_exact(&mut buf).await.unwrap();

										let mut buf: &[u8] = &buf;
										let operations: Vec<CRDTOperation> =
											rmp_serde::from_read(&mut buf).unwrap();

										debug!("ingesting sync events for library '{library_id}': {operations:?}");

										let Some(library) = library_manager.get_library(library_id).await else {
											warn!("error ingesting sync messages. no library by id '{library_id}' found!");
											return;
										};

										for op in operations {
											library.sync.ingest_op(op).await.unwrap_or_else(
												|err| {
													error!(
														"error ingesting operation for library '{}': {err:?}",
														library.id
													);
												},
											);
										}
									}
								}
							});
						}
						Event::PeerBroadcast(_event) => {
							// todo!();
						}
						Event::Shutdown => {
							shutdown = true;
							break;
						}
						_ => debug!("event: {:?}", event),
					}
				}

				if !shutdown {
					error!(
						"Manager event stream closed! The core is unstable from this point forward!"
					);
				}
			}
		});

		// TODO: proper shutdown
		// https://docs.rs/ctrlc/latest/ctrlc/
		// https://docs.rs/system_shutdown/latest/system_shutdown/

		let this = Arc::new(Self {
			pairing,
			events: (tx, rx),
			manager,
			spacedrop_pairing_reqs,
			metadata_manager,
			spacedrop_progress,
			library_manager: library_manager.clone(),
		});

		library_manager
			.subscribe({
				let this = this.clone();
				move |event| match event {
					SubscriberEvent::Load(library_id, library_identity, mut sync_rx) => {
						let this = this.clone();
						tokio::spawn(async move {
							while let Ok(op) = sync_rx.recv().await {
								let SyncMessage::Created(op) = op else { continue; };

								this.broadcast_sync_events(library_id, &library_identity, vec![op])
									.await;
							}
						});
					}
				}
			})
			.await;

		// TODO: Probs remove this once connection timeout/keepalive are working correctly
		tokio::spawn({
			let this = this.clone();
			async move {
				loop {
					tokio::time::sleep(std::time::Duration::from_secs(5)).await;
					this.ping().await;
				}
			}
		});

		Ok(this)
	}

	async fn config_to_metadata(
		config: &NodeConfig,
		library_manager: &LibraryManager,
	) -> PeerMetadata {
		PeerMetadata {
			name: config.name.clone(),
			operating_system: Some(OperatingSystem::get_os()),
			version: Some(env!("CARGO_PKG_VERSION").to_string()),
			email: config.p2p_email.clone(),
			img_url: config.p2p_img_url.clone(),
			instances: library_manager
				.get_all_instances()
				.await
				.into_iter()
				.filter_map(|i| {
					Identity::from_bytes(&i.identity)
						.map(|i| hex::encode(i.public_key().to_bytes()))
						.ok()
				})
				.collect(),
		}
	}

	#[allow(unused)] // TODO: Should probs be using this
	pub async fn update_metadata(&self, node_config_manager: &NodeConfigManager) {
		self.metadata_manager.update(
			Self::config_to_metadata(&node_config_manager.get().await, &self.library_manager).await,
		);
	}

	pub async fn accept_spacedrop(&self, id: Uuid, path: String) {
		if let Some(chan) = self.spacedrop_pairing_reqs.lock().await.remove(&id) {
			chan.send(Some(path)).unwrap();
		}
	}

	pub async fn reject_spacedrop(&self, id: Uuid) {
		if let Some(chan) = self.spacedrop_pairing_reqs.lock().await.remove(&id) {
			chan.send(None).unwrap();
		}
	}

	pub fn subscribe(&self) -> broadcast::Receiver<P2PEvent> {
		self.events.0.subscribe()
	}

	pub async fn broadcast_sync_events(
		&self,
		library_id: Uuid,
		_identity: &Identity,
		event: Vec<CRDTOperation>,
	) {
		let mut buf = match rmp_serde::to_vec_named(&event) {
			Ok(buf) => buf,
			Err(e) => {
				error!("Failed to serialize sync event: {:?}", e);
				return;
			}
		};
		let mut head_buf = Header::Sync(library_id).to_bytes(); // Max Sync payload is like 4GB
		head_buf.extend_from_slice(&(buf.len() as u32).to_le_bytes());
		head_buf.append(&mut buf);

		// TODO: Determine which clients we share that library with

		// TODO: Establish a connection to them

		let _library = self.library_manager.get_library(library_id).await.unwrap();

		todo!();

		// TODO: probs cache this query in memory cause this is gonna be stupid frequent
		// let target_nodes = library
		// 	.db
		// 	.node()
		// 	.find_many(vec![])
		// 	.exec()
		// 	.await
		// 	.unwrap()
		// 	.into_iter()
		// 	.map(|n| {
		// 		PeerId::from_str(&n.node_peer_id.expect("Node was missing 'node_peer_id'!"))
		// 			.unwrap()
		// 	})
		// 	.collect::<Vec<_>>();

		// info!(
		// 	"Sending sync messages for library '{}' to nodes with peer id's '{:?}'",
		// 	library_id, target_nodes
		// );

		// // TODO: Do in parallel
		// for peer_id in target_nodes {
		// 	let stream = self.manager.stream(peer_id).await.map_err(|_| ()).unwrap(); // TODO: handle providing incorrect peer id

		// 	let mut tunnel = Tunnel::from_stream(stream).await.unwrap();

		// 	tunnel.write_all(&head_buf).await.unwrap();
		// }
	}

	pub async fn ping(&self) {
		self.manager.broadcast(Header::Ping.to_bytes()).await;
	}

	// TODO: Proper error handling
	pub async fn big_bad_spacedrop(
		&self,
		peer_id: PeerId,
		path: PathBuf,
	) -> Result<Option<Uuid>, ()> {
		let id = Uuid::new_v4();
		let (tx, _) = broadcast::channel(25);
		let mut stream = self.manager.stream(peer_id).await.map_err(|_| ())?; // TODO: handle providing incorrect peer id

		let file = File::open(&path).await.map_err(|_| ())?;
		let metadata = file.metadata().await.map_err(|_| ())?;

		let header = Header::Spacedrop(SpaceblockRequest {
			name: path
				.file_name()
				.map(|v| v.to_string_lossy())
				.unwrap_or(Cow::Borrowed(""))
				.to_string(),
			size: metadata.len(),
			block_size: BlockSize::from_size(metadata.len()), // TODO: This should be dynamic
		});
		stream.write_all(&header.to_bytes()).await.map_err(|_| ())?;

		debug!("Waiting for Spacedrop to be accepted from peer '{peer_id}'");
		let mut buf = [0; 1];
		// TODO: Add timeout so the connection is dropped if they never response
		stream.read_exact(&mut buf).await.map_err(|_| ())?;
		if buf[0] != 1 {
			debug!("Spacedrop was rejected from peer '{peer_id}'");
			return Ok(None);
		}

		debug!("Starting Spacedrop to peer '{peer_id}'");
		let i = Instant::now();

		let file = BufReader::new(file);
		self.spacedrop_progress.lock().await.insert(id, tx.clone());
		Transfer::new(
			&match header {
				Header::Spacedrop(req) => req,
				_ => unreachable!(),
			},
			|percent| {
				tx.send(percent).ok();
			},
		)
		.send(&mut stream, file)
		.await;

		debug!(
			"Finished Spacedrop to peer '{peer_id}' after '{:?}",
			i.elapsed()
		);

		Ok(Some(id))
	}

	pub async fn spacedrop_progress(&self, id: Uuid) -> Option<impl Stream<Item = u8>> {
		self.spacedrop_progress.lock().await.get(&id).map(|v| {
			let mut v = v.subscribe();
			async_stream::stream! {
				while let Ok(item) = v.recv().await {
					yield item;
				}
			}
		})
	}

	pub async fn shutdown(&self) {
		self.manager.shutdown().await;
	}
}
