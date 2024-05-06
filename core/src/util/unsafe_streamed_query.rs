use std::pin::pin;

use async_stream::stream;
use futures::{Stream, StreamExt};
use serde::Serialize;
use specta::{reference::Reference, DataType, Type, TypeMap};

#[derive(Serialize)]
#[serde(untagged)]
pub enum Output<T> {
	Data(T),
	Complete { __stream_complete: () },
}

impl<T: Type> Type for Output<T> {
	fn inline(type_map: &mut TypeMap, generics: &[DataType]) -> DataType {
		T::inline(type_map, generics)
	}

	fn definition(type_map: &mut TypeMap) -> DataType {
		T::definition(type_map)
	}

	fn reference(type_map: &mut TypeMap, generics: &[DataType]) -> Reference {
		T::reference(type_map, generics)
	}
}

// Marked as unsafe as the types are a lie and this should always be used with `useUnsafeStreamedQuery`
pub fn unsafe_streamed_query<S: Stream>(stream: S) -> impl Stream<Item = Output<S::Item>> {
	stream! {
		let mut stream = pin!(stream);
		while let Some(v) = stream.next().await {
			yield Output::Data(v);
		}

		yield Output::Complete { __stream_complete: () };
	}
}
