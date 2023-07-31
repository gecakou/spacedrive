use crate::{library::LibraryName, util::MaybeUndefined, volume::get_volumes};

use chrono::Utc;
use rspc::alpha::AlphaRouter;
use sd_prisma::prisma::statistics;
use serde::Deserialize;
use specta::Type;
use tracing::debug;
use uuid::Uuid;

use super::{
	utils::{get_size, library},
	Ctx, R,
};

pub(crate) fn mount() -> AlphaRouter<Ctx> {
	R.router()
		.procedure("list", {
			R.query(
				|ctx, _: ()| async move { ctx.library_manager.get_all_libraries_config().await },
			)
		})
		.procedure("statistics", {
			R.with2(library()).query(|(_, library), _: ()| async move {
				// TODO: get from database if library is offline
				// let _statistics = library
				// 	.db
				// 	.statistics()
				// 	.find_unique(statistics::id::equals(library.node_local_id))
				// 	.exec()
				// 	.await?;

				let volumes = get_volumes().await;
				// save_volume(&library).await?;

				let mut total_capacity: u64 = 0;
				let mut available_capacity: u64 = 0;
				for volume in volumes {
					total_capacity += volume.total_capacity;
					available_capacity += volume.available_capacity;
				}

				let library_db_size = get_size(
					library
						.config()
						.data_directory()
						.join("libraries")
						.join(&format!("{}.db", library.id)),
				)
				.await
				.unwrap_or(0);

				let thumbnail_folder_size =
					get_size(library.config().data_directory().join("thumbnails"))
						.await
						.unwrap_or(0);

				use statistics::*;
				let params = vec![
					id::set(1), // Each library is a database so only one of these ever exists
					date_captured::set(Utc::now().into()),
					total_object_count::set(0),
					library_db_size::set(library_db_size.to_string()),
					total_bytes_used::set(0.to_string()),
					total_bytes_capacity::set(total_capacity.to_string()),
					total_unique_bytes::set(0.to_string()),
					total_bytes_free::set(available_capacity.to_string()),
					preview_media_bytes::set(thumbnail_folder_size.to_string()),
				];

				Ok(library
					.db
					.statistics()
					.upsert(
						statistics::id::equals(1), // Each library is a database so only one of these ever exists
						statistics::create(params.clone()),
						params,
					)
					.exec()
					.await?)
			})
		})
		.procedure("create", {
			#[derive(Deserialize, Type)]
			pub struct CreateLibraryArgs {
				name: LibraryName,
			}

			R.mutation(|ctx, args: CreateLibraryArgs| async move {
				debug!("Creating library");

				let new_library = ctx
					.library_manager
					.create(args.name, None, ctx.config.get().await)
					.await?;

				debug!("Created library {}", new_library.uuid);

				Ok(new_library)
			})
		})
		.procedure("edit", {
			#[derive(Type, Deserialize)]
			pub struct EditLibraryArgs {
				pub id: Uuid,
				pub name: Option<LibraryName>,
				pub description: MaybeUndefined<String>,
			}

			R.mutation(|ctx, args: EditLibraryArgs| async move {
				Ok(ctx
					.library_manager
					.edit(args.id, args.name, args.description)
					.await?)
			})
		})
		.procedure(
			"delete",
			R.mutation(|ctx, id: Uuid| async move {
				ctx.library_manager.delete(id).await.map_err(Into::into)
			}),
		)
}
