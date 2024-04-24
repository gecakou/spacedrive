use std::{error::Error, sync::Arc};

use axum::{body::Body, http, Router};
use hyper::{server::conn::Http, Response};
use sd_p2p::{RemoteIdentity, UnicastStream, P2P};
use tokio::io::AsyncWriteExt;
use tracing::debug;

use crate::{p2p::Header, Node};

/// Transfer an rspc query to a remote node.
#[allow(unused)]
pub async fn remote_rspc(
	p2p: Arc<P2P>,
	identity: RemoteIdentity,
	request: http::Request<axum::body::Body>,
) -> Result<Response<Body>, Box<dyn Error>> {
	let peer = p2p
		.peers()
		.get(&identity)
		.ok_or("Peer not found, has it been discovered?")?
		.clone();
	let mut stream = peer.new_stream().await?;

	stream.write_all(&Header::Http.to_bytes()).await?;

	let (mut sender, conn) = hyper::client::conn::handshake(stream).await?;
	tokio::task::spawn(async move {
		if let Err(err) = conn.await {
			println!("Connection error: {:?}", err);
		}
	});

	sender.send_request(request).await.map_err(Into::into)
}

pub(crate) async fn receiver(
	stream: UnicastStream,
	service: &mut Router,
	node: &Node,
) -> Result<(), Box<dyn Error>> {
	debug!(
		"Received http request from peer '{}'",
		stream.remote_identity(),
	);

	// TODO: Authentication
	#[allow(clippy::todo)]
	if node.config.get().await.p2p.remote_access {
		todo!("No way buddy!");
	}

	Http::new()
		.http1_only(true)
		.http1_keep_alive(true)
		.serve_connection(stream, service)
		.with_upgrades()
		.await
		.map_err(Into::into)
}
