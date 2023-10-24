use crate::{
	library::{Libraries, LibraryId},
	object::media::thumbnail::ONE_SEC,
	util::{
		error::FileIOError,
		version_manager::{ManagedVersion, VersionManager, VersionManagerError},
	},
};

use sd_prisma::prisma::{file_path, PrismaClient};

use std::{
	collections::{HashMap, HashSet},
	path::{Path, PathBuf},
	sync::Arc,
};

use futures_concurrency::future::{Join, TryJoin};
use int_enum::IntEnum;
use tokio::{
	fs, io, spawn,
	time::{sleep, timeout},
};
use tracing::{debug, error, info, trace, warn};

use super::{
	get_shard_hex, ThumbnailerError, EPHEMERAL_DIR, THIRTY_SECS, THUMBNAIL_CACHE_DIR_NAME,
	VERSION_FILE, WEBP_EXTENSION,
};

#[derive(IntEnum, Debug, Clone, Copy, Eq, PartialEq, strum::Display)]
#[repr(i32)]
enum ThumbnailVersion {
	V1 = 1,
	V2 = 2,
	V3 = 3,
	Unknown = 0,
}

impl ManagedVersion for ThumbnailVersion {
	const LATEST_VERSION: Self = Self::V3;
	type MigrationError = ThumbnailerError;
}

pub(super) async fn init_thumbnail_dir(
	data_dir: impl AsRef<Path>,
	libraries_manager: Arc<Libraries>,
) -> Result<PathBuf, ThumbnailerError> {
	debug!("Initializing thumbnail directory");
	let thumbnails_directory = data_dir.as_ref().join(THUMBNAIL_CACHE_DIR_NAME);

	debug!("Thumbnail directory: {:?}", thumbnails_directory);

	// create thumbnails base directory
	fs::create_dir_all(&thumbnails_directory)
		.await
		.map_err(|e| FileIOError::from((&thumbnails_directory, e)))?;

	spawn({
		let thumbnails_directory = thumbnails_directory.clone();
		async move {
			let Ok(databases) = timeout(THIRTY_SECS, async move {
				loop {
					let libraries = libraries_manager.get_all().await;
					if !libraries.is_empty() {
						break libraries
							.into_iter()
							.map(|library| (library.id, Arc::clone(&library.db)))
							.collect::<HashMap<_, _>>();
					}

					sleep(ONE_SEC).await;
				}
			})
			.await
			else {
				warn!(
					"Failed to get libraries after 30 seconds, thumbnailer migration will not work; \
					Ignore this warning if you don't created libraries yet."
				);
				return;
			};

			if let Err(e) = process_migration(thumbnails_directory, databases).await {
				error!("Failed to migrate thumbnails: {e:#?}");
			}
		}
	});

	Ok(thumbnails_directory)
}

async fn process_migration(
	thumbnails_directory: impl AsRef<Path>,
	databases: HashMap<LibraryId, Arc<PrismaClient>>,
) -> Result<(), ThumbnailerError> {
	let thumbnails_directory = thumbnails_directory.as_ref();

	let version_manager =
		VersionManager::<ThumbnailVersion>::new(thumbnails_directory.join(VERSION_FILE));

	// create all other directories, for each library and for ephemeral thumbnails
	databases
		.keys()
		.map(|library_id| thumbnails_directory.join(library_id.to_string()))
		.chain([thumbnails_directory.join(EPHEMERAL_DIR)])
		.map(|path| async move {
			fs::create_dir_all(&path)
				.await
				.map_err(|e| FileIOError::from((&path, e)))
		})
		.collect::<Vec<_>>()
		.join()
		.await
		.into_iter()
		.collect::<Result<Vec<_>, _>>()?;

	let current_version = match version_manager.get_version().await {
		Ok(version) => version,
		Err(e) => {
			debug!("Thumbnail version file does not exist, starting fresh: {e:#?}");
			// Version file does not exist, start fresh
			version_manager.set_version(ThumbnailVersion::V1).await?;
			ThumbnailVersion::V1
		}
	};

	if current_version != ThumbnailVersion::LATEST_VERSION {
		info!(
			"Migrating thumbnail directory from {:?} to V3",
			current_version
		);

		// Taking a reference to databases so we can move it into the closure and comply with the borrowck
		let databases = &databases;

		version_manager
			.migrate(current_version, |current, next| {
				let thumbnail_dir = &thumbnails_directory;
				async move {
					match (current, next) {
						(ThumbnailVersion::V1, ThumbnailVersion::V2) => {
							move_to_shards(thumbnail_dir).await
						}
						(ThumbnailVersion::V2, ThumbnailVersion::V3) => {
							segregate_thumbnails_by_library(thumbnail_dir, databases).await
						}

						_ => {
							error!("Thumbnail version is not handled: {:?}", current);
							Err(VersionManagerError::UnexpectedMigration {
								current_version: current.int_value(),
								next_version: next.int_value(),
							}
							.into())
						}
					}
				}
			})
			.await?;
	}

	Ok(())
}

