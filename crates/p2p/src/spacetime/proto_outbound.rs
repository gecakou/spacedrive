use std::future::{ready, Ready};

use libp2p::{
	core::UpgradeInfo, futures::AsyncWriteExt, swarm::NegotiatedSubstream, OutboundUpgrade,
};
use tokio::sync::oneshot;
use tracing::error;

use super::{SpaceTimeProtocolName, UnicastStream, BROADCAST_DISCRIMINATOR};

#[derive(Debug)]
pub enum OutboundRequest {
	Broadcast(Vec<u8>),
	Unicast(oneshot::Sender<UnicastStream>),
}

pub struct OutboundProtocol(pub(crate) &'static [u8], pub(crate) OutboundRequest);

impl UpgradeInfo for OutboundProtocol {
	type Info = SpaceTimeProtocolName;
	type InfoIter = [Self::Info; 1];

	fn protocol_info(&self) -> Self::InfoIter {
		[SpaceTimeProtocolName(self.0)]
	}
}

impl OutboundUpgrade<NegotiatedSubstream> for OutboundProtocol {
	type Output = ();
	type Error = ();
	type Future = Ready<Result<(), ()>>;

	fn upgrade_outbound(self, mut io: NegotiatedSubstream, _protocol: Self::Info) -> Self::Future {
		match self.1 {
			OutboundRequest::Broadcast(data) => {
				tokio::spawn(async move {
					io.write_all(&[BROADCAST_DISCRIMINATOR]).await.unwrap();
					if let Err(err) = io.write_all(&data).await {
						// TODO: Print the peer which we failed to send to here
						error!("Error sending broadcast: {:?}", err);
					}
					io.flush().await.unwrap();
					io.close().await.unwrap();
				});
			}
			OutboundRequest::Unicast(sender) => {
				// We write the discriminator to the stream in the `Manager::stream` method before returning the stream to the user to make async a tad nicer.
				sender.send(UnicastStream::new(io)).unwrap();
			}
		}

		ready(Ok(()))
	}
}
