use std::{
	borrow::Cow,
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{
		atomic::{AtomicBool, Ordering},
		Arc,
	},
	time::{Duration, Instant},
};

use sd_p2p::{
	spaceblock::{BlockSize, Range, SpaceblockRequest, Transfer},
	spacetunnel::{RemoteIdentity, Tunnel},
	Event, Manager, ManagerError, ManagerStream, MetadataManager, PeerId,
};
use sd_prisma::prisma::file_path;
use serde::Serialize;
use specta::Type;
use tokio::{
	fs::File,
	io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader},
	sync::{broadcast, oneshot, Mutex},
	time::sleep,
};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::{
	library::Library,
	location::file_path_helper::{file_path_to_handle_p2p_serve_file, IsolatedFilePathData},
	node::config::{self, NodeConfig},
	p2p::{OperatingSystem, SPACEDRIVE_APP_ID},
	Node,
};

use super::{
	sync::{InstanceState, NetworkedLibraries, SyncMessage},
	Header, PairingManager, PairingStatus, PeerMetadata,
};

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
	ExpiredPeer {
		peer_id: PeerId,
	},
	ConnectedPeer {
		peer_id: PeerId,
	},
	DisconnectedPeer {
		peer_id: PeerId,
	},
	SpacedropRequest {
		id: Uuid,
		peer_id: PeerId,
		peer_name: String,
		file_name: String,
	},
	SpacedropProgress {
		id: Uuid,
		percent: u8,
	},
	SpacedropRejected {
		id: Uuid,
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
	spacedrop_cancelations: Arc<Mutex<HashMap<Uuid, Arc<AtomicBool>>>>,
	pub metadata_manager: Arc<MetadataManager<PeerMetadata>>,
	pub pairing: Arc<PairingManager>,
	node_config_manager: Arc<config::Manager>,
}

impl P2PManager {
	pub async fn new(
		node_config: Arc<config::Manager>,
	) -> Result<(Arc<P2PManager>, ManagerStream<PeerMetadata>), ManagerError> {
		let (config, keypair) = {
			let config = node_config.get().await;

			// TODO: The `vec![]` here is problematic but will be fixed with delayed `MetadataManager`
			(Self::config_to_metadata(&config, vec![]), config.keypair)
		};

		// TODO: Delay building this until the libraries are loaded
		let metadata_manager = MetadataManager::new(config);

		let (manager, stream) = sd_p2p::Manager::<PeerMetadata>::new(
			SPACEDRIVE_APP_ID,
			&keypair,
			metadata_manager.clone(),
		)
		.await?;

		info!(
			"Node '{}' is now online listening at addresses: {:?}",
			manager.peer_id(),
			manager.listen_addrs().await
		);

		// need to keep 'rx' around so that the channel isn't dropped
		let (tx, rx) = broadcast::channel(100);
		let pairing = PairingManager::new(manager.clone(), tx.clone(), metadata_manager.clone());

		Ok((
			Arc::new(Self {
				pairing,
				events: (tx, rx),
				manager,
				spacedrop_pairing_reqs: Default::default(),
				spacedrop_cancelations: Default::default(),
				metadata_manager,
				node_config_manager: node_config,
			}),
			stream,
		))
	}

