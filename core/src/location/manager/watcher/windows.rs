//! Windows file system event handler implementation has some caveats die to how
//! file system events are emitted on Windows.
//!
//! For example: When a file is moved to another
//! directory, we receive a remove event and then a create event, so to avoid having to actually
//! remove and create the `file_path` in the database, we have to wait some time after receiving
//! a remove event to see if a create event is emitted. If it is, we just update the `file_path`
//! in the database. If not, we remove the file from the database.

use crate::{
	invalidate_query,
	library::Library,
	location::{file_path_helper::get_inode_and_device_from_path, location_with_indexer_rules},
};

use std::{
	collections::{BTreeMap, HashMap},
	path::PathBuf,
	time::Duration,
};

use async_trait::async_trait;
use notify::{
	event::{CreateKind, ModifyKind, RenameMode},
	Event, EventKind,
};
use tokio::{fs, time::Instant};
use tracing::{error, trace};

use super::{
	utils::{
		create_dir_or_file, extract_inode_and_device_from_path, remove_by_path, rename, update_file,
	},
	EventHandler, INodeAndDevice, InstantLocationPathAndLibrary, LocationManagerError,
};

const ONE_SECOND: Duration = Duration::from_secs(1);
const HUNDRED_MILLIS: Duration = Duration::from_millis(100);

/// Windows file system event handler
#[derive(Debug)]
pub(super) struct WindowsEventHandler {
	last_check_recently_files: Instant,
	recently_created_files: BTreeMap<PathBuf, Instant>,
	last_check_rename_and_remove: Instant,
	rename_from_map: BTreeMap<INodeAndDevice, InstantLocationPathAndLibrary>,
	rename_to_map: BTreeMap<INodeAndDevice, InstantLocationPathAndLibrary>,
	to_remove_files: HashMap<INodeAndDevice, InstantLocationPathAndLibrary>,
	removal_buffer: Vec<(INodeAndDevice, InstantLocationPathAndLibrary)>,
}

#[async_trait]
impl EventHandler for WindowsEventHandler {
	fn new() -> Self
	where
		Self: Sized,
	{
		Self {
			last_check_recently_files: Instant::now(),
			recently_created_files: BTreeMap::new(),
			last_check_rename_and_remove: Instant::now(),
			rename_from_map: BTreeMap::new(),
			rename_to_map: BTreeMap::new(),
			to_remove_files: HashMap::new(),
			removal_buffer: Vec::new(),
		}
	}

