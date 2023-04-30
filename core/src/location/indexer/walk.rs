use crate::{
	location::file_path_helper::{FilePathMetadata, IsolatedFilePathData, MetadataExt},
	prisma::file_path,
	util::error::FileIOError,
};

#[cfg(target_family = "unix")]
use crate::location::file_path_helper::get_inode_and_device;

#[cfg(target_family = "windows")]
use crate::location::file_path_helper::get_inode_and_device_from_path;

use std::{
	collections::{HashMap, HashSet, VecDeque},
	future::Future,
	hash::{Hash, Hasher},
	path::{Path, PathBuf},
};

use tokio::fs;
use tracing::trace;

use super::{
	rules::{IndexerRule, RuleKind},
	IndexerError,
};

const TO_WALK_QUEUE_INITIAL_CAPACITY: usize = 32;
const WALKER_PATHS_BUFFER_INITIAL_CAPACITY: usize = 256;
const WALK_SINGLE_DIR_PATHS_BUFFER_INITIAL_CAPACITY: usize = 32;

/// `WalkEntry` represents a single path in the filesystem, for any comparison purposes, we only
/// consider the path itself, not the metadata.
#[derive(Debug)]
pub(super) struct WalkedEntry {
	pub(super) iso_file_path: IsolatedFilePathData<'static>,
	pub(super) metadata: FilePathMetadata,
}

pub struct ToWalkEntry {
	path: PathBuf,
	parent_dir_accepted_by_its_children: Option<bool>,
}

struct WalkingEntry {
	iso_file_path: IsolatedFilePathData<'static>,
	maybe_metadata: Option<FilePathMetadata>,
}

impl PartialEq for WalkingEntry {
	fn eq(&self, other: &Self) -> bool {
		self.iso_file_path == other.iso_file_path
	}
}

impl Eq for WalkingEntry {}

impl Hash for WalkingEntry {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.iso_file_path.hash(state);
	}
}

pub struct WalkResult<Walked, ToWalk>
where
	Walked: Iterator<Item = WalkedEntry>,
	ToWalk: Iterator<Item = ToWalkEntry>,
{
	pub walked: Walked,
	pub to_walk: ToWalk,
	pub errors: Vec<IndexerError>,
}

/// This function walks through the filesystem, applying the rules to each entry and then returning
/// a list of accepted entries. There are some useful comments in the implementation of this function
/// in case of doubts.
pub(super) async fn walk<F>(
	root: impl AsRef<Path>,
	rules_per_kind: &HashMap<RuleKind, Vec<IndexerRule>>,
	mut update_notifier: impl FnMut(&Path, usize) + '_,
	db_file_paths_fetcher: impl FnOnce(Vec<file_path::UniqueWhereParam>) -> F,
	iso_file_path_factory: impl Fn(&Path, bool) -> Result<IsolatedFilePathData<'static>, IndexerError>,
	limit: u64,
) -> Result<
	WalkResult<impl Iterator<Item = WalkedEntry>, impl Iterator<Item = ToWalkEntry>>,
	IndexerError,
>
where
	F: Future<Output = Result<Vec<file_path::Data>, IndexerError>>,
{
	let root = root.as_ref().to_path_buf();

	let mut to_walk = VecDeque::with_capacity(TO_WALK_QUEUE_INITIAL_CAPACITY);
	to_walk.push_back(ToWalkEntry {
		path: root.clone(),
		parent_dir_accepted_by_its_children: None,
	});
	let mut indexed_paths = HashSet::with_capacity(WALKER_PATHS_BUFFER_INITIAL_CAPACITY);
	let mut errors = vec![];
	let mut paths_buffer = Vec::with_capacity(WALKER_PATHS_BUFFER_INITIAL_CAPACITY);

	while let Some(entry) = to_walk.pop_front() {
		let Ok(mut read_dir) = fs::read_dir(&entry.path).await
			.map_err(|e| errors.push(FileIOError::from((entry.path.clone(), e)).into()))
			else {
			continue;
		};

		let walk_errors = inner_walk_single_dir(
			&root,
			entry,
			&mut read_dir,
			rules_per_kind,
			&mut update_notifier,
			&iso_file_path_factory,
			(&mut indexed_paths, &mut paths_buffer, Some(&mut to_walk)),
		)
		.await;
		errors.extend(walk_errors);

		if indexed_paths.len() >= limit as usize {
			break;
		}
	}

	Ok(WalkResult {
		walked: filter_existing_paths(indexed_paths, db_file_paths_fetcher).await?,
		to_walk: to_walk.into_iter(),
		errors,
	})
}

