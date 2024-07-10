// File generated by Prisma Client Rust. DO NOT EDIT

use super::super::{_prisma::*, *};
use super::{SetParam, UncheckedSetParam, UniqueWhereParam, WhereParam, WithParam};
pub const NAME: &str = "timestamp";
pub type Type = BigInt;
pub type RecursiveSafeType = Type;
pub struct Equals(pub BigInt);
pub fn equals<T: From<Equals>>(value: BigInt) -> T {
	Equals(value).into()
}
impl From<Equals> for WhereParam {
	fn from(Equals(v): Equals) -> Self {
		WhereParam::Timestamp(_prisma::read_filters::BigIntFilter::Equals(v))
	}
}
::prisma_client_rust::scalar_where_param_fns!(_prisma::read_filters::BigIntFilter, Timestamp, {
	fn in_vec(_: Vec<BigInt>) -> InVec;
	fn not_in_vec(_: Vec<BigInt>) -> NotInVec;
	fn lt(_: BigInt) -> Lt;
	fn lte(_: BigInt) -> Lte;
	fn gt(_: BigInt) -> Gt;
	fn gte(_: BigInt) -> Gte;
	fn not(_: BigInt) -> Not;
});
pub struct Order(SortOrder);
pub fn order<T: From<Order>>(v: SortOrder) -> T {
	Order(v).into()
}
impl From<Order> for super::OrderByWithRelationParam {
	fn from(Order(v): Order) -> Self {
		Self::Timestamp(v)
	}
}
pub struct Set(pub BigInt);
impl From<Set> for SetParam {
	fn from(Set(v): Set) -> Self {
		Self::Timestamp(_prisma::write_params::BigIntParam::Set(v))
	}
}
pub fn set<T: From<Set>>(value: BigInt) -> T {
	Set(value).into()
}
pub struct UpdateOperation(pub _prisma::write_params::BigIntParam);
impl From<UpdateOperation> for SetParam {
	fn from(UpdateOperation(v): UpdateOperation) -> Self {
		Self::Timestamp(v)
	}
}
pub fn increment<T: From<UpdateOperation>>(value: BigInt) -> T {
	UpdateOperation(_prisma::write_params::BigIntParam::Increment(value)).into()
}
pub fn decrement<T: From<UpdateOperation>>(value: BigInt) -> T {
	UpdateOperation(_prisma::write_params::BigIntParam::Decrement(value)).into()
}
pub fn multiply<T: From<UpdateOperation>>(value: BigInt) -> T {
	UpdateOperation(_prisma::write_params::BigIntParam::Multiply(value)).into()
}
pub fn divide<T: From<UpdateOperation>>(value: BigInt) -> T {
	UpdateOperation(_prisma::write_params::BigIntParam::Divide(value)).into()
}
impl From<Set> for UncheckedSetParam {
	fn from(Set(v): Set) -> Self {
		Self::Timestamp(_prisma::write_params::BigIntParam::Set(v))
	}
}
impl From<UpdateOperation> for UncheckedSetParam {
	fn from(UpdateOperation(v): UpdateOperation) -> Self {
		Self::Timestamp(v)
	}
}
pub struct Select;
impl Into<super::SelectParam> for Select {
	fn into(self) -> super::SelectParam {
		super::SelectParam::Timestamp(self)
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
		super::IncludeParam::Timestamp(self)
	}
}
impl Into<::prisma_client_rust::Selection> for Include {
	fn into(self) -> ::prisma_client_rust::Selection {
		::prisma_client_rust::sel(NAME)
	}
}
