//! A system for creating encrypted tunnels between peers over untrusted connections.

mod tunnel;

pub use sd_p2p::{Identity, IdentityErr, RemoteIdentity};
pub use tunnel::*;
