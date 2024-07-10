// File generated by Prisma Client Rust. DO NOT EDIT

use super::super::{_prisma::*, *};
use super::{SetParam, UncheckedSetParam, UniqueWhereParam, WhereParam, WithParam};
pub const NAME: &str = "children";
pub type Type = Vec<job::Data>;
pub type RecursiveSafeType = Type;
pub fn some(value: Vec<job::WhereParam>) -> WhereParam {
	WhereParam::ChildrenSome(value)
}
pub fn every(value: Vec<job::WhereParam>) -> WhereParam {
	WhereParam::ChildrenEvery(value)
}
pub fn none(value: Vec<job::WhereParam>) -> WhereParam {
	WhereParam::ChildrenNone(value)
}
pub struct Order(Vec<job::OrderByRelationAggregateParam>);
pub fn order<T: From<Order>>(v: Vec<job::OrderByRelationAggregateParam>) -> T {
	Order(v).into()
}
impl From<Order> for super::OrderByWithRelationParam {
	fn from(Order(v): Order) -> Self {
		Self::Children(v)
	}
}
pub struct Fetch(pub job::ManyArgs);
impl Fetch {
	pub fn with(mut self, params: impl Into<job::WithParam>) -> Self {
		self.0 = self.0.with(params.into());
		self
	}
	pub fn order_by(mut self, param: job::OrderByWithRelationParam) -> Self {
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
	pub fn cursor(mut self, value: job::UniqueWhereParam) -> Self {
		self.0 = self.0.cursor(value.into());
		self
	}
}
impl From<Fetch> for WithParam {
	fn from(Fetch(v): Fetch) -> Self {
		WithParam::Children(v)
	}
}
pub fn fetch(params: Vec<job::WhereParam>) -> Fetch {
	Fetch(job::ManyArgs::new(params))
}
pub struct Connect(pub Vec<job::UniqueWhereParam>);
impl From<Connect> for SetParam {
	fn from(Connect(v): Connect) -> Self {
		Self::ConnectChildren(v)
	}
}
pub fn connect<T: From<Connect>>(params: Vec<job::UniqueWhereParam>) -> T {
	Connect(params).into()
}
pub fn disconnect(params: Vec<job::UniqueWhereParam>) -> SetParam {
	SetParam::DisconnectChildren(params)
}
pub fn set(params: Vec<job::UniqueWhereParam>) -> SetParam {
	SetParam::SetChildren(params)
}
pub enum Select {
	Select(job::ManyArgs, Vec<job::SelectParam>),
	Include(job::ManyArgs, Vec<job::IncludeParam>),
	Fetch(job::ManyArgs),
}
impl Into<super::SelectParam> for Select {
	fn into(self) -> super::SelectParam {
		super::SelectParam::Children(self)
	}
}
impl Select {
	pub fn select(args: job::ManyArgs, nested_selections: Vec<job::SelectParam>) -> Self {
		Self::Select(args, nested_selections)
	}
	pub fn include(args: job::ManyArgs, nested_selections: Vec<job::IncludeParam>) -> Self {
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
				<job::Types as ::prisma_client_rust::ModelTypes>::scalar_selections(),
			),
		};
		::prisma_client_rust::Selection::new(NAME, None, args, selections)
	}
}
pub enum Include {
	Select(job::ManyArgs, Vec<job::SelectParam>),
	Include(job::ManyArgs, Vec<job::IncludeParam>),
	Fetch(job::ManyArgs),
}
impl Into<super::IncludeParam> for Include {
	fn into(self) -> super::IncludeParam {
		super::IncludeParam::Children(self)
	}
}
impl Include {
	pub fn select(args: job::ManyArgs, nested_selections: Vec<job::SelectParam>) -> Self {
		Self::Select(args, nested_selections)
	}
	pub fn include(args: job::ManyArgs, nested_selections: Vec<job::IncludeParam>) -> Self {
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
				let mut nested_selections =
					<job::Types as ::prisma_client_rust::ModelTypes>::scalar_selections();
				nested_selections.extend(selections.into_iter().map(Into::into));
				nested_selections
			}),
			Self::Fetch(args) => (
				args.to_graphql().0,
				<job::Types as ::prisma_client_rust::ModelTypes>::scalar_selections(),
			),
		};
		::prisma_client_rust::Selection::new(NAME, None, args, selections)
	}
}