	async fn handle_event(
		&mut self,
		location: location_with_indexer_rules::Data,
		library: &Library,
		event: Event,
	) -> Result<(), LocationManagerError> {
		trace!("Received Windows event: {:#?}", event);

		match event.kind {
			EventKind::Create(CreateKind::Any) => {
				let inode_and_device = get_inode_and_device_from_path(&event.paths[0]).await?;

				if let Some((_, (old_path_location, old_path, old_path_library))) =
					self.to_remove_files.remove(&inode_and_device)
				{
					// if previously we added a file to be removed with the same inode and device
					// of this "newly created" created file, it means that the file was just moved to another location
					// so we can treat if just as a file rename, like in other OSes

					// Just to make sure we're not doing anything wrong
					assert_eq!(location.id, old_path_location.id);
					assert_eq!(library.id, old_path_library.id);

					trace!(
						"Got a rename instead of remove/create: {} -> {}",
						old_path.display(),
						event.paths[0].display(),
					);

					// We found a new path for this old path, so we can rename it instead of removing and creating it
					rename(&event.paths[0], &old_path, &location, library).await?;
				} else {
					let metadata = create_dir_or_file(&location, &event, library).await?;

					if metadata.is_file() {
						let Event { mut paths, .. } = event;
						self.recently_created_files
							.insert(paths.remove(0), Instant::now());
					}
				}
			}
			EventKind::Modify(ModifyKind::Any) => {
				// Windows emite events of update right after create events
				if !self.recently_created_files.contains_key(&event.paths[0]) {
					let metadata = fs::metadata(&event.paths[0]).await?;
					if metadata.is_file() {
						update_file(&location, &event, library).await?;
					}
				}
			}
			EventKind::Modify(ModifyKind::Name(RenameMode::From)) => {
				let Event { mut paths, .. } = event;
				let path = paths.remove(0);

				let inode_and_device =
					extract_inode_and_device_from_path(&location, &path, library).await?;

				if let Some((_, (new_path_location, new_path, new_path_library))) =
					self.rename_to_map.remove(&inode_and_device)
				{
					// Just to make sure we're not doing anything wrong
					assert_eq!(location.id, new_path_location.id);
					assert_eq!(library.id, new_path_library.id);

					// We found a new path for this old path, so we can rename it
					rename(&new_path, &path, &location, library).await?;
				} else {
					self.rename_from_map.insert(
						inode_and_device,
						(Instant::now(), (location, path, library.clone())),
					);
				}
			}
			EventKind::Modify(ModifyKind::Name(RenameMode::To)) => {
				let Event { mut paths, .. } = event;
				let path = paths.remove(0);

				let inode_and_device =
					extract_inode_and_device_from_path(&location, &path, library).await?;

				if let Some((_, (old_path_location, old_path, old_path_library))) =
					self.rename_to_map.remove(&inode_and_device)
				{
					// Just to make sure we're not doing anything wrong
					assert_eq!(location.id, old_path_location.id);
					assert_eq!(library.id, old_path_library.id);

					// We found a old path for this new path, so we can rename it
					rename(&path, &old_path, &location, library).await?;
				} else {
					self.rename_from_map.insert(
						inode_and_device,
						(Instant::now(), (location, path, library.clone())),
					);
				}
			}
			EventKind::Remove(_) => {
				let Event { mut paths, .. } = event;
				let path = paths.remove(0);
				self.to_remove_files.insert(
					extract_inode_and_device_from_path(&location, &path, library).await?,
					(Instant::now(), (location, path, library.clone())),
				);
			}

			other_event_kind => {
				trace!("Other Windows event that we don't handle for now: {other_event_kind:#?}");
			}
		}

		// Cleaning out recently created files that are older than 1 second
		if self.last_check_recently_files.elapsed() > ONE_SECOND {
			self.last_check_recently_files = Instant::now();
			self.recently_created_files
				.retain(|_, created_at| created_at.elapsed() < ONE_SECOND);
		}

		if self.last_check_rename_and_remove.elapsed() > HUNDRED_MILLIS {
			self.last_check_rename_and_remove = Instant::now();
			self.rename_from_map
				.retain(|_, (created_at, (_, path, _))| {
					let to_retain = created_at.elapsed() < HUNDRED_MILLIS;
					if !to_retain {
						trace!("Removing from rename from map: {:#?}", path.display())
					}
					to_retain
				});
			self.rename_to_map.retain(|_, (created_at, (_, path, _))| {
				let to_retain = created_at.elapsed() < HUNDRED_MILLIS;
				if !to_retain {
					trace!("Removing from rename to map: {:#?}", path.display())
				}
				to_retain
			});
			handle_removes_eviction(&mut self.to_remove_files, &mut self.removal_buffer).await;
		}

		Ok(())
	}
}

async fn handle_removes_eviction(
	to_remove_files: &mut HashMap<INodeAndDevice, InstantLocationPathAndLibrary>,
	temp_buffer: &mut Vec<(INodeAndDevice, InstantLocationPathAndLibrary)>,
) {
	temp_buffer.clear();

	for (created_at, (instant, (location, path, library))) in to_remove_files.drain() {
		if instant.elapsed() > HUNDRED_MILLIS {
			if let Err(e) = remove_by_path(&location, &path, &library).await {
				error!("Failed to remove file_path: {e}");
			} else {
				trace!("Removed file_path due timeout: {}", path.display());
				invalidate_query!(&library, "locations.getExplorerData");
			}
		} else {
			temp_buffer.push((created_at, (instant, (location, path, library))));
		}
	}

	for (key, value) in temp_buffer.drain(..) {
		to_remove_files.insert(key, value);
	}
}
