// File generated by SD Sync Generator. DO NOT EDIT

use super::prisma::*;
use prisma_client_rust::scalar_types::*;
#[derive(serde :: Serialize, serde :: Deserialize, Clone, Debug)]
pub struct SyncId {
	pub key: String,
}
impl sd_sync::SyncId for SyncId {
	type Model = preference::Types;
}
pub const MODEL_ID: u16 = 9u16;
impl sd_sync::SyncModel for preference::Types {
	const MODEL_ID: u16 = MODEL_ID;
}
impl sd_sync::SharedSyncModel for preference::Types {
	type SyncId = SyncId;
}
impl preference::SetParam {
	pub fn deserialize(field: &str, val: ::rmpv::Value) -> Option<Self> {
		Some(match field {
			preference::key::NAME => preference::key::set(::rmpv::ext::from_value(val).unwrap()),
			preference::value::NAME => {
				preference::value::set(::rmpv::ext::from_value(val).unwrap())
			}
			_ => return None,
		})
	}
}
impl preference::UniqueWhereParam {
	pub fn deserialize(field: &str, val: ::rmpv::Value) -> Option<Self> {
		Some(match field {
			preference::key::NAME => preference::key::equals(::rmpv::ext::from_value(val).unwrap()),
			_ => return None,
		})
	}
}
