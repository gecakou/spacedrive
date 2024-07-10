// File generated by Prisma Client Rust. DO NOT EDIT

use super::super::{_prisma::*, *};
use super::{SetParam, UncheckedSetParam, UniqueWhereParam, WhereParam, WithParam};
pub const NAME: &str = "exif_data";
pub type Type = Option<exif_data::Data>;
pub type RecursiveSafeType = Option<Box<exif_data::Data>>;
pub fn is(value: Vec<exif_data::WhereParam>) -> WhereParam {
	WhereParam::ExifDataIs(value)
}
pub fn is_not(value: Vec<exif_data::WhereParam>) -> WhereParam {
	WhereParam::ExifDataIsNot(value)
}
pub struct Order(Vec<exif_data::OrderByWithRelationParam>);
pub fn order<T: From<Order>>(v: Vec<exif_data::OrderByWithRelationParam>) -> T {
	Order(v).into()
}
impl From<Order> for super::OrderByWithRelationParam {
	fn from(Order(v): Order) -> Self {
		Self::ExifData(v)
	}
}
pub struct Fetch(pub exif_data::UniqueArgs);
impl Fetch {
	pub fn with(mut self, params: impl Into<exif_data::WithParam>) -> Self {
		self.0 = self.0.with(params.into());
		self
	}
}
impl From<Fetch> for WithParam {
	fn from(Fetch(v): Fetch) -> Self {
		WithParam::ExifData(v)
	}
}
pub fn fetch() -> Fetch {
	Fetch(exif_data::UniqueArgs::new())
}
pub struct Connect(exif_data::UniqueWhereParam);
impl From<Connect> for SetParam {
	fn from(Connect(v): Connect) -> Self {
		Self::ConnectExifData(v)
	}
}
pub fn connect<T: From<Connect>>(value: exif_data::UniqueWhereParam) -> T {
	Connect(value).into()
}
pub fn disconnect() -> SetParam {
	SetParam::DisconnectExifData
}
pub fn is_null() -> WhereParam {
	WhereParam::ExifDataIsNull
}
pub enum Select {
	Select(Vec<exif_data::SelectParam>),
	Include(Vec<exif_data::IncludeParam>),
	Fetch,
}
impl Into<super::SelectParam> for Select {
	fn into(self) -> super::SelectParam {
		super::SelectParam::ExifData(self)
	}
}
impl Select {
	pub fn select(nested_selections: Vec<exif_data::SelectParam>) -> Self {
		Self::Select(nested_selections)
	}
	pub fn include(nested_selections: Vec<exif_data::IncludeParam>) -> Self {
		Self::Include(nested_selections)
	}
}
impl Into<::prisma_client_rust::Selection> for Select {
	fn into(self) -> ::prisma_client_rust::Selection {
		let selections = match self {
			Self::Select(selections) => selections.into_iter().map(Into::into).collect(),
			Self::Include(selections) => {
				let mut nested_selections = vec![];
				nested_selections.extend(selections.into_iter().map(Into::into));
				nested_selections
			}
			Self::Fetch => {
				<exif_data::Types as ::prisma_client_rust::ModelTypes>::scalar_selections()
			}
		};
		::prisma_client_rust::Selection::new("exif_data", None, [], selections)
	}
}
pub enum Include {
	Select(Vec<exif_data::SelectParam>),
	Include(Vec<exif_data::IncludeParam>),
	Fetch,
}
impl Into<super::IncludeParam> for Include {
	fn into(self) -> super::IncludeParam {
		super::IncludeParam::ExifData(self)
	}
}
impl Include {
	pub fn select(nested_selections: Vec<exif_data::SelectParam>) -> Self {
		Self::Select(nested_selections)
	}
	pub fn include(nested_selections: Vec<exif_data::IncludeParam>) -> Self {
		Self::Include(nested_selections)
	}
}
impl Into<::prisma_client_rust::Selection> for Include {
	fn into(self) -> ::prisma_client_rust::Selection {
		let selections = match self {
			Self::Select(selections) => selections.into_iter().map(Into::into).collect(),
			Self::Include(selections) => {
				let mut nested_selections =
					<exif_data::Types as ::prisma_client_rust::ModelTypes>::scalar_selections();
				nested_selections.extend(selections.into_iter().map(Into::into));
				nested_selections
			}
			Self::Fetch => {
				<exif_data::Types as ::prisma_client_rust::ModelTypes>::scalar_selections()
			}
		};
		::prisma_client_rust::Selection::new("exif_data", None, [], selections)
	}
}