pub(super) async fn walk_single_dir<F>(
	root: impl AsRef<Path>,
	rules_per_kind: &HashMap<RuleKind, Vec<IndexerRule>>,
	mut update_notifier: impl FnMut(&Path, usize) + '_,
	db_file_paths_fetcher: impl FnOnce(Vec<file_path::UniqueWhereParam>) -> F,
	iso_file_path_factory: impl Fn(&Path, bool) -> Result<IsolatedFilePathData<'static>, IndexerError>,
) -> Result<(impl Iterator<Item = WalkedEntry>, Vec<IndexerError>), IndexerError>
where
	F: Future<Output = Result<Vec<file_path::Data>, IndexerError>>,
{
	let root = root.as_ref();

	let mut read_dir = fs::read_dir(&root)
		.await
		.map_err(|e| FileIOError::from((&root, e)))?;

	let mut indexed_paths = HashSet::with_capacity(WALK_SINGLE_DIR_PATHS_BUFFER_INITIAL_CAPACITY);
	let mut paths_buffer = Vec::with_capacity(WALK_SINGLE_DIR_PATHS_BUFFER_INITIAL_CAPACITY);

	let errors = inner_walk_single_dir(
		&root,
		ToWalkEntry {
			path: root.to_path_buf(),
			parent_dir_accepted_by_its_children: None,
		},
		&mut read_dir,
		rules_per_kind,
		&mut update_notifier,
		&iso_file_path_factory,
		(&mut indexed_paths, &mut paths_buffer, None),
	)
	.await;

	Ok((
		filter_existing_paths(indexed_paths, db_file_paths_fetcher).await?,
		errors,
	))
}

async fn filter_existing_paths<F>(
	indexed_paths: HashSet<WalkingEntry>,
	db_file_paths_fetcher: impl FnOnce(Vec<file_path::UniqueWhereParam>) -> F,
) -> Result<impl Iterator<Item = WalkedEntry>, IndexerError>
where
	F: Future<Output = Result<Vec<file_path::Data>, IndexerError>>,
{
	db_file_paths_fetcher(
		indexed_paths
			.iter()
			.map(|entry| &entry.iso_file_path)
			.map(Into::into)
			.collect(),
	)
	.await
	.map(move |file_paths| {
		let already_in_db_isolated_paths = file_paths
			.into_iter()
			.map(IsolatedFilePathData::from)
			.collect::<HashSet<_>>();

		indexed_paths.into_iter().filter_map(move |entry| {
			(!already_in_db_isolated_paths.contains(&entry.iso_file_path)).then(|| WalkedEntry {
				iso_file_path: entry.iso_file_path,
				metadata: entry
					.maybe_metadata
					.expect("we always use Some in `the inner_walk_single_dir` function"),
			})
		})
	})
}

