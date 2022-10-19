use std::{collections::HashMap, time::Duration};

use rspc::Type;
use serde::*;
use serde_json::*;
use uhlc::{HLCBuilder, Timestamp, HLC, NTP64};
use uuid::Uuid;

use super::crdt::*;

// Bytes
#[derive(Default, Debug, Serialize, Type, Clone)]
pub struct Color {
	pub red: u8,
	pub green: u8,
	pub blue: u8,
}

// Unique Shared
#[derive(Default, Debug, Serialize, Type, Clone)]
pub struct Tag {
	pub color: Color,
	pub name: String,
}

// Atomic Shared
#[derive(Default, Debug, Serialize, Type, Clone)]
pub struct Object {
	pub name: String,
}

// Owned
#[derive(Serialize, Deserialize, Debug, Type, Clone)]
pub struct FilePath {
	pub id: i32,
	pub path: String,
	pub file: Option<i32>,
}

pub struct Db {
	pub files: HashMap<i32, Object>,
	pub file_paths: HashMap<i32, FilePath>,
	pub tags: HashMap<Uuid, Tag>,
	pub _operations: Vec<CRDTOperation>,
	pub _clocks: HashMap<Uuid, NTP64>,
	_clock: HLC,
	_node: Uuid,
}

impl std::fmt::Debug for Db {
	fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Db")
			.field("files", &self.files)
			.field("file_paths", &self.file_paths)
			.finish()
	}
}

impl Db {
	pub fn new(node: Uuid) -> Self {
		Self {
			files: Default::default(),
			file_paths: Default::default(),
			tags: Default::default(),
			_clocks: Default::default(),
			_node: node,
			_clock: HLCBuilder::new().with_id(node.into()).build(),
			_operations: Default::default(),
		}
	}

	pub fn register_node(&mut self, id: Uuid) {
		if !self._clocks.contains_key(&id) {
			self._clocks.insert(id, Duration::from_millis(0).into());
		}
	}

	pub fn create_crdt_operation(&mut self, typ: CRDTOperationType) -> CRDTOperation {
		let hlc_timestamp = self._clock.new_timestamp();

		let op = CRDTOperation {
			node: self._node,
			timestamp: *hlc_timestamp.get_time(),
			id: Uuid::new_v4(),
			typ,
		};

		self._operations.push(op.clone());

		op
	}

	fn compare_messages(&self, operations: Vec<CRDTOperation>) -> Vec<(CRDTOperation, bool)> {
		operations
			.into_iter()
			.map(|op| {
				let old = match &op.typ {
					CRDTOperationType::Owned(_) => false,
					CRDTOperationType::Shared(shared_op) => {
						let similar_op = self._operations.iter().find(|find_op| {
							if let CRDTOperationType::Shared(find_shared_op) = &find_op.typ {
								shared_op.model == find_shared_op.model
									&& shared_op.record_id == find_shared_op.record_id
									&& op.timestamp >= find_op.timestamp
							} else {
								false
							}
						});

						similar_op
							.map(|similar_op| similar_op.timestamp == op.timestamp)
							.unwrap_or(false)
					}
				};

				(op, old)
			})
			.collect()
	}

	pub fn receive_crdt_operations(&mut self, ops: Vec<CRDTOperation>) {
		for op in &ops {
			self._clock
				.update_with_timestamp(&Timestamp::new(op.timestamp, op.node.into()))
				.ok();

			self._clocks.insert(self._node.clone(), op.timestamp);
		}

		for (op, old) in self.compare_messages(ops) {
			let push_op = op.clone();

			if !old {
				match op.typ {
					CRDTOperationType::Shared(shared_op) => match shared_op.model.as_str() {
						"File" => {
							let id = from_value(shared_op.record_id).unwrap();

							match shared_op.data {
								SharedOperationData::Create(SharedOperationCreateData::Atomic) => {
									self.files.insert(id, Default::default());
								}
								SharedOperationData::Update { field, value } => {
									let mut file = self.files.get_mut(&id).unwrap();

									match field.as_str() {
										"name" => {
											file.name = from_value(value).unwrap();
										}
										_ => unreachable!(),
									}
								}
								SharedOperationData::Delete => {
									self.files.remove(&id).unwrap();
								}
								_ => {}
							}
						}
						_ => unreachable!(),
					},
					CRDTOperationType::Owned(owned_op) => match owned_op.model.as_str() {
						"FilePath" => {
							for item in owned_op.items {
								let id = from_value(item.id).unwrap();

								match item.data {
									OwnedOperationData::Create(data) => {
										self.file_paths
											.insert(id, from_value(Value::Object(data)).unwrap());
									}
									OwnedOperationData::Update(data) => {
										let obj = self.file_paths.get_mut(&id).unwrap();

										for (key, value) in data {
											match key.as_str() {
												"path" => obj.path = from_value(value).unwrap(),
												"file" => obj.file = from_value(value).unwrap(),
												_ => unreachable!(),
											}
										}
									}
									OwnedOperationData::Delete => {
										self.file_paths.remove(&id);
									}
								}
							}
						}
						_ => unreachable!(),
					},
				}
			}

			self._operations.push(push_op)
		}
	}
}