	pub fn start(&self, mut stream: ManagerStream<PeerMetadata>, node: Arc<Node>) {
		tokio::spawn({
			let manager = self.manager.clone();
			let metadata_manager = self.metadata_manager.clone();
			let events = self.events.0.clone();
			let spacedrop_pairing_reqs = self.spacedrop_pairing_reqs.clone();
			let spacedrop_cancelations = self.spacedrop_cancelations.clone();

			let pairing = self.pairing.clone();

			async move {
				let mut shutdown = false;
				while let Some(event) = stream.next().await {
					match event {
						Event::PeerDiscovered(event) => {
							events
								.send(P2PEvent::DiscoveredPeer {
									peer_id: event.peer_id,
									metadata: event.metadata.clone(),
								})
								.map_err(|_| error!("Failed to send event to p2p event stream!"))
								.ok();

							node.nlm.peer_discovered(event).await;
						}
						Event::PeerExpired { id, .. } => {
							events
								.send(P2PEvent::ExpiredPeer { peer_id: id })
								.map_err(|_| error!("Failed to send event to p2p event stream!"))
								.ok();

							node.nlm.peer_expired(id).await;
						}
						Event::PeerConnected(event) => {
							events
								.send(P2PEvent::ConnectedPeer {
									peer_id: event.peer_id,
								})
								.map_err(|_| error!("Failed to send event to p2p event stream!"))
								.ok();

							node.nlm.peer_connected(event.peer_id).await;

							let manager = manager.clone();
							let nlm = node.nlm.clone();
							let instances = metadata_manager.get().instances;
							let node = node.clone();
							tokio::spawn(async move {
								if event.establisher {
									let mut stream = manager.stream(event.peer_id).await.unwrap();
									Self::resync(
										nlm.clone(),
										&mut stream,
										event.peer_id,
										instances,
									)
									.await;

									drop(stream);
								}

								Self::resync_part2(nlm, node, &event.peer_id).await;
							});
						}
						Event::PeerDisconnected(peer_id) => {
							events
								.send(P2PEvent::DisconnectedPeer { peer_id })
								.map_err(|_| error!("Failed to send event to p2p event stream!"))
								.ok();

							node.nlm.peer_disconnected(peer_id).await;
						}
						Event::PeerMessage(event) => {
							let events = events.clone();
							let metadata_manager = metadata_manager.clone();
							let spacedrop_pairing_reqs = spacedrop_pairing_reqs.clone();
							let pairing = pairing.clone();
							let spacedrop_cancelations = spacedrop_cancelations.clone();
							let node = node.clone();
							let manager = manager.clone();

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

										if events
											.send(P2PEvent::SpacedropRequest {
												id,
												peer_id: event.peer_id,
												peer_name: manager
													.get_discovered_peers()
													.await
													.into_iter()
													.find(|p| p.peer_id == event.peer_id)
													.map(|p| p.metadata.name)
													.unwrap_or_else(|| "Unknown".to_string()),
												file_name: req.name.clone(),
											})
											.is_err()
										{
											// No frontend's are active

											todo!("Outright reject Spacedrop");
										}

										tokio::select! {
											_ = sleep(SPACEDROP_TIMEOUT) => {
												info!("spacedrop({id}): timeout, rejecting!");

												stream.write_all(&[0]).await.unwrap();
												stream.flush().await.unwrap();
											}
											file_path = rx => {
												match file_path {
													Ok(Some(file_path)) => {
														info!("spacedrop({id}): accepted saving to '{:?}'", file_path);

														let cancelled = Arc::new(AtomicBool::new(false));
														spacedrop_cancelations
															.lock()
															.await
															.insert(id, cancelled.clone());

														stream.write_all(&[1]).await.unwrap();

														let f = File::create(file_path).await.unwrap();

														Transfer::new(&req, |percent| {
															events.send(P2PEvent::SpacedropProgress { id, percent }).ok();
														}, &cancelled).receive(&mut stream, f).await;

														info!("spacedrop({id}): complete");
													}
													Ok(None) => {
														info!("spacedrop({id}): rejected");

														stream.write_all(&[0]).await.unwrap();
														stream.flush().await.unwrap();
													}
													Err(_) => {
														info!("spacedrop({id}): error with Spacedrop pairing request receiver!");
													}
												}
											}
										};
									}
									Header::Pair => {
										pairing
											.responder(
												event.peer_id,
												stream,
												&node.libraries,
												node.clone(),
											)
											.await;
									}
									Header::Sync(library_id) => {
										let mut tunnel = Tunnel::responder(stream).await.unwrap();

										let msg =
											SyncMessage::from_stream(&mut tunnel).await.unwrap();

										let library =
											node.libraries.get_library(&library_id).await.unwrap();

										match msg {
											SyncMessage::NewOperations => {
												super::sync::responder(&mut tunnel, library).await;
											}
										};
									}
									Header::File {
										library_id,
										file_path_id,
										range,
									} => {
										if !node.files_over_p2p_flag.load(Ordering::Relaxed) {
											panic!("Files over P2P is disabled!");
										}

										// TODO: Tunnel and authentication
										// TODO: Use BufReader

										let library =
											node.libraries.get_library(&library_id).await.unwrap();

										let file_path = library
											.db
											.file_path()
											.find_unique(file_path::pub_id::equals(
												file_path_id.as_bytes().to_vec(),
											))
											.select(file_path_to_handle_p2p_serve_file::select())
											.exec()
											.await
											.unwrap()
											.unwrap();

										let location = file_path.location.as_ref().unwrap();
										let location_path = location.path.as_ref().unwrap();
										let path = Path::new(location_path).join(
											IsolatedFilePathData::try_from((
												location.id,
												&file_path,
											))
											.unwrap(),
										);

										debug!("Serving path '{:?}' over P2P", path);

										let file = File::open(&path).await.unwrap();

										let metadata = file.metadata().await.unwrap();
										let block_size = BlockSize::from_size(metadata.len());

										stream.write_all(&block_size.to_bytes()).await.unwrap();
										stream
											.write_all(&metadata.len().to_le_bytes())
											.await
											.unwrap();

										let file = BufReader::new(file);
										Transfer::new(
											&SpaceblockRequest {
												// TODO: Removing need for this field in this case
												name: "todo".to_string(),
												size: metadata.len(),
												block_size,
												range,
											},
											|percent| {
												debug!(
													"P2P loading file path '{}' - progress {}%",
													file_path_id, percent
												);
											},
											&Arc::new(AtomicBool::new(false)),
										)
										.send(&mut stream, file)
										.await;
									}
									Header::Connected(identities) => {
										Self::resync_handler(
											&node.nlm,
											&mut stream,
											event.peer_id,
											metadata_manager.get().instances,
											identities,
										)
										.await;
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
						_ => {}
					}
				}

				if !shutdown {
					error!(
						"Manager event stream closed! The core is unstable from this point forward!"
					);
				}
			}
		});
	}

	fn config_to_metadata(config: &NodeConfig, instances: Vec<RemoteIdentity>) -> PeerMetadata {
		PeerMetadata {
			name: config.name.clone(),
			operating_system: Some(OperatingSystem::get_os()),
			version: Some(env!("CARGO_PKG_VERSION").to_string()),
			email: config.p2p_email.clone(),
			img_url: config.p2p_img_url.clone(),
			instances,
		}
	}

	// TODO: Remove this & move to `NetworkedLibraryManager`??? or make it private?
	pub async fn update_metadata(&self, instances: Vec<RemoteIdentity>) {
		self.metadata_manager.update(Self::config_to_metadata(
			&self.node_config_manager.get().await,
			instances,
		));
	}

	pub async fn resync(
		nlm: Arc<NetworkedLibraries>,
		stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
		peer_id: PeerId,
		instances: Vec<RemoteIdentity>,
	) {
		// TODO: Make this encrypted using node to node auth so it can't be messed with in transport

		stream
			.write_all(&Header::Connected(instances).to_bytes())
			.await
			.unwrap();

		let Header::Connected(identities) = Header::from_stream(stream).await.unwrap() else {
			panic!("unreachable but error handling")
		};

		for identity in identities {
			nlm.peer_connected2(identity, peer_id).await;
		}
	}

	pub async fn resync_handler(
		nlm: &NetworkedLibraries,
		stream: &mut (impl AsyncRead + AsyncWrite + Unpin),
		peer_id: PeerId,
		local_identities: Vec<RemoteIdentity>,
		remote_identities: Vec<RemoteIdentity>,
	) {
		for identity in remote_identities {
			nlm.peer_connected2(identity, peer_id).await;
		}

		stream
			.write_all(&Header::Connected(local_identities).to_bytes())
			.await
			.unwrap();
	}

	// TODO: Using tunnel for security - Right now all sync events here are unencrypted
	pub async fn resync_part2(
		nlm: Arc<NetworkedLibraries>,
		node: Arc<Node>,
		connected_with_peer_id: &PeerId,
	) {
		for (library_id, data) in nlm.state().await {
			let mut library = None;

			for (_, data) in data.instances {
				let InstanceState::Connected(instance_peer_id) = data else {
					continue;
				};

				if instance_peer_id != *connected_with_peer_id {
					continue;
				};

				let library = match library.clone() {
					Some(library) => library,
					None => match node.libraries.get_library(&library_id).await {
						Some(new_library) => {
							library = Some(new_library.clone());

							new_library
						}
						None => continue,
					},
				};

				// Remember, originator creates a new stream internally so the handler for this doesn't have to do anything.
				super::sync::originator(library_id, &library.sync, &node.nlm, &node.p2p).await;
			}
		}
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

	pub async fn cancel_spacedrop(&self, id: Uuid) {
		if let Some(cancelled) = self.spacedrop_cancelations.lock().await.remove(&id) {
			cancelled.store(true, Ordering::Relaxed);
		}
	}

	pub fn subscribe(&self) -> broadcast::Receiver<P2PEvent> {
		self.events.0.subscribe()
	}

	pub async fn ping(&self) {
		self.manager.broadcast(Header::Ping.to_bytes()).await;
	}

	// TODO: Proper error handling
	pub async fn spacedrop(&self, peer_id: PeerId, path: PathBuf) -> Result<Option<Uuid>, ()> {
		let id = Uuid::new_v4();
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
			range: Range::Full,                               // range: None,
		});
		stream.write_all(&header.to_bytes()).await.map_err(|_| ())?;

		debug!("Waiting for Spacedrop to be accepted from peer '{peer_id}'");
		let mut buf = [0; 1];
		// TODO: Add timeout so the connection is dropped if they never response
		stream.read_exact(&mut buf).await.map_err(|_| ())?;
		if buf[0] != 1 {
			debug!("Spacedrop was rejected from peer '{peer_id}'");
			self.events.0.send(P2PEvent::SpacedropRejected { id }).ok();
			return Ok(None);
		}

		let cancelled = Arc::new(AtomicBool::new(false));
		self.spacedrop_cancelations
			.lock()
			.await
			.insert(id, cancelled.clone());

		debug!("Starting Spacedrop to peer '{peer_id}'");
		let i = Instant::now();

		let file = BufReader::new(file);
		Transfer::new(
			&match header {
				Header::Spacedrop(req) => req,
				_ => unreachable!(),
			},
			|percent| {
				self.events
					.0
					.send(P2PEvent::SpacedropProgress { id, percent })
					.ok();
			},
			&cancelled,
		)
		.send(&mut stream, file)
		.await;

		debug!(
			"Finished Spacedrop to peer '{peer_id}' after '{:?}",
			i.elapsed()
		);

		Ok(Some(id))
	}