async fn inner_walk_single_dir(
	root: impl AsRef<Path>,
	ToWalkEntry {
		path,
		parent_dir_accepted_by_its_children,
	}: ToWalkEntry,
	read_dir: &mut fs::ReadDir,
	rules_per_kind: &HashMap<RuleKind, Vec<IndexerRule>>,
	update_notifier: &mut impl FnMut(&Path, usize),
	iso_file_path_factory: &impl Fn(&Path, bool) -> Result<IsolatedFilePathData<'static>, IndexerError>,
	(already_indexed_paths, paths_buffer, mut maybe_to_walk): (
		&mut HashSet<WalkingEntry>,
		&mut Vec<WalkingEntry>,
		Option<&mut VecDeque<ToWalkEntry>>,
	),
) -> Vec<IndexerError> {
	let root = root.as_ref();
	let mut errors = vec![];

	// Just to make sure...
	paths_buffer.clear();

	let mut found_paths_counts = 0;

	// Marking with a loop label here in case of rejection or erros, to continue with next entry
	'entries: loop {
		let entry = match read_dir.next_entry().await {
			Ok(Some(entry)) => entry,
			Ok(None) => break,
			Err(e) => {
				errors.push(FileIOError::from((root, e)).into());
				continue;
			}
		};

		// Accept by children has three states,
		// None if we don't now yet or if this check doesn't apply
		// Some(true) if this check applies and it passes
		// Some(false) if this check applies and it was rejected
		// and we pass the current parent state to its children
		let mut accept_by_children_dir = parent_dir_accepted_by_its_children;

		let current_path = entry.path();

		// Just sending updates if we found more paths since the last loop
		let current_found_paths_count = paths_buffer.len();
		if found_paths_counts != current_found_paths_count {
			update_notifier(
				&current_path,
				already_indexed_paths.len() + current_found_paths_count,
			);
			found_paths_counts = current_found_paths_count;
		}

		trace!(
			"Current filesystem path: {}, accept_by_children_dir: {:#?}",
			current_path.display(),
			accept_by_children_dir
		);
		if let Some(reject_rules) = rules_per_kind.get(&RuleKind::RejectFilesByGlob) {
			for reject_rule in reject_rules {
				if !reject_rule
					.apply(&current_path)
					.await
					.expect("reject rules of this kind must be infallible")
				{
					trace!(
						"Path {} rejected by rule {}",
						current_path.display(),
						reject_rule.name
					);
					continue 'entries;
				}
			}
		}

		let Ok(metadata) = entry
			.metadata()
			.await
			.map_err(|e| errors.push(FileIOError::from((entry.path(), e)).into()))
			else {
				continue 'entries;
		};

		// TODO: Hard ignoring symlinks for now, but this should be configurable
		if metadata.is_symlink() {
			continue 'entries;
		}

		let is_dir = metadata.is_dir();

		let Ok((inode, device)) = {
			#[cfg(target_family = "unix")]
			{
				get_inode_and_device(&metadata)
			}

			#[cfg(target_family = "windows")]
			{
				get_inode_and_device_from_path(&current_path).await
			}
		}.map_err(|e| errors.push(e.into()))
		else {
			continue 'entries;
		};

		if is_dir {
			// If it is a directory, first we check if we must reject it and its children entirely
			if let Some(reject_by_children_rules) =
				rules_per_kind.get(&RuleKind::RejectIfChildrenDirectoriesArePresent)
			{
				for reject_by_children_rule in reject_by_children_rules {
					match reject_by_children_rule.apply(&current_path).await {
						Ok(false) => {
							trace!(
								"Path {} rejected by rule {}",
								current_path.display(),
								reject_by_children_rule.name
							);
							continue 'entries;
						}
						Ok(true) => {}
						Err(e) => {
							errors.push(e.into());
							continue 'entries;
						}
					}
				}
			}

			// Then we check if we must accept it and its children
			if let Some(accept_by_children_rules) =
				rules_per_kind.get(&RuleKind::AcceptIfChildrenDirectoriesArePresent)
			{
				for accept_by_children_rule in accept_by_children_rules {
					match accept_by_children_rule.apply(&current_path).await {
						Ok(true) => {
							accept_by_children_dir = Some(true);
							break;
						}
						Ok(false) => {}
						Err(e) => {
							errors.push(e.into());
						}
					}
				}

				// If it wasn't accepted then we mark as rejected
				if accept_by_children_dir.is_none() {
					trace!(
							"Path {} rejected because it didn't passed in any AcceptIfChildrenDirectoriesArePresent rule",
							current_path.display()
						);
					accept_by_children_dir = Some(false);
				}
			}

			// Then we mark this directory the be walked in too
			if let Some(ref mut to_walk) = maybe_to_walk {
				to_walk.push_back(ToWalkEntry {
					path: entry.path(),
					parent_dir_accepted_by_its_children: accept_by_children_dir,
				});
			}
		}

		let mut accept_by_glob = false;
		if let Some(accept_rules) = rules_per_kind.get(&RuleKind::AcceptFilesByGlob) {
			for accept_rule in accept_rules {
				if accept_rule
					.apply(&current_path)
					.await
					.expect("accept rules by glob must be infallible")
				{
					trace!(
						"Path {} accepted by rule {}",
						current_path.display(),
						accept_rule.name
					);
					accept_by_glob = true;
					break;
				}
			}
			if !accept_by_glob {
				trace!(
					"Path {} reject because it didn't passed in any AcceptFilesByGlob rules",
					current_path.display()
				);
				continue 'entries;
			}
		} else {
			// If there are no accept rules, then accept all paths
			accept_by_glob = true;
		}

		if accept_by_glob
			&& (accept_by_children_dir.is_none() || accept_by_children_dir.expect("<-- checked"))
		{
			let Ok(iso_file_path) = iso_file_path_factory(&current_path, is_dir)
				.map_err(|e| errors.push(e.into()))
				else {
					continue 'entries;
			};
			paths_buffer.push(WalkingEntry {
				iso_file_path,
				maybe_metadata: Some(FilePathMetadata {
					inode,
					device,
					size_in_bytes: metadata.len(),
					created_at: metadata.created_or_now().into(),
					modified_at: metadata.modified_or_now().into(),
				}),
			});

			// If the ancestors directories wasn't indexed before, now we do
			for ancestor in current_path
				.ancestors()
				.skip(1) // Skip the current directory as it was already indexed
				.take_while(|&ancestor| ancestor != root)
			{
				let Ok(iso_file_path) = iso_file_path_factory(ancestor, true)
					.map_err(|e| errors.push(e.into()))
					else {
						// Checking the next ancestor, as this one we got an error
						continue;
				};

				let mut ancestor_iso_walking_entry = WalkingEntry {
					iso_file_path,
					maybe_metadata: None,
				};
				trace!("Indexing ancestor {}", ancestor.display());
				if !already_indexed_paths.contains(&ancestor_iso_walking_entry) {
					let Ok(metadata) = fs::metadata(ancestor)
						.await
						.map_err(|e| errors.push(FileIOError::from((&root, e)).into()))
						else {
							// Checking the next ancestor, as this one we got an error
							continue;
					};
					let Ok((inode, device)) = {
						#[cfg(target_family = "unix")]
						{
							get_inode_and_device(&metadata)
						}

						#[cfg(target_family = "windows")]
						{
							get_inode_and_device_from_path(ancestor).await
						}
					}.map_err(|e| errors.push(e.into())) else {
						// Checking the next ancestor, as this one we got an error
						continue;
					};

					ancestor_iso_walking_entry.maybe_metadata = Some(FilePathMetadata {
						inode,
						device,
						size_in_bytes: metadata.len(),
						created_at: metadata.created_or_now().into(),
						modified_at: metadata.modified_or_now().into(),
					});

					paths_buffer.push(ancestor_iso_walking_entry);
				} else {
					// If indexed_paths contains the current ancestors, then it will contain
					// also all if its ancestors too, so we can stop here
					break;
				}
			}
		}
	}

	// Just merging the `found_paths` with `already_indexed_paths` here in the end to avoid possibly
	// multiple rehashes during function execution
	already_indexed_paths.extend(paths_buffer.drain(..));

	errors
}

