use crate::prisma::{file_path, object};

use rspc::Type;
use serde::{Deserialize, Serialize};

pub mod cas;
pub mod file_identifier;
pub mod fs;
pub mod preview;
pub mod tag;
pub mod validation;

// Objects are primarily created by the identifier from Paths
// Some Objects are purely virtual, unless they have one or more associated Paths, which refer to a file found in a Location
// Objects are what can be added to Spaces

// Object selectables!
object::select!(object_just_id_has_thumbnail { id has_thumbnail });
object::select!(object_just_pub_id_with_file_paths_just_id_cas_id {
	pub_id
	file_paths: select { id cas_id }
});

// The response to provide the Explorer when looking at Objects
#[derive(Debug, Serialize, Deserialize, Type)]
pub struct ObjectsForExplorer {
	pub objects: Vec<ObjectData>,
	// pub context: ExplorerContext,
}

// #[derive(Debug, Serialize, Deserialize, Type)]
// pub enum ExplorerContext {
// 	Location(Box<file_path::Data>),
// 	Space(Box<space::Data>),
// 	Tag(Box<tag::Data>),
// 	// Search(Box<file_path::Data>),
// }

#[derive(Debug, Serialize, Deserialize, Type)]
pub enum ObjectData {
	Object(Box<object::Data>),
	Path(Box<file_path::Data>),
}
