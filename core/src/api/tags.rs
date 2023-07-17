use chrono::Utc;
use rspc::{alpha::AlphaRouter, ErrorCode};
use serde::Deserialize;
use specta::Type;

use serde_json::json;

use crate::{
	invalidate_query,
	library::Library,
	object::tag::TagCreateArgs,
	prisma::{tag, tag_on_object},
	sync::{self, OperationFactory},
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
		.procedure("create", {
			R.with2(library())
				.mutation(|(_, library), args: TagCreateArgs| async move {
					let created_tag = args.exec(&library).await?;

					invalidate_query!(library, "tags.list");

					Ok(created_tag)
				})
		})
		.procedure("assign", {
			#[derive(Debug, Type, Deserialize)]
			pub struct TagAssignArgs {
				pub object_ids: Vec<i32>,
				pub tag_id: i32,
				pub unassign: bool,
			}

			R.with2(library())
				.mutation(|(_, library), args: TagAssignArgs| async move {
					let Library { db, .. } = &library;

					if args.unassign {
						db.tag_on_object()
							.delete_many(vec![
								tag_on_object::tag_id::equals(args.tag_id),
								tag_on_object::object_id::in_vec(args.object_ids),
							])
							.exec()
							.await?;
					} else {
						db.tag_on_object()
							.create_many(
								args.object_ids
									.iter()
									.map(|&object_id| tag_on_object::CreateUnchecked {
										tag_id: args.tag_id,
										object_id,
										_params: vec![],
									})
									.collect(),
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
						.ok_or(rspc::Error::new(
							ErrorCode::NotFound,
							"Error finding tag in db".into(),
						))?;

					db.tag()
						.update(
							tag::id::equals(args.id),
							vec![tag::date_modified::set(Some(Utc::now().into()))],
						)
						.exec()
						.await?;

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