#[cfg(test)]
mod tests {
	use super::super::rules::ParametersPerKind;
	use super::*;
	use chrono::Utc;
	use globset::{Glob, GlobSetBuilder};
	use tempfile::{tempdir, TempDir};
	use tokio::fs;
	use tracing_test::traced_test;

	impl PartialEq for WalkedEntry {
		fn eq(&self, other: &Self) -> bool {
			self.iso_file_path == other.iso_file_path
		}
	}

	impl Eq for WalkedEntry {}

	impl Hash for WalkedEntry {
		fn hash<H: Hasher>(&self, state: &mut H) {
			self.iso_file_path.hash(state);
		}
	}

	async fn prepare_location() -> TempDir {
		let root = tempdir().unwrap();
		let root_path = root.path();
		let rust_project = root_path.join("rust_project");
		let inner_project = root_path.join("inner");
		let node_project = inner_project.join("node_project");
		let photos = root_path.join("photos");

		fs::create_dir(&rust_project).await.unwrap();
		fs::create_dir(&inner_project).await.unwrap();
		fs::create_dir(&node_project).await.unwrap();
		fs::create_dir(&photos).await.unwrap();

		// Making rust and node projects a git repository
		fs::create_dir(rust_project.join(".git")).await.unwrap();
		fs::create_dir(node_project.join(".git")).await.unwrap();

		// Populating rust project
		fs::File::create(rust_project.join("Cargo.toml"))
			.await
			.unwrap();
		let rust_src_dir = rust_project.join("src");
		fs::create_dir(&rust_src_dir).await.unwrap();
		fs::File::create(rust_src_dir.join("main.rs"))
			.await
			.unwrap();
		let rust_target_dir = rust_project.join("target");
		fs::create_dir(&rust_target_dir).await.unwrap();
		let rust_build_dir = rust_target_dir.join("debug");
		fs::create_dir(&rust_build_dir).await.unwrap();
		fs::File::create(rust_build_dir.join("main")).await.unwrap();

		// Populating node project
		fs::File::create(node_project.join("package.json"))
			.await
			.unwrap();
		let node_src_dir = node_project.join("src");
		fs::create_dir(&node_src_dir).await.unwrap();
		fs::File::create(node_src_dir.join("App.tsx"))
			.await
			.unwrap();
		let node_modules = node_project.join("node_modules");
		fs::create_dir(&node_modules).await.unwrap();
		let node_modules_dep = node_modules.join("react");
		fs::create_dir(&node_modules_dep).await.unwrap();
		fs::File::create(node_modules_dep.join("package.json"))
			.await
			.unwrap();

		// Photos directory
		for photo in ["photo1.png", "photo2.jpg", "photo3.jpeg", "text.txt"].iter() {
			fs::File::create(photos.join(photo)).await.unwrap();
		}

		root
	}

