// File generated by Prisma Client Rust. DO NOT EDIT

use super::super::{_prisma::*, *};
use super::{SetParam, UncheckedSetParam, UniqueWhereParam, WhereParam, WithParam};
pub const NAME: &str = "sample_rate";
pub type Type = Option<Int>;
pub type RecursiveSafeType = Type;
pub struct Equals(pub Option<Int>);
pub fn equals<T: From<Equals>>(value: Option<Int>) -> T {
	Equals(value).into()
}
impl From<Equals> for WhereParam {
	fn from(Equals(v): Equals) -> Self {
		WhereParam::SampleRate(_prisma::read_filters::IntNullableFilter::Equals(v))
	}
}
::prisma_client_rust::scalar_where_param_fns!(
	_prisma::read_filters::IntNullableFilter,
	SampleRate,
	{
		fn in_vec(_: Vec<Int>) -> InVec;
		fn not_in_vec(_: Vec<Int>) -> NotInVec;
		fn lt(_: Int) -> Lt;
		fn lte(_: Int) -> Lte;
		fn gt(_: Int) -> Gt;
		fn gte(_: Int) -> Gte;
		fn not(_: Option<Int>) -> Not;
	}
);
pub struct Order(SortOrder);
pub fn order<T: From<Order>>(v: SortOrder) -> T {
	Order(v).into()
}
impl From<Order> for super::OrderByWithRelationParam {
	fn from(Order(v): Order) -> Self {
		Self::SampleRate(v)
	}
}
pub struct Set(pub Option<Int>);
impl From<Set> for SetParam {
	fn from(Set(v): Set) -> Self {
		Self::SampleRate(_prisma::write_params::IntNullableParam::Set(v))
	}
}
pub fn set<T: From<Set>>(value: Option<Int>) -> T {
	Set(value).into()
}
pub struct UpdateOperation(pub _prisma::write_params::IntNullableParam);
impl From<UpdateOperation> for SetParam {
	fn from(UpdateOperation(v): UpdateOperation) -> Self {
		Self::SampleRate(v)
	}
}
pub fn increment<T: From<UpdateOperation>>(value: Int) -> T {
	UpdateOperation(_prisma::write_params::IntNullableParam::Increment(value)).into()
}
pub fn decrement<T: From<UpdateOperation>>(value: Int) -> T {
	UpdateOperation(_prisma::write_params::IntNullableParam::Decrement(value)).into()
}
pub fn multiply<T: From<UpdateOperation>>(value: Int) -> T {
	UpdateOperation(_prisma::write_params::IntNullableParam::Multiply(value)).into()
}
pub fn divide<T: From<UpdateOperation>>(value: Int) -> T {
	UpdateOperation(_prisma::write_params::IntNullableParam::Divide(value)).into()
}
impl From<Set> for UncheckedSetParam {
	fn from(Set(v): Set) -> Self {
		Self::SampleRate(_prisma::write_params::IntNullableParam::Set(v))
	}
}
impl From<UpdateOperation> for UncheckedSetParam {
	fn from(UpdateOperation(v): UpdateOperation) -> Self {
		Self::SampleRate(v)
	}
}
pub struct Select;
impl Into<super::SelectParam> for Select {
	fn into(self) -> super::SelectParam {
		super::SelectParam::SampleRate(self)
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
		super::IncludeParam::SampleRate(self)
	}
}
impl Into<::prisma_client_rust::Selection> for Include {
	fn into(self) -> ::prisma_client_rust::Selection {
		::prisma_client_rust::sel(NAME)
	}
}
