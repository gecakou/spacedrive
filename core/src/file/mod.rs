use crate::{
	library::LibraryContext,
	prisma::{self, file, file_path},
	sys::SysError,
	ClientQuery, CoreError, CoreEvent, CoreResponse, LibraryQuery,
};

use chrono::{DateTime, Utc};
use int_enum::IntEnum;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use ts_rs::TS;

pub mod cas;
pub mod explorer;
pub mod indexer;

// A unique file
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct File {
	pub id: i32,
	pub cas_id: String,
	pub integrity_checksum: Option<String>,
	pub size_in_bytes: String,
	pub kind: FileKind,

	pub hidden: bool,
	pub favorite: bool,
	pub important: bool,
	pub has_thumbnail: bool,
	pub has_thumbstrip: bool,
	pub has_video_preview: bool,
	// pub encryption: EncryptionAlgorithm,
	pub ipfs_id: Option<String>,
	pub note: Option<String>,

	pub date_created: DateTime<Utc>,
	pub date_modified: DateTime<Utc>,
	pub date_indexed: DateTime<Utc>,

	pub paths: Vec<FilePath>,
	// pub media_data: Option<MediaData>,
	// pub tags: Vec<Tag>,
	// pub label: Vec<Label>,
}

// A physical file path
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct FilePath {
	pub id: i32,
	pub is_dir: bool,
	pub location_id: i32,
	pub materialized_path: String,
	pub name: String,
	pub extension: Option<String>,
	pub file_id: Option<i32>,
	pub parent_id: Option<i32>,

	pub date_created: DateTime<Utc>,
	pub date_modified: DateTime<Utc>,
	pub date_indexed: DateTime<Utc>,

	pub file: Option<File>,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, Eq, PartialEq, IntEnum)]
#[ts(export)]
pub enum FileKind {
	Unknown = 0,
	Directory = 1,
	Package = 2,
	Archive = 3,
	Image = 4,
	Video = 5,
	Audio = 6,
	Plaintext = 7,
	Alias = 8,
}

impl From<file::Data> for File {
	fn from(data: file::Data) -> Self {
		Self {
			id: data.id,
			cas_id: data.cas_id,
			integrity_checksum: data.integrity_checksum,
			kind: IntEnum::from_int(data.kind).unwrap(),
			size_in_bytes: data.size_in_bytes.to_string(),
			//   encryption: EncryptionAlgorithm::from_int(data.encryption).unwrap(),
			ipfs_id: data.ipfs_id,
			hidden: data.hidden,
			favorite: data.favorite,
			important: data.important,
			has_thumbnail: data.has_thumbnail,
			has_thumbstrip: data.has_thumbstrip,
			has_video_preview: data.has_video_preview,
			note: data.note,
			date_created: data.date_created.into(),
			date_modified: data.date_modified.into(),
			date_indexed: data.date_indexed.into(),
			paths: vec![],
		}
	}
}

impl From<Box<file::Data>> for File {
	fn from(data: Box<file::Data>) -> Self {
		Self::from(*data)
	}
}

impl From<file_path::Data> for FilePath {
	fn from(data: file_path::Data) -> Self {
		Self {
			id: data.id,
			is_dir: data.is_dir,
			materialized_path: data.materialized_path,
			file_id: data.file_id,
			parent_id: data.parent_id,
			location_id: data.location_id.unwrap_or(0),
			date_indexed: data.date_indexed.into(),
			name: data.name,
			extension: data.extension,
			date_created: data.date_created.into(),
			date_modified: data.date_modified.into(),
			file: data.file.unwrap_or(None).map(Into::into),
		}
	}
}

#[derive(Serialize, Deserialize, TS, Debug)]
#[ts(export)]
pub struct DirectoryWithContents {
	pub directory: FilePath,
	pub contents: Vec<FilePath>,
}

#[derive(Error, Debug)]
pub enum FileError {
	#[error("Directory not found (path: {0:?})")]
	DirectoryNotFound(PathBuf),
	#[error("File not found (path: {0:?})")]
	FileNotFound(PathBuf),
	#[error("Database error")]
	DatabaseError(#[from] prisma::QueryError),
	#[error("System error")]
	SysError(#[from] SysError),
}

pub async fn set_note(
	ctx: LibraryContext,
	id: i32,
	note: Option<String>,
) -> Result<CoreResponse, CoreError> {
	let _response = ctx
		.db
		.file()
		.find_unique(file::id::equals(id))
		.update(vec![file::note::set(note.clone())])
		.exec()
		.await
		.unwrap();

	send_invalidate_query(&ctx).await;

	Ok(CoreResponse::Success(()))
}

pub async fn favorite(
	ctx: LibraryContext,
	id: i32,
	favorite: bool,
) -> Result<CoreResponse, CoreError> {
	let _response = ctx
		.db
		.file()
		.find_unique(file::id::equals(id))
		.update(vec![file::favorite::set(favorite)])
		.exec()
		.await
		.unwrap();

	send_invalidate_query(&ctx).await;

	Ok(CoreResponse::Success(()))
}

async fn send_invalidate_query(ctx: &LibraryContext) {
	ctx.emit(CoreEvent::InvalidateQuery(ClientQuery::LibraryQuery {
		library_id: ctx.id,
		query: LibraryQuery::GetExplorerDir {
			limit: 0,
			path: PathBuf::new(),
			location_id: 0,
		},
	}))
	.await;
}
