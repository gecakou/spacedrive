// File generated by Prisma Client Rust. DO NOT EDIT

use super::super::{_prisma::*, *};
use super::{SetParam, UncheckedSetParam, UniqueWhereParam, WhereParam, WithParam};
pub const NAME: &str = "total_bytes_capacity";
pub type Type = String;
pub type RecursiveSafeType = Type;
pub struct Equals(pub String);
pub fn equals<T: From<Equals>>(value: String) -> T {
	Equals(value).into()
}
impl From<Equals> for WhereParam {
	fn from(Equals(v): Equals) -> Self {
		WhereParam::TotalBytesCapacity(_prisma::read_filters::StringFilter::Equals(v))
	}
}
::prisma_client_rust::scalar_where_param_fns!(
	_prisma::read_filters::StringFilter,
	TotalBytesCapacity,
	{
		fn in_vec(_: Vec<String>) -> InVec;
		fn not_in_vec(_: Vec<String>) -> NotInVec;
		fn lt(_: String) -> Lt;
		fn lte(_: String) -> Lte;
		fn gt(_: String) -> Gt;
		fn gte(_: String) -> Gte;
		fn contains(_: String) -> Contains;
		fn starts_with(_: String) -> StartsWith;
		fn ends_with(_: String) -> EndsWith;
		fn not(_: String) -> Not;
	}
);
pub struct Order(SortOrder);
pub fn order<T: From<Order>>(v: SortOrder) -> T {
	Order(v).into()
}
impl From<Order> for super::OrderByWithRelationParam {
	fn from(Order(v): Order) -> Self {
		Self::TotalBytesCapacity(v)
	}
}
pub struct Set(pub String);
impl From<Set> for SetParam {
	fn from(Set(v): Set) -> Self {
		Self::TotalBytesCapacity(_prisma::write_params::StringParam::Set(v))
	}
}
pub fn set<T: From<Set>>(value: String) -> T {
	Set(value).into()
}
pub struct UpdateOperation(pub _prisma::write_params::StringParam);
impl From<UpdateOperation> for SetParam {
	fn from(UpdateOperation(v): UpdateOperation) -> Self {
		Self::TotalBytesCapacity(v)
	}
}
impl From<Set> for UncheckedSetParam {
	fn from(Set(v): Set) -> Self {
		Self::TotalBytesCapacity(_prisma::write_params::StringParam::Set(v))
	}
}
impl From<UpdateOperation> for UncheckedSetParam {
	fn from(UpdateOperation(v): UpdateOperation) -> Self {
		Self::TotalBytesCapacity(v)
	}
}
pub struct Select;
impl Into<super::SelectParam> for Select {
	fn into(self) -> super::SelectParam {
		super::SelectParam::TotalBytesCapacity(self)
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
		super::IncludeParam::TotalBytesCapacity(self)
	}
}
impl Into<::prisma_client_rust::Selection> for Include {
	fn into(self) -> ::prisma_client_rust::Selection {
		::prisma_client_rust::sel(NAME)
	}
}
