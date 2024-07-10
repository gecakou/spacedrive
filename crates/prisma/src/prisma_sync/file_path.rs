// File generated by SD Sync Generator. DO NOT EDIT

use super::prisma::*;
use prisma_client_rust::scalar_types::*;
#[derive(serde :: Serialize, serde :: Deserialize, Clone, Debug)]
pub struct SyncId {
	pub pub_id: Bytes,
}
impl sd_sync::SyncId for SyncId {
	type Model = file_path::Types;
}
pub const MODEL_ID: u16 = 2u16;
impl sd_sync::SyncModel for file_path::Types {
	const MODEL_ID: u16 = MODEL_ID;
}
impl sd_sync::SharedSyncModel for file_path::Types {
	type SyncId = SyncId;
}
impl file_path::SetParam {
	pub fn deserialize(field: &str, val: ::rmpv::Value) -> Option<Self> {
		Some(match field {
			file_path::id::NAME => file_path::id::set(::rmpv::ext::from_value(val).unwrap()),
			file_path::pub_id::NAME => {
				file_path::pub_id::set(::rmpv::ext::from_value(val).unwrap())
			}
			file_path::is_dir::NAME => {
				file_path::is_dir::set(::rmpv::ext::from_value(val).unwrap())
			}
			file_path::cas_id::NAME => {
				file_path::cas_id::set(::rmpv::ext::from_value(val).unwrap())
			}
			file_path::integrity_checksum::NAME => {
				file_path::integrity_checksum::set(::rmpv::ext::from_value(val).unwrap())
			}
			file_path::location::NAME => {
				let val: std::collections::HashMap<String, rmpv::Value> =
					::rmpv::ext::from_value(val).unwrap();
				let val = val.into_iter().next().unwrap();
				file_path::location::connect(
					location::UniqueWhereParam::deserialize(&val.0, val.1).unwrap(),
				)
			}
			file_path::materialized_path::NAME => {
				file_path::materialized_path::set(::rmpv::ext::from_value(val).unwrap())
			}
			file_path::name::NAME => file_path::name::set(::rmpv::ext::from_value(val).unwrap()),
			file_path::extension::NAME => {
				file_path::extension::set(::rmpv::ext::from_value(val).unwrap())
			}
			file_path::hidden::NAME => {
				file_path::hidden::set(::rmpv::ext::from_value(val).unwrap())
			}
			file_path::size_in_bytes::NAME => {
				file_path::size_in_bytes::set(::rmpv::ext::from_value(val).unwrap())
			}
			file_path::size_in_bytes_bytes::NAME => {
				file_path::size_in_bytes_bytes::set(::rmpv::ext::from_value(val).unwrap())
			}
			file_path::inode::NAME => file_path::inode::set(::rmpv::ext::from_value(val).unwrap()),
			file_path::object::NAME => {
				let val: std::collections::HashMap<String, rmpv::Value> =
					::rmpv::ext::from_value(val).unwrap();
				let val = val.into_iter().next().unwrap();
				file_path::object::connect(
					object::UniqueWhereParam::deserialize(&val.0, val.1).unwrap(),
				)
			}
			file_path::key_id::NAME => {
				file_path::key_id::set(::rmpv::ext::from_value(val).unwrap())
			}
			file_path::date_created::NAME => {
				file_path::date_created::set(::rmpv::ext::from_value(val).unwrap())
			}
			file_path::date_modified::NAME => {
				file_path::date_modified::set(::rmpv::ext::from_value(val).unwrap())
			}
			file_path::date_indexed::NAME => {
				file_path::date_indexed::set(::rmpv::ext::from_value(val).unwrap())
			}
			_ => return None,
		})
	}
}
impl file_path::UniqueWhereParam {
	pub fn deserialize(field: &str, val: ::rmpv::Value) -> Option<Self> {
		Some(match field {
			file_path::id::NAME => file_path::id::equals(::rmpv::ext::from_value(val).unwrap()),
			file_path::pub_id::NAME => {
				file_path::pub_id::equals(::rmpv::ext::from_value(val).unwrap())
			}
			_ => return None,
		})
	}
}
