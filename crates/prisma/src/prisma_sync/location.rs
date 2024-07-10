// File generated by SD Sync Generator. DO NOT EDIT

use super::prisma::*;
use prisma_client_rust::scalar_types::*;
#[derive(serde :: Serialize, serde :: Deserialize, Clone, Debug)]
pub struct SyncId {
	pub pub_id: Bytes,
}
impl sd_sync::SyncId for SyncId {
	type Model = location::Types;
}
pub const MODEL_ID: u16 = 1u16;
impl sd_sync::SyncModel for location::Types {
	const MODEL_ID: u16 = MODEL_ID;
}
impl sd_sync::SharedSyncModel for location::Types {
	type SyncId = SyncId;
}
impl location::SetParam {
	pub fn deserialize(field: &str, val: ::rmpv::Value) -> Option<Self> {
		Some(match field {
			location::id::NAME => location::id::set(::rmpv::ext::from_value(val).unwrap()),
			location::pub_id::NAME => location::pub_id::set(::rmpv::ext::from_value(val).unwrap()),
			location::name::NAME => location::name::set(::rmpv::ext::from_value(val).unwrap()),
			location::path::NAME => location::path::set(::rmpv::ext::from_value(val).unwrap()),
			location::total_capacity::NAME => {
				location::total_capacity::set(::rmpv::ext::from_value(val).unwrap())
			}
			location::available_capacity::NAME => {
				location::available_capacity::set(::rmpv::ext::from_value(val).unwrap())
			}
			location::size_in_bytes::NAME => {
				location::size_in_bytes::set(::rmpv::ext::from_value(val).unwrap())
			}
			location::is_archived::NAME => {
				location::is_archived::set(::rmpv::ext::from_value(val).unwrap())
			}
			location::generate_preview_media::NAME => {
				location::generate_preview_media::set(::rmpv::ext::from_value(val).unwrap())
			}
			location::sync_preview_media::NAME => {
				location::sync_preview_media::set(::rmpv::ext::from_value(val).unwrap())
			}
			location::hidden::NAME => location::hidden::set(::rmpv::ext::from_value(val).unwrap()),
			location::date_created::NAME => {
				location::date_created::set(::rmpv::ext::from_value(val).unwrap())
			}
			location::scan_state::NAME => {
				location::scan_state::set(::rmpv::ext::from_value(val).unwrap())
			}
			location::instance::NAME => {
				let val: std::collections::HashMap<String, rmpv::Value> =
					::rmpv::ext::from_value(val).unwrap();
				let val = val.into_iter().next().unwrap();
				location::instance::connect(
					instance::UniqueWhereParam::deserialize(&val.0, val.1).unwrap(),
				)
			}
			_ => return None,
		})
	}
}
impl location::UniqueWhereParam {
	pub fn deserialize(field: &str, val: ::rmpv::Value) -> Option<Self> {
		Some(match field {
			location::id::NAME => location::id::equals(::rmpv::ext::from_value(val).unwrap()),
			location::pub_id::NAME => {
				location::pub_id::equals(::rmpv::ext::from_value(val).unwrap())
			}
			_ => return None,
		})
	}
}