	// DO NOT USE THIS WITHOUT `node.files_over_p2p_flag == true`
	// TODO: Error handling
	pub async fn request_file(
		&self,
		peer_id: PeerId,
		library: &Library,
		file_path_id: Uuid,
		range: Range,
		output: impl AsyncWrite + Unpin,
	) {
		let mut stream = self.manager.stream(peer_id).await.unwrap(); // TODO: handle providing incorrect peer id

		// TODO: Tunnel for encryption + authentication

		stream
			.write_all(
				&Header::File {
					library_id: library.id,
					file_path_id,
					range: range.clone(),
				}
				.to_bytes(),
			)
			.await
			.unwrap();

		let block_size = BlockSize::from_stream(&mut stream).await.unwrap();
		let size = stream.read_u64_le().await.unwrap();

		Transfer::new(
			&SpaceblockRequest {
				// TODO: Removing need for this field in this case
				name: "todo".to_string(),
				// TODO: Maybe removing need for `size` from this side
				size,
				block_size,
				range,
			},
			|percent| {
				debug!(
					"P2P receiving file path '{}' - progress {}%",
					file_path_id, percent
				);
			},
			&Arc::new(AtomicBool::new(false)),
		)
		.receive(&mut stream, output)
		.await;
	}

	pub async fn shutdown(&self) {
		self.manager.shutdown().await;
	}
}
