use rspc::{
	alpha::{
		unstable::{MwArgMapper, MwArgMapperMiddleware},
		MwV3,
	},
	ErrorCode,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

use crate::{api::Ctx, library::Library};

/// Can wrap a query argument to require it to contain a `library_id` and provide helpers for working with libraries.
#[derive(Clone, Serialize, Deserialize, Type)]
pub(crate) struct LibraryArgs<T>(pub Uuid, pub T);

pub(crate) struct LibraryArgsLike;
impl MwArgMapper for LibraryArgsLike {
	type Input<T> = LibraryArgs<T> where T: Type + DeserializeOwned + 'static;
	type State = Uuid;

	fn map<T: Serialize + DeserializeOwned + Type + 'static>(
		arg: Self::Input<T>,
	) -> (T, Self::State) {
		(arg.1, arg.0)
	}
}

pub(crate) fn library() -> impl MwV3<Ctx, NewCtx = (Ctx, Library)> {
	MwArgMapperMiddleware::<LibraryArgsLike>::new().mount(|mw, ctx: Ctx, library_id| async move {
		let library = ctx
			.library_manager
			.get_ctx(library_id)
			.await
			.ok_or_else(|| {
				rspc::Error::new(
					ErrorCode::BadRequest,
					"You must specify a valid library to use this operation.".to_string(),
				)
			})?;

		Ok(mw.next((ctx, library)))
	})
}
