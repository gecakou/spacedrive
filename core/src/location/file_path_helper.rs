use crate::{library::LibraryContext, prisma::file_path};

use std::sync::atomic::{AtomicI32, Ordering};

use prisma_client_rust::{Direction, QueryError};

static LAST_FILE_PATH_ID: AtomicI32 = AtomicI32::new(0);

file_path::select!(file_path_id_only { id });

pub async fn get_max_file_path_id(library_ctx: &LibraryContext) -> Result<i32, QueryError> {
	let mut last_id = LAST_FILE_PATH_ID.load(Ordering::Acquire);
	if last_id == 0 {
		last_id = fetch_max_file_path_id(library_ctx).await?;
		LAST_FILE_PATH_ID.store(last_id, Ordering::Release);
	}

	Ok(last_id)
}

pub fn set_max_file_path_id(id: i32) {
	LAST_FILE_PATH_ID.store(id, Ordering::Relaxed);
}

async fn fetch_max_file_path_id(library_ctx: &LibraryContext) -> Result<i32, QueryError> {
	Ok(library_ctx
		.db
		.file_path()
		.find_first(vec![])
		.order_by(file_path::id::order(Direction::Desc))
		.select(file_path_id_only::select())
		.exec()
		.await?
		.map(|r| r.id)
		.unwrap_or(0))
}

#[cfg(feature = "location-watcher")]
pub async fn create_file_path(
	library_ctx: &LibraryContext,
	location_id: i32,
	mut materialized_path: String,
	name: String,
	extension: Option<String>,
	parent_id: Option<i32>,
	is_dir: bool,
) -> Result<file_path::Data, QueryError> {
	use crate::prisma::location;

	let mut last_id = LAST_FILE_PATH_ID.load(Ordering::Acquire);
	if last_id == 0 {
		last_id = fetch_max_file_path_id(library_ctx).await?;
	}

	// If this new file_path is a directory, materialized_path must end with "/"
	if is_dir && !materialized_path.ends_with('/') {
		materialized_path += "/";
	}

	let next_id = last_id + 1;

	let created_path = library_ctx
		.db
		.file_path()
		.create(
			next_id,
			location::id::equals(location_id),
			materialized_path,
			name,
			vec![
				file_path::parent_id::set(parent_id),
				file_path::is_dir::set(is_dir),
				file_path::extension::set(extension),
			],
		)
		.exec()
		.await?;

	LAST_FILE_PATH_ID.store(next_id, Ordering::Release);

	Ok(created_path)
}
