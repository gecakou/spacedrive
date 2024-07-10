// File generated by Prisma Client Rust. DO NOT EDIT

use super::super::{_prisma::*, *};
use super::{SetParam, UncheckedSetParam, UniqueWhereParam, WhereParam, WithParam};
pub const NAME: &str = "tag_id";
pub type Type = Int;
pub type RecursiveSafeType = Type;
pub struct Equals(pub Int);
pub fn equals<T: From<Equals>>(value: Int) -> T {
	Equals(value).into()
}
impl From<Equals> for WhereParam {
	fn from(Equals(v): Equals) -> Self {
		WhereParam::TagId(_prisma::read_filters::IntFilter::Equals(v))
	}
}
::prisma_client_rust::scalar_where_param_fns!(_prisma::read_filters::IntFilter, TagId, {
	fn in_vec(_: Vec<Int>) -> InVec;
	fn not_in_vec(_: Vec<Int>) -> NotInVec;
	fn lt(_: Int) -> Lt;
	fn lte(_: Int) -> Lte;
	fn gt(_: Int) -> Gt;
	fn gte(_: Int) -> Gte;
	fn not(_: Int) -> Not;
});
pub struct Order(SortOrder);
pub fn order<T: From<Order>>(v: SortOrder) -> T {
	Order(v).into()
}
impl From<Order> for super::OrderByWithRelationParam {
	fn from(Order(v): Order) -> Self {
		Self::TagId(v)
	}
}
pub struct Set(pub Int);
impl From<Set> for SetParam {
	fn from(Set(v): Set) -> Self {
		Self::TagId(_prisma::write_params::IntParam::Set(v))
	}
}
pub fn set<T: From<Set>>(value: Int) -> T {
	Set(value).into()
}
pub struct UpdateOperation(pub _prisma::write_params::IntParam);
impl From<UpdateOperation> for SetParam {
	fn from(UpdateOperation(v): UpdateOperation) -> Self {
		Self::TagId(v)
	}
}
pub fn increment<T: From<UpdateOperation>>(value: Int) -> T {
	UpdateOperation(_prisma::write_params::IntParam::Increment(value)).into()
}
pub fn decrement<T: From<UpdateOperation>>(value: Int) -> T {
	UpdateOperation(_prisma::write_params::IntParam::Decrement(value)).into()
}
pub fn multiply<T: From<UpdateOperation>>(value: Int) -> T {
	UpdateOperation(_prisma::write_params::IntParam::Multiply(value)).into()
}
pub fn divide<T: From<UpdateOperation>>(value: Int) -> T {
	UpdateOperation(_prisma::write_params::IntParam::Divide(value)).into()
}
impl From<Set> for UncheckedSetParam {
	fn from(Set(v): Set) -> Self {
		Self::TagId(_prisma::write_params::IntParam::Set(v))
	}
}
impl From<UpdateOperation> for UncheckedSetParam {
	fn from(UpdateOperation(v): UpdateOperation) -> Self {
		Self::TagId(v)
	}
}
pub struct Select;
impl Into<super::SelectParam> for Select {
	fn into(self) -> super::SelectParam {
		super::SelectParam::TagId(self)
	}
}
impl Into<::prisma_client_rust::Selection> for Select {
	fn into(self) -> ::prisma_client_rust::Selection {
		::prisma_client_rust::sel(NAME)
	}
}
pub struct Include;
impl Into<super::IncludeParam> for Include {
	fn into(self) -> super::IncludeParam {
		super::IncludeParam::TagId(self)
	}
}
impl Into<::prisma_client_rust::Selection> for Include {
	fn into(self) -> ::prisma_client_rust::Selection {
		::prisma_client_rust::sel(NAME)
	}
}
