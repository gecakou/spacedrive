use std::{
	collections::{BTreeSet, HashMap},
	sync::Arc,
};

use sd_core::{
	prisma::{file_path, location},
	Node,
};
use serde::Serialize;
use specta::Type;
use tracing::error;

type NodeState<'a> = tauri::State<'a, Arc<Node>>;

#[derive(Serialize, Type)]
#[serde(tag = "t", content = "c")]
pub enum OpenFilePathResult {
	NoLibrary,
	NoFile(i32),
	OpenError(i32, String),
	AllGood(i32),
	Internal(String),
}

#[tauri::command(async)]
#[specta::specta]
pub async fn open_file_paths(
	library: uuid::Uuid,
	ids: Vec<i32>,
	node: tauri::State<'_, Arc<Node>>,
) -> Result<Vec<OpenFilePathResult>, ()> {
	let res = if let Some(library) = node.library_manager.get_library(library).await {
		library.get_file_paths(ids).await.map_or_else(
			|e| vec![OpenFilePathResult::Internal(e.to_string())],
			|paths| {
				paths
					.into_iter()
					.map(|(id, maybe_path)| {
						if let Some(path) = maybe_path {
							opener::open(path)
								.map(|_| OpenFilePathResult::AllGood(id))
								.unwrap_or_else(|e| {
									OpenFilePathResult::OpenError(id, e.to_string())
								})
						} else {
							OpenFilePathResult::NoFile(id)
						}
					})
					.collect()
			},
		)
	} else {
		vec![OpenFilePathResult::NoLibrary]
	};

	Ok(res)
}

