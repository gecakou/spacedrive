use crate::{
	job::JobError,
	prisma::{file_path, location, PrismaClient},
};

use std::{ffi::OsStr, path::PathBuf};

use serde::{Deserialize, Serialize};

pub mod copy;
pub mod cut;
pub mod decrypt;
pub mod delete;
pub mod encrypt;
pub mod erase;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub enum ObjectType {
	File,
	Directory,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FsInfo {
	pub obj_id: Option<i32>,
	pub obj_name: String,
	pub obj_path: PathBuf,
	pub obj_type: ObjectType,
}

pub fn osstr_to_string(os_str: Option<&OsStr>) -> Result<String, JobError> {
	let string = os_str
		.ok_or(JobError::OsStr)?
		.to_str()
		.ok_or(JobError::OsStr)?
		.to_string();

	Ok(string)
}

pub async fn get_path_from_location_id(
	db: &PrismaClient,
	location_id: i32,
) -> Result<PathBuf, JobError> {
	let location = db
		.location()
		.find_unique(location::id::equals(location_id))
		.exec()
		.await?
		.ok_or(JobError::MissingData {
			value: String::from("location which matches location_id"),
		})?;

	location
		.local_path
		.as_ref()
		.map(PathBuf::from)
		.ok_or(JobError::MissingData {
			value: String::from("path when cast as `PathBuf`"),
		})
}

pub async fn context_menu_fs_info(
	db: &PrismaClient,
	location_id: i32,
	path_id: i32,
) -> Result<FsInfo, JobError> {
	let location_path = get_path_from_location_id(db, location_id).await?;

	let item = db
		.file_path()
		.find_unique(file_path::location_id_id(location_id, path_id))
		.exec()
		.await?
		.ok_or(JobError::MissingData {
			value: String::from("file_path that matches both location id and path id"),
		})?;

	let obj_path = location_path.join(&item.materialized_path);

	// i don't know if this covers symlinks
	let obj_type = if item.is_dir {
		ObjectType::Directory
	} else {
		ObjectType::File
	};

	Ok(FsInfo {
		obj_id: item.object_id,
		obj_name: item.materialized_path,
		obj_type,
		obj_path,
	})
}
