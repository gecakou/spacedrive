use rspc::alpha::AlphaRouter;
use serde::Deserialize;
use specta::Type;

use serde_json::json;
use uuid::Uuid;

use crate::{
	invalidate_query,
	library::Library,
	prisma::{object, tag, tag_on_object},
	sync,
};

use super::{utils::library, Ctx, R};

pub(crate) fn mount() -> AlphaRouter<Ctx> {
	R.router()
		.procedure("list", {
			R.with2(library()).query(|(_, library), _: ()| async move {
				Ok(library.db.tag().find_many(vec![]).exec().await?)
			})
		})
		.procedure("getForObject", {
			R.with2(library())
				.query(|(_, library), object_id: i32| async move {
					Ok(library
						.db
						.tag()
						.find_many(vec![tag::tag_objects::some(vec![
							tag_on_object::object_id::equals(object_id),
						])])
						.exec()
						.await?)
				})
		})
		.procedure("get", {
			R.with2(library())
				.query(|(_, library), tag_id: i32| async move {
					Ok(library
						.db
						.tag()
						.find_unique(tag::id::equals(tag_id))
						.exec()
						.await?)
				})
		})
		// .library_mutation("create", |t| {
		// 	#[derive(Type, Deserialize)]
		// 	pub struct TagCreateArgs {
		// 		pub name: String,
		// 		pub color: String,
		// 	}
		// 	t(|_, args: TagCreateArgs, library| async move {
		// 		let created_tag = Tag::new(args.name, args.color);
		// 		created_tag.save(&library.db).await?;
		// 		invalidate_query!(library, "tags.list");
		// 		Ok(created_tag)
		// 	})
		// })
		.procedure("create", {
			#[derive(Type, Deserialize)]
			pub struct TagCreateArgs {
				pub name: String,
				pub color: String,
			}

			R.with2(library())
				.mutation(|(_, library), args: TagCreateArgs| async move {
					let Library { db, sync, .. } = &library;

					let pub_id = Uuid::new_v4().as_bytes().to_vec();

					let created_tag = sync
						.write_op(
							db,
							sync.unique_shared_create(
								sync::tag::SyncId {
									pub_id: pub_id.clone(),
								},
								[
									(tag::name::NAME, json!(args.name)),
									(tag::color::NAME, json!(args.color)),
								],
							),
							db.tag().create(
								pub_id,
								vec![
									tag::name::set(Some(args.name)),
									tag::color::set(Some(args.color)),
								],
							),
						)
						.await?;

					invalidate_query!(library, "tags.list");

					Ok(created_tag)
				})
		})
		.procedure("assign", {
			#[derive(Debug, Type, Deserialize)]
			pub struct TagAssignArgs {
				pub object_id: i32,
				pub tag_id: i32,
				pub unassign: bool,
			}

			R.with2(library())
				.mutation(|(_, library), args: TagAssignArgs| async move {
					if args.unassign {
						library
							.db
							.tag_on_object()
							.delete(tag_on_object::tag_id_object_id(args.tag_id, args.object_id))
							.exec()
							.await?;
					} else {
						library
							.db
							.tag_on_object()
							.create(
								tag::id::equals(args.tag_id),
								object::id::equals(args.object_id),
								vec![],
							)
							.exec()
							.await?;
					}

					invalidate_query!(library, "tags.getForObject");

					Ok(())
				})
		})
		.procedure("update", {
			#[derive(Type, Deserialize)]
			pub struct TagUpdateArgs {
				pub id: i32,
				pub name: Option<String>,
				pub color: Option<String>,
			}

			R.with2(library())
				.mutation(|(_, library), args: TagUpdateArgs| async move {
					let Library { sync, db, .. } = &library;

					let tag = db
						.tag()
						.find_unique(tag::id::equals(args.id))
						.select(tag::select!({ pub_id }))
						.exec()
						.await?
						.unwrap();

					sync.write_ops(
						db,
						(
							[
								args.name.as_ref().map(|v| (tag::name::NAME, json!(v))),
								args.color.as_ref().map(|v| (tag::color::NAME, json!(v))),
							]
							.into_iter()
							.flatten()
							.map(|(k, v)| {
								sync.shared_update(
									sync::tag::SyncId {
										pub_id: tag.pub_id.clone(),
									},
									k,
									v,
								)
							})
							.collect(),
							db.tag().update(
								tag::id::equals(args.id),
								vec![tag::name::set(args.name), tag::color::set(args.color)],
							),
						),
					)
					.await?;

					invalidate_query!(library, "tags.list");

					Ok(())
				})
		})
		.procedure(
			"delete",
			R.with2(library())
				.mutation(|(_, library), tag_id: i32| async move {
					library
						.db
						.tag()
						.delete(tag::id::equals(tag_id))
						.exec()
						.await?;

					invalidate_query!(library, "tags.list");

					Ok(())
				}),
		)
}