/// This function moves all webp files in the thumbnail directory to their respective shard folders.
/// It is used to migrate from V1 to V2.
async fn move_to_shards(thumbnails_directory: impl AsRef<Path>) -> Result<(), ThumbnailerError> {
	let thumbnails_directory = thumbnails_directory.as_ref();

	let mut dir_entries = fs::read_dir(thumbnails_directory)
		.await
		.map_err(|source| FileIOError::from((thumbnails_directory, source)))?;

	let mut count = 0;

	while let Ok(Some(entry)) = dir_entries.next_entry().await {
		if entry
			.file_type()
			.await
			.map_err(|e| FileIOError::from((entry.path(), e)))?
			.is_file()
		{
			let path = entry.path();
			if path.extension() == Some(WEBP_EXTENSION.as_ref()) {
				let file_name = entry.file_name();

				// we know they're cas_id's, so they're valid utf8
				let shard_folder = get_shard_hex(file_name.to_str().expect("Failed to parse UTF8"));

				let new_dir = thumbnails_directory.join(shard_folder);
				fs::create_dir_all(&new_dir)
					.await
					.map_err(|source| FileIOError::from((new_dir.clone(), source)))?;

				let new_path = new_dir.join(file_name);
				fs::rename(&path, &new_path)
					.await
					.map_err(|source| FileIOError::from((path.clone(), source)))?;
				count += 1;
			}
		}
	}

	info!(
		"Moved {} webp files to their respective shard folders.",
		count
	);

	Ok(())
}

