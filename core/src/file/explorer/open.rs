use std::path::Path;

use crate::{
	encode::thumb::THUMBNAIL_CACHE_DIR_NAME,
	file::{DirectoryWithContents, FileError, FilePath},
	prisma::file_path,
	state::client,
	sys::locations::get_location,
	CoreContext,
};

pub async fn open_dir(
	ctx: &CoreContext,
	location_id: &i32,
	path: &str,
) -> Result<DirectoryWithContents, FileError> {
	let db = &ctx.database;
	let config = client::get();

	// get location
	let location = get_location(ctx, location_id.clone()).await?;

	let directory = db
		.file_path()
		.find_first(vec![
			file_path::location_id::equals(location.id),
			file_path::materialized_path::equals(path.into()),
			file_path::is_dir::equals(true),
		])
		.exec()
		.await?
		.ok_or(FileError::DirectoryNotFound(path.to_string()))?;

	let files = db
		.file_path()
		.find_many(vec![file_path::parent_id::equals(Some(directory.id))])
		.exec()
		.await?;

	let files: Vec<FilePath> = files.into_iter().map(|l| l.into()).collect();

	let mut contents: Vec<FilePath> = vec![];

	for mut file in files {
		if file.temp_cas_id.is_some() {
			let path = Path::new(&config.data_path)
				.join(THUMBNAIL_CACHE_DIR_NAME)
				.join(format!("{}", location.id))
				.join(file.temp_cas_id.as_ref().unwrap())
				.with_extension("webp");

			let exists = path.exists();
			file.has_local_thumbnail = exists;
		}
		contents.push(file);
	}

	Ok(DirectoryWithContents {
		directory: directory.into(),
		contents,
	})
}
