// File generated by Prisma Client Rust. DO NOT EDIT

use super::super::{_prisma::*, *};
use super::{SetParam, UncheckedSetParam, UniqueWhereParam, WhereParam, WithParam};
pub const NAME: &str = "video_props";
pub type Type = Option<ffmpeg_media_video_props::Data>;
pub type RecursiveSafeType = Option<Box<ffmpeg_media_video_props::Data>>;
pub fn is(value: Vec<ffmpeg_media_video_props::WhereParam>) -> WhereParam {
	WhereParam::VideoPropsIs(value)
}
pub fn is_not(value: Vec<ffmpeg_media_video_props::WhereParam>) -> WhereParam {
	WhereParam::VideoPropsIsNot(value)
}
pub struct Order(Vec<ffmpeg_media_video_props::OrderByWithRelationParam>);
pub fn order<T: From<Order>>(v: Vec<ffmpeg_media_video_props::OrderByWithRelationParam>) -> T {
	Order(v).into()
}
impl From<Order> for super::OrderByWithRelationParam {
	fn from(Order(v): Order) -> Self {
		Self::VideoProps(v)
	}
}
pub struct Fetch(pub ffmpeg_media_video_props::UniqueArgs);
impl Fetch {
	pub fn with(mut self, params: impl Into<ffmpeg_media_video_props::WithParam>) -> Self {
		self.0 = self.0.with(params.into());
		self
	}
}
impl From<Fetch> for WithParam {
	fn from(Fetch(v): Fetch) -> Self {
		WithParam::VideoProps(v)
	}
}
pub fn fetch() -> Fetch {
	Fetch(ffmpeg_media_video_props::UniqueArgs::new())
}
pub struct Connect(ffmpeg_media_video_props::UniqueWhereParam);
impl From<Connect> for SetParam {
	fn from(Connect(v): Connect) -> Self {
		Self::ConnectVideoProps(v)
	}
}
pub fn connect<T: From<Connect>>(value: ffmpeg_media_video_props::UniqueWhereParam) -> T {
	Connect(value).into()
}
pub fn disconnect() -> SetParam {
	SetParam::DisconnectVideoProps
}
pub fn is_null() -> WhereParam {
	WhereParam::VideoPropsIsNull
}
pub enum Select {
	Select(Vec<ffmpeg_media_video_props::SelectParam>),
	Include(Vec<ffmpeg_media_video_props::IncludeParam>),
	Fetch,
}
impl Into<super::SelectParam> for Select {
	fn into(self) -> super::SelectParam {
		super::SelectParam::VideoProps(self)
	}
}
impl Select {
	pub fn select(nested_selections: Vec<ffmpeg_media_video_props::SelectParam>) -> Self {
		Self::Select(nested_selections)
	}
	pub fn include(nested_selections: Vec<ffmpeg_media_video_props::IncludeParam>) -> Self {
		Self::Include(nested_selections)
	}
}
impl Into<::prisma_client_rust::Selection> for Select {
	fn into(self) -> ::prisma_client_rust::Selection {
		let selections = match self { Self :: Select (selections) => { selections . into_iter () . map (Into :: into) . collect () } , Self :: Include (selections) => { let mut nested_selections = vec ! [] ; nested_selections . extend (selections . into_iter () . map (Into :: into)) ; nested_selections } , Self :: Fetch => { < ffmpeg_media_video_props :: Types as :: prisma_client_rust :: ModelTypes > :: scalar_selections () } } ;
		::prisma_client_rust::Selection::new("video_props", None, [], selections)
	}
}
pub enum Include {
	Select(Vec<ffmpeg_media_video_props::SelectParam>),
	Include(Vec<ffmpeg_media_video_props::IncludeParam>),
	Fetch,
}
impl Into<super::IncludeParam> for Include {
	fn into(self) -> super::IncludeParam {
		super::IncludeParam::VideoProps(self)
	}
}
impl Include {
	pub fn select(nested_selections: Vec<ffmpeg_media_video_props::SelectParam>) -> Self {
		Self::Select(nested_selections)
	}
	pub fn include(nested_selections: Vec<ffmpeg_media_video_props::IncludeParam>) -> Self {
		Self::Include(nested_selections)
	}
}
impl Into<::prisma_client_rust::Selection> for Include {
	fn into(self) -> ::prisma_client_rust::Selection {
		let selections = match self { Self :: Select (selections) => { selections . into_iter () . map (Into :: into) . collect () } , Self :: Include (selections) => { let mut nested_selections = < ffmpeg_media_video_props :: Types as :: prisma_client_rust :: ModelTypes > :: scalar_selections () ; nested_selections . extend (selections . into_iter () . map (Into :: into)) ; nested_selections } , Self :: Fetch => { < ffmpeg_media_video_props :: Types as :: prisma_client_rust :: ModelTypes > :: scalar_selections () } } ;
		::prisma_client_rust::Selection::new("video_props", None, [], selections)
	}
}