async fn segregate_thumbnails_by_library(
	thumbnails_directory: impl AsRef<Path>,
	databases: &HashMap<LibraryId, Arc<PrismaClient>>,
) -> Result<(), ThumbnailerError> {
	// We already created the library folders in init_thumbnail_dir, so we can just move the files
	// to their respective folders

	let thumbnails_directory = thumbnails_directory.as_ref();

	databases
		.iter()
		.map(|(library_id, db)| (*library_id, Arc::clone(db)))
		.map(|(library_id, db)| {
			let library_thumbs_dir = thumbnails_directory.join(library_id.to_string());
			let old_thumbs_dir = thumbnails_directory.to_path_buf();
			spawn(async move {
				let mut shards_to_create = HashSet::new();

				let to_move = db
					.file_path()
					.find_many(vec![file_path::cas_id::not(None)])
					.select(file_path::select!({ cas_id }))
					.exec()
					.await?
					.into_iter()
					.filter_map(|file_path| file_path.cas_id)
					.map(|cas_id| {
						let new_shard = get_shard_hex(&cas_id).to_string();
						let new_sharded_filename = format!("{new_shard}/{cas_id}.webp");
						let old_sharded_filename = format!("{}/{cas_id}.webp", &cas_id[0..2]);

						(new_shard, new_sharded_filename, old_sharded_filename)
					})
					.map(|(new_shard, new_sharded_filename, old_sharded_filename)| {
						let old = old_thumbs_dir.join(old_sharded_filename);
						let new = library_thumbs_dir.join(new_sharded_filename);
						let new_shard_dir = library_thumbs_dir.join(new_shard);

						shards_to_create.insert(new_shard_dir);

						async move {
							trace!(
								"Moving thumbnail from old location to new location: {} -> {}",
								old.display(),
								new.display()
							);

							match fs::rename(&old, new).await {
								Ok(_) => Ok(1),
								Err(e) if e.kind() == io::ErrorKind::NotFound => {
									// Thumbnail not found, it probably wasn't processed yet
									Ok(0)
								}
								Err(e) => {
									Err(ThumbnailerError::FileIO(FileIOError::from((old, e))))
								}
							}
						}
					})
					.collect::<Vec<_>>();

				let shards_created_count = shards_to_create
					.into_iter()
					.map(|path| async move {
						fs::create_dir_all(&path)
							.await
							.map_err(|e| FileIOError::from((path, e)))
					})
					.collect::<Vec<_>>()
					.try_join()
					.await?
					.len();

				let moved_count = to_move.try_join().await?.into_iter().sum::<u64>();

				info!(
					"Created {shards_created_count} shards and moved {moved_count} \
					thumbnails to library folder {library_id}"
				);

				Ok::<_, ThumbnailerError>(())
			})
		})
		.collect::<Vec<_>>()
		.try_join()
		.await?
		.into_iter()
		.collect::<Result<_, _>>()?;

	// Now that we moved all files from all databases, everything else should be ephemeral thumbnails
	// so we can just move all of them to the ephemeral directory
	let ephemeral_thumbs_dir = thumbnails_directory.join(EPHEMERAL_DIR);

	let mut shards_to_create = HashSet::new();
	let mut to_move = vec![];

	let mut read_thumbs_dir = fs::read_dir(thumbnails_directory)
		.await
		.map_err(|e| FileIOError::from((thumbnails_directory, e)))?;

	let mut empty_shards = vec![];

	while let Some(shard_entry) = read_thumbs_dir
		.next_entry()
		.await
		.map_err(|e| FileIOError::from((thumbnails_directory, e)))?
	{
		let old_shard_path = shard_entry.path();
		if shard_entry
			.file_type()
			.await
			.map_err(|e| FileIOError::from((&old_shard_path, e)))?
			.is_dir()
		{
			let mut read_shard_dir = fs::read_dir(&old_shard_path)
				.await
				.map_err(|e| FileIOError::from((&old_shard_path, e)))?;

			while let Some(thumb_entry) = read_shard_dir
				.next_entry()
				.await
				.map_err(|e| FileIOError::from((&old_shard_path, e)))?
			{
				let thumb_path = thumb_entry.path();
				if thumb_path.extension() == Some(WEBP_EXTENSION.as_ref()) {
					let thumb_filename = thumb_entry.file_name();

					let mut new_ephemeral_shard = ephemeral_thumbs_dir.join(get_shard_hex(
						thumb_filename.to_str().expect("cas_ids are utf-8"),
					));

					shards_to_create.insert(new_ephemeral_shard.clone());

					new_ephemeral_shard.push(thumb_filename);

					to_move.push(async move {
						trace!(
							"Moving thumbnail from old location to new location: {} -> {}",
							thumb_path.display(),
							new_ephemeral_shard.display()
						);

						fs::rename(&thumb_path, &new_ephemeral_shard)
							.await
							.map_err(|e| FileIOError::from((thumb_path, e)))
					});
				}
			}

			empty_shards.push(old_shard_path);
		}
	}

	shards_to_create
		.into_iter()
		.map(|path| async move {
			fs::create_dir_all(&path)
				.await
				.map_err(|e| FileIOError::from((path, e)))
		})
		.collect::<Vec<_>>()
		.try_join()
		.await?;

	let moved_shard = to_move.try_join().await?.len();

	info!("Moved {moved_shard} shards to the ephemeral directory");

	empty_shards
		.into_iter()
		.filter_map(|path| {
			path.file_name()
				.map_or(false, |name| name.len() == 2)
				.then_some(async move {
					trace!("Removing empty shard directory: {}", path.display());
					fs::remove_dir(&path)
						.await
						.map_err(|e| FileIOError::from((path, e)))
				})
		})
		.collect::<Vec<_>>()
		.try_join()
		.await?;

	Ok(())
}