#[derive(Serialize, Type)]
pub struct OpenWithApplication {
	id: i32,
	name: String,
	#[cfg(target_os = "linux")]
	url: std::path::PathBuf,
	#[cfg(not(target_os = "linux"))]
	url: String,
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_file_path_open_with_apps(
	library: uuid::Uuid,
	ids: Vec<i32>,
	node: NodeState<'_>,
) -> Result<Vec<OpenWithApplication>, ()> {
	let Some(library) = node.library_manager.get_library(library).await
		else {
			return Ok(vec![]);
		};

	let Ok(paths) = library
		.get_file_paths(ids).await
		.map_err(|e| {error!("{e:#?}");})
		else {
			return Ok(vec![]);
		};

	#[cfg(target_os = "macos")]
	return Ok(paths
		.into_iter()
		.flat_map(|(id, path)| {
			let Some(path) = path
				else {
					error!("File not found in database");
					return vec![];
				};

			unsafe { sd_desktop_macos::get_open_with_applications(&path.to_str().unwrap().into()) }
				.as_slice()
				.iter()
				.map(|app| OpenWithApplication {
					id,
					name: app.name.to_string(),
					url: app.url.to_string(),
				})
				.collect::<Vec<_>>()
		})
		.collect());

	#[cfg(target_os = "linux")]
	{
		use sd_desktop_linux::{DesktopEntry, HandlerType, SystemApps};

		// TODO: cache this, and only update when the underlying XDG desktop apps changes
		let Ok(system_apps) = SystemApps::populate()
			.map_err(|e| { error!("{e:#?}"); })
			else {
				return Ok(vec![]);
			};

		return Ok(paths
			.into_iter()
			.flat_map(|(id, path)| {
				let Some(path) = path
					else {
						error!("File not found in database");
						return vec![];
					};

				let Some(name) = path.file_name()
					.and_then(|name| name.to_str())
					.map(|name| name.to_string())
					else {
						error!("Failed to extract file name");
						return vec![];
					};

				system_apps
					.get_handlers(HandlerType::Ext(name))
					.map(|handler| {
						handler
							.get_path()
							.map_err(|e| {
								error!("{e:#?}");
							})
							.and_then(|path| {
								DesktopEntry::try_from(&path)
									// TODO: Ignore desktop entries that have commands that don't exist/aren't available in path
									.map(|entry| OpenWithApplication {
										id,
										name: entry.name,
										url: path,
									})
									.map_err(|e| {
										error!("{e:#?}");
									})
							})
					})
					.collect::<Result<Vec<_>, _>>()
					.unwrap_or(vec![])
			})
			.collect());
	}

	#[cfg(windows)]
	return Ok(paths
		.into_iter()
		.flat_map(|(id, path)| {
			let Some(path) = path
				else {
					error!("File not found in database");
					return vec![];
				};

			let Some(ext) = path.extension()
				else {
					error!("Failed to extract file extension");
					return vec![];
				};

			sd_desktop_windows::list_apps_associated_with_ext(ext)
				.map_err(|e| {
					error!("{e:#?}");
				})
				.map(|handlers| {
					handlers
						.iter()
						.filter_map(|handler| {
							let (Ok(name), Ok(url)) = (
							unsafe { handler.GetUIName() }.map_err(|e| { error!("{e:#?}");})
								.and_then(|name| unsafe { name.to_string() }
								.map_err(|e| { error!("{e:#?}");})),
							unsafe { handler.GetName() }.map_err(|e| { error!("{e:#?}");})
								.and_then(|name| unsafe { name.to_string() }
								.map_err(|e| { error!("{e:#?}");})),
						) else {
							error!("Failed to get handler info");
							return None
						};

							Some(OpenWithApplication { id, name, url })
						})
						.collect::<Vec<_>>()
				})
				.unwrap_or(vec![])
		})
		.collect());

	#[allow(unreachable_code)]
	Ok(vec![])
}

type FileIdAndUrl = (i32, String);

#[tauri::command(async)]
#[specta::specta]
pub async fn open_file_path_with(
	library: uuid::Uuid,
	file_ids_and_urls: Vec<FileIdAndUrl>,
	node: NodeState<'_>,
) -> Result<(), ()> {
	let Some(library) = node.library_manager.get_library(library).await
		else {
			return Err(())
		};

	let url_by_id = file_ids_and_urls.into_iter().collect::<HashMap<_, _>>();
	let ids = url_by_id.keys().copied().collect::<Vec<_>>();

	library
		.get_file_paths(ids)
		.await
		.map_err(|e| {
			error!("{e:#?}");
		})
		.and_then(|paths| {
			paths
				.iter()
				.map(|(id, path)| {
					let (Some(path), Some(url)) = (
						#[cfg(windows)]
						path.as_ref(),
						#[cfg(not(windows))]
						path.as_ref().and_then(|path| path.to_str()),
						url_by_id.get(id)
					)
						else {
							error!("File not found in database");
							return Err(());
						};

					#[cfg(target_os = "macos")]
					return {
						unsafe {
							sd_desktop_macos::open_file_path_with(
								&path.into(),
								&url.as_str().into(),
							)
						};
						Ok(())
					};

					#[cfg(target_os = "linux")]
					return sd_desktop_linux::Handler::assume_valid(url.into())
						.open(&[path])
						.map_err(|e| {
							error!("{e:#?}");
						});

					#[cfg(windows)]
					return sd_desktop_windows::open_file_path_with(path, url).map_err(|e| {
						error!("{e:#?}");
					});

					#[allow(unreachable_code)]
					Err(())
				})
				.collect::<Result<Vec<_>, _>>()
				.map(|_| ())
		})
}

#[derive(specta::Type, serde::Deserialize)]
pub enum RevealItem {
	Location { id: location::id::Type },
	FilePath { id: file_path::id::Type },
}

#[tauri::command(async)]
#[specta::specta]
pub async fn reveal_items(
	library: uuid::Uuid,
	items: Vec<RevealItem>,
	node: NodeState<'_>,
) -> Result<(), ()> {
	let Some(library) = node.library_manager.get_library(library).await
		else {
			return Err(())
		};

	let (paths, locations): (Vec<_>, Vec<_>) =
		items
			.into_iter()
			.fold((vec![], vec![]), |(mut paths, mut locations), item| {
				match item {
					RevealItem::FilePath { id } => paths.push(id),
					RevealItem::Location { id } => locations.push(id),
				}

				(paths, locations)
			});

	let mut paths_to_open = BTreeSet::new();

	if !paths.is_empty() {
		paths_to_open.extend(
			library
				.get_file_paths(paths)
				.await
				.unwrap_or_default()
				.into_values()
				.flatten(),
		);
	}

	if !locations.is_empty() {
		paths_to_open.extend(
			library
				.db
				.location()
				.find_many(vec![
					// TODO(N): This will fall apart with removable media and is making an invalid assumption that the `Node` is fixed for an `Instance`.
					location::instance_id::equals(Some(library.config.instance_id)),
					location::id::in_vec(locations),
				])
				.select(location::select!({ path }))
				.exec()
				.await
				.unwrap_or_default()
				.into_iter()
				.flat_map(|location| location.path.map(Into::into)),
		);
	}

	for path in paths_to_open {
		opener::reveal(path).ok();
	}

	Ok(())
}