	#[tokio::test]
	async fn test_walk_without_rules() {
		let root = prepare_location().await;
		let root_path = root.path();

		let metadata = FilePathMetadata {
			inode: 0,
			device: 0,
			size_in_bytes: 0,
			created_at: Utc::now(),
			modified_at: Utc::now(),
		};

		let f = |path, is_dir| IsolatedFilePathData::new(0, root_path, path, is_dir).unwrap();

		#[rustfmt::skip]
		let expected = [
			WalkedEntry { iso_file_path: f(root_path.join("rust_project"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/.git"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/Cargo.toml"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/src"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/src/main.rs"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/target"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/target/debug"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/target/debug/main"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/.git"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/package.json"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/src"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/src/App.tsx"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/node_modules"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/node_modules/react"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/node_modules/react/package.json"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("photos"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("photos/photo1.png"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("photos/photo2.jpg"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("photos/photo3.jpeg"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("photos/text.txt"), false), metadata },
		]
		.into_iter()
		.collect::<HashSet<_>>();

		let actual = walk(
			root_path.to_path_buf(),
			&HashMap::new(),
			|_, _| {},
			|_| async move { Ok(vec![]) },
			|path, is_dir| {
				IsolatedFilePathData::new(0, root_path, path, is_dir).map_err(Into::into)
			},
			420,
		)
		.await
		.unwrap()
		.walked
		.collect::<HashSet<_>>();

		assert_eq!(actual, expected);
	}

	#[tokio::test]
	#[traced_test]
	async fn test_only_photos() {
		let root = prepare_location().await;
		let root_path = root.path();

		let metadata = FilePathMetadata {
			inode: 0,
			device: 0,
			size_in_bytes: 0,
			created_at: Utc::now(),
			modified_at: Utc::now(),
		};

		let f = |path, is_dir| IsolatedFilePathData::new(0, root_path, path, is_dir).unwrap();

		#[rustfmt::skip]
		let expected = [
			WalkedEntry { iso_file_path: f(root_path.join("photos"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("photos/photo1.png"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("photos/photo2.jpg"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("photos/photo3.jpeg"), false), metadata },
		]
		.into_iter()
		.collect::<HashSet<_>>();

		let only_photos_rule = [(
			RuleKind::AcceptFilesByGlob,
			vec![IndexerRule::new(
				RuleKind::AcceptFilesByGlob,
				"only photos".to_string(),
				false,
				ParametersPerKind::AcceptFilesByGlob(
					vec![],
					GlobSetBuilder::new()
						.add(Glob::new("{*.png,*.jpg,*.jpeg}").unwrap())
						.build()
						.unwrap(),
				),
			)],
		)]
		.into_iter()
		.collect::<HashMap<_, _>>();

		let actual = walk(
			root_path.to_path_buf(),
			&only_photos_rule,
			|_, _| {},
			|_| async move { Ok(vec![]) },
			|path, is_dir| {
				IsolatedFilePathData::new(0, root_path, path, is_dir).map_err(Into::into)
			},
			420,
		)
		.await
		.unwrap()
		.walked
		.collect::<HashSet<_>>();

		assert_eq!(actual, expected);
	}

	#[tokio::test]
	#[traced_test]
	async fn test_git_repos() {
		let root = prepare_location().await;
		let root_path = root.path();

		let metadata = FilePathMetadata {
			inode: 0,
			device: 0,
			size_in_bytes: 0,
			created_at: Utc::now(),
			modified_at: Utc::now(),
		};

		let f = |path, is_dir| IsolatedFilePathData::new(0, root_path, path, is_dir).unwrap();

		#[rustfmt::skip]
		let expected = [
			WalkedEntry { iso_file_path: f(root_path.join("rust_project"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/.git"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/Cargo.toml"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/src"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/src/main.rs"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/target"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/target/debug"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/target/debug/main"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/.git"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/package.json"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/src"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/src/App.tsx"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/node_modules"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/node_modules/react"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/node_modules/react/package.json"), false), metadata },
		]
		.into_iter()
		.collect::<HashSet<_>>();

		let git_repos = [(
			RuleKind::AcceptIfChildrenDirectoriesArePresent,
			vec![IndexerRule::new(
				RuleKind::AcceptIfChildrenDirectoriesArePresent,
				"git repos".to_string(),
				false,
				ParametersPerKind::AcceptIfChildrenDirectoriesArePresent(
					[".git".to_string()].into_iter().collect(),
				),
			)],
		)]
		.into_iter()
		.collect::<HashMap<_, _>>();

		let actual = walk(
			root_path.to_path_buf(),
			&git_repos,
			|_, _| {},
			|_| async move { Ok(vec![]) },
			|path, is_dir| {
				IsolatedFilePathData::new(0, root_path, path, is_dir).map_err(Into::into)
			},
			420,
		)
		.await
		.unwrap()
		.walked
		.collect::<HashSet<_>>();

		assert_eq!(actual, expected);
	}

	#[tokio::test]
	#[traced_test]
	async fn git_repos_without_deps_or_build_dirs() {
		let root = prepare_location().await;
		let root_path = root.path();

		let metadata = FilePathMetadata {
			inode: 0,
			device: 0,
			size_in_bytes: 0,
			created_at: Utc::now(),
			modified_at: Utc::now(),
		};

		let f = |path, is_dir| IsolatedFilePathData::new(0, root_path, path, is_dir).unwrap();

		#[rustfmt::skip]
		let expected = [
			WalkedEntry { iso_file_path: f(root_path.join("rust_project"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/.git"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/Cargo.toml"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/src"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("rust_project/src/main.rs"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/.git"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/package.json"), false), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/src"), true), metadata },
			WalkedEntry { iso_file_path: f(root_path.join("inner/node_project/src/App.tsx"), false), metadata },
		]
		.into_iter()
		.collect::<HashSet<_>>();

		let git_repos_no_deps_no_build_dirs = [
			(
				RuleKind::AcceptIfChildrenDirectoriesArePresent,
				vec![IndexerRule::new(
					RuleKind::AcceptIfChildrenDirectoriesArePresent,
					"git repos".to_string(),
					false,
					ParametersPerKind::AcceptIfChildrenDirectoriesArePresent(
						[".git".to_string()].into_iter().collect(),
					),
				)],
			),
			(
				RuleKind::RejectFilesByGlob,
				vec![
					IndexerRule::new(
						RuleKind::RejectFilesByGlob,
						"reject node_modules".to_string(),
						false,
						ParametersPerKind::RejectFilesByGlob(
							vec![],
							GlobSetBuilder::new()
								.add(Glob::new("{**/node_modules/*,**/node_modules}").unwrap())
								.build()
								.unwrap(),
						),
					),
					IndexerRule::new(
						RuleKind::RejectFilesByGlob,
						"reject rust build dir".to_string(),
						false,
						ParametersPerKind::RejectFilesByGlob(
							vec![],
							GlobSetBuilder::new()
								.add(Glob::new("{**/target/*,**/target}").unwrap())
								.build()
								.unwrap(),
						),
					),
				],
			),
		]
		.into_iter()
		.collect::<HashMap<_, _>>();

		let actual = walk(
			root_path.to_path_buf(),
			&git_repos_no_deps_no_build_dirs,
			|_, _| {},
			|_| async move { Ok(vec![]) },
			|path, is_dir| {
				IsolatedFilePathData::new(0, root_path, path, is_dir).map_err(Into::into)
			},
			420,
		)
		.await
		.unwrap()
		.walked
		.collect::<HashSet<_>>();

		assert_eq!(actual, expected);
	}
}
