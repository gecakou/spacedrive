use std::{
	collections::{HashMap, HashSet},
	net::{Ipv4Addr, Ipv6Addr, SocketAddr},
	str::FromStr,
	sync::{atomic::AtomicBool, Arc, Mutex, PoisonError, RwLock},
	time::Duration,
};

use flume::{bounded, Receiver, Sender};
use libp2p::{
	autonat, dcutr,
	futures::{AsyncReadExt, AsyncWriteExt, StreamExt},
	multiaddr::Protocol,
	noise, relay,
	swarm::{NetworkBehaviour, SwarmEvent},
	yamux, Multiaddr, PeerId, Stream, StreamProtocol, Swarm, SwarmBuilder,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{
	net::TcpListener,
	sync::{mpsc, oneshot},
	time::timeout,
};
use tokio_util::compat::FuturesAsyncReadCompatExt;
use tracing::{debug, error, warn};
use uuid::Uuid;

use super::{
	handle::QuicHandle,
	utils::{
		identity_to_libp2p_keypair, remote_identity_to_libp2p_peerid, socketaddr_to_quic_multiaddr,
	},
};
use crate::{
	identity::REMOTE_IDENTITY_LEN, ConnectionRequest, HookEvent, ListenerId,
	PeerConnectionCandidate, RemoteIdentity, UnicastStream, P2P,
};

const PROTOCOL: StreamProtocol = StreamProtocol::new("/sdp2p/1");

/// [libp2p::PeerId] for debugging purposes only.
#[derive(Debug)]
#[allow(dead_code)]
pub struct Libp2pPeerId(libp2p::PeerId);

#[derive(Debug)]
enum InternalEvent {
	RegisterListener {
		id: ListenerId,
		ipv4: bool,
		addr: SocketAddr,
		result: oneshot::Sender<Result<(), String>>,
	},
	UnregisterListener {
		id: ListenerId,
		ipv4: bool,
		result: oneshot::Sender<Result<(), String>>,
	},
	RegisterRelays {
		relays: Vec<RelayServerEntry>,
		result: oneshot::Sender<Result<(), String>>,
	},
	RegisterPeerAddr {
		addrs: HashSet<SocketAddr>,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayServerEntry {
	id: Uuid,
	peer_id: String,
	addrs: Vec<SocketAddr>,
}

#[derive(NetworkBehaviour)]
struct MyBehaviour {
	stream: libp2p_stream::Behaviour,
	// TODO: Can this be optional?
	relay: relay::client::Behaviour,
	// TODO: Can this be optional?
	autonat: autonat::Behaviour,
	// TODO: Can this be optional?
	dcutr: dcutr::Behaviour,
}

#[derive(Debug, Error)]
pub enum QuicTransportError {
	#[error("Failed to modify the SwarmBuilder: {0}")]
	SwarmBuilderCreation(String),
	#[error("Internal response channel closed: {0}")]
	SendChannelClosed(String),
	#[error("Internal response channel closed: {0}")]
	ReceiveChannelClosed(#[from] oneshot::error::RecvError),
	#[error("Failed internal event: {0}")]
	InternalEvent(String),
	#[error("Failed to create the Listener: {0}")]
	ListenerSetup(std::io::Error),
}

/// Transport using Quic to establish a connection between peers.
/// This uses `libp2p` internally.
#[derive(Debug)]
pub struct QuicTransport {
	id: ListenerId,
	p2p: Arc<P2P>,
	internal_tx: Sender<InternalEvent>,
	relay_config: Mutex<Vec<RelayServerEntry>>,
	handle: Arc<QuicHandle>,
}

impl QuicTransport {
	/// Spawn the `QuicTransport` and register it with the P2P system.
	/// Be aware spawning this does nothing unless you call `Self::set_ipv4_enabled`/`Self::set_ipv6_enabled` to enable the listeners.
	pub fn spawn(p2p: Arc<P2P>) -> Result<(Self, Libp2pPeerId), QuicTransportError> {
		let keypair = identity_to_libp2p_keypair(p2p.identity());
		let libp2p_peer_id = Libp2pPeerId(keypair.public().to_peer_id());

		let (tx, rx) = bounded(15);
		let (internal_tx, internal_rx) = bounded(15);
		let (connect_tx, connect_rx) = mpsc::channel(15);
		let id = p2p.register_listener("libp2p-quic", tx, move |listener_id, peer, _addrs| {
			// TODO: I don't love this always being registered. Really it should only show up if the other device is online (do a ping-type thing)???
			peer.clone()
				.listener_available(listener_id, connect_tx.clone());
		});

		let swarm = SwarmBuilder::with_existing_identity(keypair)
			.with_tokio()
			.with_quic()
			.with_relay_client(noise::Config::new, yamux::Config::default)
			.map_err(|err| QuicTransportError::SwarmBuilderCreation(err.to_string()))?
			.with_behaviour(|keypair, relay_behaviour| MyBehaviour {
				stream: libp2p_stream::Behaviour::new(),
				relay: relay_behaviour,
				autonat: autonat::Behaviour::new(keypair.public().to_peer_id(), Default::default()),
				dcutr: dcutr::Behaviour::new(keypair.public().to_peer_id()),
			})
			.map_err(|err| QuicTransportError::SwarmBuilderCreation(err.to_string()))?
			.with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
			.build();

		let handle = Arc::new(QuicHandle {
			shutdown: Default::default(),
			p2p: p2p.clone(),
			hook_id: id.into(),
			nodes: Default::default(),
			enabled: AtomicBool::new(true),
			connected_via_relay: Default::default(),
		});

		tokio::spawn(start(
			p2p.clone(),
			id,
			swarm,
			rx,
			internal_rx,
			connect_rx,
			handle.clone(),
		));

		Ok((
			Self {
				id,
				p2p,
				internal_tx,
				relay_config: Mutex::new(Vec::new()),
				handle,
			},
			libp2p_peer_id,
		))
	}

	/// Configure the relay servers to use.
	/// This method will replace any existing relay servers.
	pub async fn set_relay_config(
		&self,
		relays: Vec<RelayServerEntry>,
	) -> Result<(), QuicTransportError> {
		let (tx, rx) = oneshot::channel();
		let event = InternalEvent::RegisterRelays {
			relays: relays.clone(),
			result: tx,
		};

		self.internal_tx
			.send(event)
			.map_err(|e| QuicTransportError::SendChannelClosed(e.to_string()))?;

		let result = rx
			.await
			.map_err(QuicTransportError::ReceiveChannelClosed)
			.and_then(|r| r.map_err(QuicTransportError::InternalEvent));

		if result.is_ok() {
			*self
				.relay_config
				.lock()
				.unwrap_or_else(PoisonError::into_inner) = relays;
		}

		result
	}

	pub fn get_relay_config(&self) -> Vec<RelayServerEntry> {
		self.relay_config
			.lock()
			.unwrap_or_else(PoisonError::into_inner)
			.clone()
	}

	pub fn set_manual_peer_addrs(&self, addrs: HashSet<SocketAddr>) {
		self.internal_tx
			.send(InternalEvent::RegisterPeerAddr { addrs })
			.ok();
	}

	// `None` on the port means disabled. Use `0` for random port.
	pub async fn set_ipv4_enabled(&self, port: Option<u16>) -> Result<(), QuicTransportError> {
		self.setup_listener(
			port.map(|p| SocketAddr::from((Ipv4Addr::UNSPECIFIED, p))),
			true,
		)
		.await
	}

	pub async fn set_ipv6_enabled(&self, port: Option<u16>) -> Result<(), QuicTransportError> {
		self.setup_listener(
			port.map(|p| SocketAddr::from((Ipv6Addr::UNSPECIFIED, p))),
			false,
		)
		.await
	}

	async fn setup_listener(
		&self,
		addr: Option<SocketAddr>,
		ipv4: bool,
	) -> Result<(), QuicTransportError> {
		let (tx, rx) = oneshot::channel();
		let event = if let Some(mut addr) = addr {
			if addr.port() == 0 {
				addr.set_port(
					TcpListener::bind(addr)
						.await
						.map_err(QuicTransportError::ListenerSetup)?
						.local_addr()
						.map_err(QuicTransportError::ListenerSetup)?
						.port(),
				);
			}

			InternalEvent::RegisterListener {
				id: self.id,
				ipv4,
				addr,
				result: tx,
			}
		} else {
			InternalEvent::UnregisterListener {
				id: self.id,
				ipv4,
				result: tx,
			}
		};

		self.internal_tx
			.send(event)
			.map_err(|e| QuicTransportError::SendChannelClosed(e.to_string()))?;

		rx.await
			.map_err(QuicTransportError::ReceiveChannelClosed)
			.and_then(|r| r.map_err(QuicTransportError::InternalEvent))
	}

	pub fn handle(&self) -> Arc<QuicHandle> {
		self.handle.clone()
	}

	pub async fn shutdown(self) {
		self.p2p.unregister_hook(self.id.into()).await;
	}
}

async fn start(
	p2p: Arc<P2P>,
	id: ListenerId,
	mut swarm: Swarm<MyBehaviour>,
	rx: Receiver<HookEvent>,
	internal_rx: Receiver<InternalEvent>,
	mut connect_rx: mpsc::Receiver<ConnectionRequest>,
	handle: Arc<QuicHandle>,
) {
	let mut ipv4_listener = None;
	let mut ipv6_listener = None;

	let mut control = swarm.behaviour().stream.new_control();
	#[allow(clippy::unwrap_used)] // TODO: Error handling
	let mut incoming = control.accept(PROTOCOL).unwrap();
	let map = Arc::new(RwLock::new(HashMap::new()));
	let mut relay_config = Vec::new();
	let mut registered_relays = HashMap::new();
	let mut manual_addrs = HashSet::new();
	let mut interval = tokio::time::interval(Duration::from_secs(15));

	loop {
		tokio::select! {
			Ok(event) = rx.recv_async() => match event {
				HookEvent::PeerExpiredBy(_, identity) => {
					let Some(peer) = p2p.peers.read().unwrap_or_else(PoisonError::into_inner).get(&identity).cloned() else {
						continue;
					};

					let peer_id = remote_identity_to_libp2p_peerid(&identity);
					let addrs = {
						let state = peer.state.read().unwrap_or_else(PoisonError::into_inner);

						get_addrs(peer_id, &relay_config, state.discovered.values().flatten())
					};


					let mut control = control.clone();
					tokio::spawn(async move {
						match timeout(Duration::from_secs(5), control.open_stream_with_addrs(
							peer_id,
							PROTOCOL,
							addrs
						)).await {
							Ok(Ok(_)) => {}
							Err(_) | Ok(Err(_)) => peer.disconnected_from(id),
						};
					});
				},
				HookEvent::Shutdown { _guard } => {
					let connected_peers = swarm.connected_peers().cloned().collect::<Vec<_>>();
					for peer_id in connected_peers {
						let _ = swarm.disconnect_peer_id(peer_id);
					}

					if let Some((id, _)) = ipv4_listener.take() {
						let _ = swarm.remove_listener(id);
					}
					if let Some((id, _)) = ipv6_listener.take() {
						let _ = swarm.remove_listener(id);
					}

					// TODO: We don't break the event loop so libp2p can be polled to keep cleaning up.
					// break;
				},
				_ => {},
			},
			Some((peer_id, mut stream)) = incoming.next() => {
				let p2p = p2p.clone();
				let map = map.clone();

				tokio::spawn(async move {
					let mut actual = [0; REMOTE_IDENTITY_LEN];
					match stream.read_exact(&mut actual).await {
						Ok(_) => {},
						Err(e) => {
							warn!("Failed to read remote identity with libp2p::PeerId({peer_id:?}): {e:?}");
							return;
						},
					}
					let identity = match RemoteIdentity::from_bytes(&actual) {
						Ok(i) => i,
						Err(e) => {
							warn!("Failed to parse remote identity with libp2p::PeerId({peer_id:?}): {e:?}");
							return;
						},
					};

					// We need to go `PeerId -> RemoteIdentity` but as `PeerId` is a hash that's impossible.
					// So to make this work the connection initiator will send their remote identity.
					// It is however untrusted as they could send anything, so we convert it to a PeerId and check it matches the PeerId for this connection.
					// If it matches, we are certain they own the private key as libp2p takes care of ensuring the PeerId is trusted.
					let remote_identity_peer_id = remote_identity_to_libp2p_peerid(&identity);
					if peer_id != remote_identity_peer_id {
						warn!("Derived remote identity '{remote_identity_peer_id:?}' does not match libp2p::PeerId({peer_id:?})");
						return;
					}
					map.write().unwrap_or_else(PoisonError::into_inner).insert(peer_id, identity);

					let remote_metadata = match read_metadata(&mut stream, &p2p).await {
						Ok(metadata) => metadata,
						Err(e) => {
							warn!("Failed to read metadata from '{}': {e}", identity);
							return;
						},
					};

					let stream = UnicastStream::new(identity, stream.compat());
					p2p.connected_to_incoming(
						id,
						remote_metadata,
						stream,
					);

					debug!("established inbound stream with '{}'", identity);
				});
			},
			event = swarm.select_next_some() => match event {
				SwarmEvent::ConnectionEstablished { peer_id, endpoint, .. } => {
					if endpoint.is_relayed() {
						if let Some((remote_identity, _)) = handle.nodes.lock()
							.unwrap_or_else(PoisonError::into_inner)
							.iter()
							.find(|(i, _)| remote_identity_to_libp2p_peerid(i) == peer_id) {
								handle.connected_via_relay.lock()
									.unwrap_or_else(PoisonError::into_inner)
									.insert(*remote_identity);
								}
					}
				}
				SwarmEvent::ConnectionClosed { peer_id, num_established: 0, .. } => {
					let Some(identity) = map.write().unwrap_or_else(PoisonError::into_inner).remove(&peer_id) else {
						warn!("Tried to remove a peer that wasn't in the map.");
						continue;
					};

					let peers = p2p.peers.read().unwrap_or_else(PoisonError::into_inner);
					let Some(peer) = peers.get(&identity) else {
						warn!("Tried to remove a peer that wasn't in the P2P system.");
						continue;
					};

					peer.disconnected_from(id);
				},
				_ => {}
			},
			Ok(event) = internal_rx.recv_async() => match event {
				InternalEvent::RegisterListener { id, ipv4, addr, result } => {
					match swarm.listen_on(socketaddr_to_quic_multiaddr(&addr)) {
						Ok(libp2p_listener_id) => {
							let this = match ipv4 {
								true => &mut ipv4_listener,
								false => &mut ipv6_listener,
							};
							// TODO: Diff the `addr` & if it's changed actually update it
							if this.is_none() {
								*this =  Some((libp2p_listener_id, addr));
								p2p.register_listener_addr(id, addr);
							}

							let _ = result.send(Ok(()));
						},
						Err(e) => {
							let _ = result.send(Err(e.to_string()));
						},
					}
				},
				InternalEvent::UnregisterListener { id, ipv4, result } => {
					let this = match ipv4 {
						true => &mut ipv4_listener,
						false => &mut ipv6_listener,
					};
					if let Some((addr_id, addr)) = this.take() {
						if swarm.remove_listener(addr_id) {
							p2p.unregister_listener_addr(id, addr);
						}
					}
					let _ = result.send(Ok(()));
				},
				InternalEvent::RegisterRelays { relays, result } => {
					// TODO: We should only add some of the relays - This is discussion in P2P documentation about the Relay
					let mut err = None;
					for relay in &relays {
						let peer_id = match PeerId::from_str(&relay.peer_id) {
							Ok(peer_id) => peer_id,
							Err(err) => {
								error!("Failed to parse Relay peer ID '{}': {err:?}", relay.peer_id);
								continue;
							},
						};

						let addrs = relay
							.addrs
							.iter()
							.map(socketaddr_to_quic_multiaddr)
							.collect::<Vec<_>>();

						for addr in addrs {
							swarm
								.behaviour_mut()
								.autonat
								.add_server(peer_id, Some(addr.clone()));
							swarm.add_peer_address(peer_id, addr);
						}

						match swarm.listen_on(
							Multiaddr::empty()
								.with(Protocol::Memory(40))
								.with(Protocol::P2p(peer_id))
								.with(Protocol::P2pCircuit)
						) {
							Ok(listener_id) => {
								for addr in &relay.addrs {
									registered_relays.insert(*addr, listener_id);
								}
							},
							Err(e) => {
								err = Some(format!("Failed to listen on relay server '{}': {e}", relay.id));
								break;
							},
						}
					}

					if let Some(err) = err {
						let _ = result.send(Err(err));
						continue;
					}

					// Cleanup connections to relays that are no longer in the config
					// We intentionally do this after establishing new connections so we don't have a gap in connectivity
					for (addr, listener_id) in &registered_relays {
						if relays.iter().any(|e| e.addrs.contains(addr)) {
							continue;
						}

						swarm.remove_listener(*listener_id);
					}

					relay_config = relays;

					result.send(Ok(())).ok();
				},
				InternalEvent::RegisterPeerAddr { addrs } => {
					manual_addrs = addrs;
				}
			},
			Some(req) = connect_rx.recv() => {
				let mut control = control.clone();
				let self_remote_identity = p2p.identity().to_remote_identity();
				let map = map.clone();
				let p2p = p2p.clone();
				let peer_id = remote_identity_to_libp2p_peerid(&req.to);
				let addrs = get_addrs(peer_id, &relay_config, req.addrs.iter());

				tokio::spawn(async move {
					match control.open_stream_with_addrs(
						peer_id,
						PROTOCOL,
						addrs,
					).await {
						Ok(mut stream) => {
							map.write().unwrap_or_else(PoisonError::into_inner).insert(peer_id, req.to);

							match stream.write_all(&self_remote_identity.get_bytes()).await {
								Ok(_) => {
									debug!("Established outbound stream with '{}'", req.to);

									let remote_metadata = match send_metadata(&mut stream, &p2p).await {
										Ok(metadata) => metadata,
										Err(e) => {
											let _ = req.tx.send(Err(e));
											return;
										},
									};

									p2p.connected_to_outgoing(id, remote_metadata, req.to);

									let _ = req.tx.send(Ok(UnicastStream::new(req.to, stream.compat())));
								},
								Err(e) => {
									let _ = req.tx.send(Err(e.to_string()));
								},
							}
						},
						Err(e) => {
							let _ = req.tx.send(Err(e.to_string()));
						},
					}
				});
			}
			_ = interval.tick() => {
				let addrs = manual_addrs.clone();

				for addr in addrs {
					let err = swarm.dial(socketaddr_to_quic_multiaddr(&addr));
					debug!("Attempting connection to {:?} {}", addr, match err {
						Ok(_) => "successfully".into(),
						Err(err) => format!("with error: {err}"),
					});
				}
			}
		}
	}
}

async fn send_metadata(
	stream: &mut Stream,
	p2p: &Arc<P2P>,
) -> Result<HashMap<String, String>, String> {
	{
		let metadata = p2p.metadata().clone();
		let result = rmp_serde::encode::to_vec_named(&metadata)
			.map_err(|err| format!("Error encoding metadata: {err:?}"))?;
		stream
			.write_all(&(result.len() as u64).to_le_bytes())
			.await
			.map_err(|err| format!("Error writing metadata length: {err:?}"))?;
		stream
			.write_all(&result)
			.await
			.map_err(|err| format!("Error writing metadata: {err:?}"))?;
	}

	let mut len = [0; 8];
	stream
		.read_exact(&mut len)
		.await
		.map_err(|err| format!("Error reading metadata length: {err:?}"))?;
	let len = u64::from_le_bytes(len);
	if len > 1000 {
		return Err("Error metadata too large".into());
	}
	let mut buf = vec![0; len as usize];
	stream
		.read_exact(&mut buf)
		.await
		.map_err(|err| format!("Error reading metadata length: {err:?}"))?;
	rmp_serde::decode::from_slice(&buf).map_err(|err| format!("Error decoding metadata: {err:?}"))
}

async fn read_metadata(
	stream: &mut Stream,
	p2p: &Arc<P2P>,
) -> Result<HashMap<String, String>, String> {
	let metadata = {
		let mut len = [0; 8];
		stream
			.read_exact(&mut len)
			.await
			.map_err(|err| format!("Error reading metadata length: {err:?}"))?;
		let len = u64::from_le_bytes(len);
		if len > 1000 {
			return Err("Error metadata too large".into());
		}
		let mut buf = vec![0; len as usize];
		stream
			.read_exact(&mut buf)
			.await
			.map_err(|err| format!("Error reading metadata length: {err:?}"))?;
		rmp_serde::decode::from_slice(&buf)
			.map_err(|err| format!("Error decoding metadata: {err:?}"))?
	};

	{
		let metadata = p2p.metadata().clone();
		let result = rmp_serde::encode::to_vec_named(&metadata)
			.map_err(|err| format!("Error encoding metadata: {err:?}"))?;
		stream
			.write_all(&(result.len() as u64).to_le_bytes())
			.await
			.map_err(|err| format!("Error writing metadata length: {err:?}"))?;
		stream
			.write_all(&result)
			.await
			.map_err(|err| format!("Error writing metadata: {err:?}"))?;
	}

	Ok(metadata)
}

fn get_addrs<'a>(
	peer_id: PeerId,
	relay_config: &[RelayServerEntry],
	addrs: impl Iterator<Item = &'a PeerConnectionCandidate> + 'a,
) -> Vec<Multiaddr> {
	addrs
		.flat_map(|v| match v {
			PeerConnectionCandidate::SocketAddr(addr) => vec![socketaddr_to_quic_multiaddr(addr)],
			PeerConnectionCandidate::Relay => relay_config
				.iter()
				.filter_map(|e| match PeerId::from_str(&e.peer_id) {
					Ok(peer_id) => Some(e.addrs.iter().map(move |addr| (peer_id, addr))),
					Err(err) => {
						error!("Failed to parse peer ID '{}': {err:?}", e.peer_id);
						None
					}
				})
				.flatten()
				.map(|(relay_peer_id, addr)| {
					let mut addr = socketaddr_to_quic_multiaddr(addr);
					addr.push(Protocol::P2p(relay_peer_id));
					addr.push(Protocol::P2pCircuit);
					addr.push(Protocol::P2p(peer_id));
					addr
				})
				.collect::<Vec<_>>(),
		})
		.collect::<Vec<_>>()
}