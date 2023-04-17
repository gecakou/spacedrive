use crate::{
	job::JobManager,
	location::{find_location, LocationError},
	object::{
		file_identifier::file_identifier_job::FileIdentifierJobInit,
		preview::thumbnailer_job::ThumbnailerJobInit,
		validation::validator_job::ObjectValidatorJobInit,
	},
};

use rspc::Type;
use serde::Deserialize;
use std::path::PathBuf;

use super::{utils::LibraryRequest, CoreEvent, RouterBuilder};

pub(crate) fn mount() -> RouterBuilder {
	<RouterBuilder>::new()
		.library_query("getRunning", |t| {
			t(|ctx, _: (), _| async move { Ok(ctx.jobs.get_running().await) })
		})
		.library_query("getHistory", |t| {
			t(|_, _: (), library| async move {
				JobManager::get_history(&library).await.map_err(Into::into)
			})
		})
		.library_mutation("clearAll", |t| {
			t(|_, _: (), library| async move {
				JobManager::clear_all_jobs(&library)
					.await
					.map_err(Into::into)
			})
		})
		.library_mutation("generateThumbsForLocation", |t| {
			#[derive(Type, Deserialize)]
			pub struct GenerateThumbsForLocationArgs {
				pub id: i32,
				pub path: PathBuf,
			}

			t(
				|_, args: GenerateThumbsForLocationArgs, library| async move {
					let Some(location) = find_location(&library, args.id).exec().await? else {
						return Err(LocationError::IdNotFound(args.id).into());
					};

					library
						.spawn_job(ThumbnailerJobInit {
							location,
							sub_path: Some(args.path),
							background: false,
						})
						.await
						.map_err(Into::into)
				},
			)
		})
		.library_mutation("objectValidator", |t| {
			#[derive(Type, Deserialize)]
			pub struct ObjectValidatorArgs {
				pub id: i32,
				pub path: PathBuf,
			}

			t(|_, args: ObjectValidatorArgs, library| async move {
				if find_location(&library, args.id).exec().await?.is_none() {
					return Err(LocationError::IdNotFound(args.id).into());
				}

				library
					.spawn_job(ObjectValidatorJobInit {
						location_id: args.id,
						path: args.path,
						background: true,
					})
					.await
					.map_err(Into::into)
			})
		})
		.library_mutation("identifyUniqueFiles", |t| {
			#[derive(Type, Deserialize)]
			pub struct IdentifyUniqueFilesArgs {
				pub id: i32,
				pub path: PathBuf,
			}

			t(|_, args: IdentifyUniqueFilesArgs, library| async move {
				let Some(location) = find_location(&library, args.id).exec().await? else {
					return Err(LocationError::IdNotFound(args.id).into());
				};

				library
					.spawn_job(FileIdentifierJobInit {
						location,
						sub_path: Some(args.path),
					})
					.await
					.map_err(Into::into)
			})
		})
		.library_subscription("newThumbnail", |t| {
			t(|ctx, _: (), _| {
				// TODO: Only return event for the library that was subscribed to

				let mut event_bus_rx = ctx.event_bus.0.subscribe();
				async_stream::stream! {
					while let Ok(event) = event_bus_rx.recv().await {
						match event {
							CoreEvent::NewThumbnail { cas_id } => yield cas_id,
							_ => {}
						}
					}
				}
			})
		})
}
