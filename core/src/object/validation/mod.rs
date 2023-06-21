use crate::{location::file_path_helper::FilePathError, util::error::FileIOError};

use std::path::Path;

use thiserror::Error;

pub mod hash;
pub mod validator_job;

#[derive(Error, Debug)]
pub enum ValidatorError {
	#[error("sub path not found: <path='{}'>", .0.display())]
	SubPathNotFound(Box<Path>),

	// Internal errors
	#[error("database error: {0}")]
	Database(#[from] prisma_client_rust::QueryError),
	#[error(transparent)]
	FilePath(#[from] FilePathError),
	#[error(transparent)]
	FileIO(#[from] FileIOError),
}
