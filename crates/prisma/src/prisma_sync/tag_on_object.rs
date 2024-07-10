// File generated by SD Sync Generator. DO NOT EDIT

use super::prisma::*;
use prisma_client_rust::scalar_types::*;
#[derive(serde :: Serialize, serde :: Deserialize, Clone, Debug)]
pub struct SyncId {
	pub tag: super::tag::SyncId,
	pub object: super::object::SyncId,
}
impl sd_sync::SyncId for SyncId {
	type Model = tag_on_object::Types;
}
impl sd_sync::RelationSyncId for SyncId {
	type ItemSyncId = super::object::SyncId;
	type GroupSyncId = super::tag::SyncId;
	fn split(&self) -> (&Self::ItemSyncId, &Self::GroupSyncId) {
		(&self.object, &self.tag)
	}
}
pub const MODEL_ID: u16 = 6u16;
impl sd_sync::SyncModel for tag_on_object::Types {
	const MODEL_ID: u16 = MODEL_ID;
}
impl sd_sync::RelationSyncModel for tag_on_object::Types {
	type SyncId = SyncId;
}
impl tag_on_object::SetParam {
	pub fn deserialize(field: &str, val: ::rmpv::Value) -> Option<Self> {
		Some(match field {
			tag_on_object::object::NAME => {
				let val: std::collections::HashMap<String, rmpv::Value> =
					::rmpv::ext::from_value(val).unwrap();
				let val = val.into_iter().next().unwrap();
				tag_on_object::object::connect(
					object::UniqueWhereParam::deserialize(&val.0, val.1).unwrap(),
				)
			}
			tag_on_object::tag::NAME => {
				let val: std::collections::HashMap<String, rmpv::Value> =
					::rmpv::ext::from_value(val).unwrap();
				let val = val.into_iter().next().unwrap();
				tag_on_object::tag::connect(
					tag::UniqueWhereParam::deserialize(&val.0, val.1).unwrap(),
				)
			}
			tag_on_object::date_created::NAME => {
				tag_on_object::date_created::set(::rmpv::ext::from_value(val).unwrap())
			}
			_ => return None,
		})
	}
}
