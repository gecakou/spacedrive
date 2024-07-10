// File generated by Prisma Client Rust. DO NOT EDIT

use super::super::{_prisma::*, *};
use super::{SetParam, UncheckedSetParam, UniqueWhereParam, WhereParam, WithParam};
pub const NAME: &str = "streams";
pub type Type = Vec<ffmpeg_media_stream::Data>;
pub type RecursiveSafeType = Type;
pub fn some(value: Vec<ffmpeg_media_stream::WhereParam>) -> WhereParam {
	WhereParam::StreamsSome(value)
}
pub fn every(value: Vec<ffmpeg_media_stream::WhereParam>) -> WhereParam {
	WhereParam::StreamsEvery(value)
}
pub fn none(value: Vec<ffmpeg_media_stream::WhereParam>) -> WhereParam {
	WhereParam::StreamsNone(value)
}
pub struct Order(Vec<ffmpeg_media_stream::OrderByRelationAggregateParam>);
pub fn order<T: From<Order>>(v: Vec<ffmpeg_media_stream::OrderByRelationAggregateParam>) -> T {
	Order(v).into()
}
impl From<Order> for super::OrderByWithRelationParam {
	fn from(Order(v): Order) -> Self {
		Self::Streams(v)
	}
}
pub struct Fetch(pub ffmpeg_media_stream::ManyArgs);
impl Fetch {
	pub fn with(mut self, params: impl Into<ffmpeg_media_stream::WithParam>) -> Self {
		self.0 = self.0.with(params.into());
		self
	}
	pub fn order_by(mut self, param: ffmpeg_media_stream::OrderByWithRelationParam) -> Self {
		self.0 = self.0.order_by(param);
		self
	}
	pub fn skip(mut self, value: i64) -> Self {
		self.0 = self.0.skip(value);
		self
	}
	pub fn take(mut self, value: i64) -> Self {
		self.0 = self.0.take(value);
		self
	}
	pub fn cursor(mut self, value: ffmpeg_media_stream::UniqueWhereParam) -> Self {
		self.0 = self.0.cursor(value.into());
		self
	}
}
impl From<Fetch> for WithParam {
	fn from(Fetch(v): Fetch) -> Self {
		WithParam::Streams(v)
	}
}
pub fn fetch(params: Vec<ffmpeg_media_stream::WhereParam>) -> Fetch {
	Fetch(ffmpeg_media_stream::ManyArgs::new(params))
}
pub struct Connect(pub Vec<ffmpeg_media_stream::UniqueWhereParam>);
impl From<Connect> for SetParam {
	fn from(Connect(v): Connect) -> Self {
		Self::ConnectStreams(v)
	}
}
pub fn connect<T: From<Connect>>(params: Vec<ffmpeg_media_stream::UniqueWhereParam>) -> T {
	Connect(params).into()
}
pub fn disconnect(params: Vec<ffmpeg_media_stream::UniqueWhereParam>) -> SetParam {
	SetParam::DisconnectStreams(params)
}
pub fn set(params: Vec<ffmpeg_media_stream::UniqueWhereParam>) -> SetParam {
	SetParam::SetStreams(params)
}
pub enum Select {
	Select(
		ffmpeg_media_stream::ManyArgs,
		Vec<ffmpeg_media_stream::SelectParam>,
	),
	Include(
		ffmpeg_media_stream::ManyArgs,
		Vec<ffmpeg_media_stream::IncludeParam>,
	),
	Fetch(ffmpeg_media_stream::ManyArgs),
}
impl Into<super::SelectParam> for Select {
	fn into(self) -> super::SelectParam {
		super::SelectParam::Streams(self)
	}
}
impl Select {
	pub fn select(
		args: ffmpeg_media_stream::ManyArgs,
		nested_selections: Vec<ffmpeg_media_stream::SelectParam>,
	) -> Self {
		Self::Select(args, nested_selections)
	}
	pub fn include(
		args: ffmpeg_media_stream::ManyArgs,
		nested_selections: Vec<ffmpeg_media_stream::IncludeParam>,
	) -> Self {
		Self::Include(args, nested_selections)
	}
}
impl Into<::prisma_client_rust::Selection> for Select {
	fn into(self) -> ::prisma_client_rust::Selection {
		let (args, selections) = match self {
			Self::Select(args, selections) => (
				args.to_graphql().0,
				selections.into_iter().map(Into::into).collect(),
			),
			Self::Include(args, selections) => (args.to_graphql().0, {
				let mut nested_selections = vec![];
				nested_selections.extend(selections.into_iter().map(Into::into));
				nested_selections
			}),
			Self::Fetch(args) => (
				args.to_graphql().0,
				<ffmpeg_media_stream::Types as ::prisma_client_rust::ModelTypes>::scalar_selections(
				),
			),
		};
		::prisma_client_rust::Selection::new(NAME, None, args, selections)
	}
}
pub enum Include {
	Select(
		ffmpeg_media_stream::ManyArgs,
		Vec<ffmpeg_media_stream::SelectParam>,
	),
	Include(
		ffmpeg_media_stream::ManyArgs,
		Vec<ffmpeg_media_stream::IncludeParam>,
	),
	Fetch(ffmpeg_media_stream::ManyArgs),
}
impl Into<super::IncludeParam> for Include {
	fn into(self) -> super::IncludeParam {
		super::IncludeParam::Streams(self)
	}
}
impl Include {
	pub fn select(
		args: ffmpeg_media_stream::ManyArgs,
		nested_selections: Vec<ffmpeg_media_stream::SelectParam>,
	) -> Self {
		Self::Select(args, nested_selections)
	}
	pub fn include(
		args: ffmpeg_media_stream::ManyArgs,
		nested_selections: Vec<ffmpeg_media_stream::IncludeParam>,
	) -> Self {
		Self::Include(args, nested_selections)
	}
}
impl Into<::prisma_client_rust::Selection> for Include {
	fn into(self) -> ::prisma_client_rust::Selection {
		let (args, selections) = match self {
			Self::Select(args, selections) => (
				args.to_graphql().0,
				selections.into_iter().map(Into::into).collect(),
			),
			Self::Include(args, selections) => (args.to_graphql().0, {
				let mut nested_selections = < ffmpeg_media_stream :: Types as :: prisma_client_rust :: ModelTypes > :: scalar_selections () ;
				nested_selections.extend(selections.into_iter().map(Into::into));
				nested_selections
			}),
			Self::Fetch(args) => (
				args.to_graphql().0,
				<ffmpeg_media_stream::Types as ::prisma_client_rust::ModelTypes>::scalar_selections(
				),
			),
		};
		::prisma_client_rust::Selection::new(NAME, None, args, selections)
	}
}
