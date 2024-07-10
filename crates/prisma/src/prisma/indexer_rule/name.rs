// File generated by Prisma Client Rust. DO NOT EDIT

use super::super::{_prisma::*, *};
use super::{SetParam, UncheckedSetParam, UniqueWhereParam, WhereParam, WithParam};
pub const NAME: &str = "name";
pub type Type = Option<String>;
pub type RecursiveSafeType = Type;
pub struct Equals(pub Option<String>);
pub fn equals<T: ::prisma_client_rust::FromOptionalUniqueArg<Equals>>(value: T::Arg) -> T {
	T::from_arg(value)
}
impl From<Equals> for WhereParam {
	fn from(Equals(v): Equals) -> Self {
		WhereParam::Name(_prisma::read_filters::StringNullableFilter::Equals(v))
	}
}
::prisma_client_rust::scalar_where_param_fns!(_prisma::read_filters::StringNullableFilter, Name, {
	fn in_vec(_: Vec<String>) -> InVec;
	fn not_in_vec(_: Vec<String>) -> NotInVec;
	fn lt(_: String) -> Lt;
	fn lte(_: String) -> Lte;
	fn gt(_: String) -> Gt;
	fn gte(_: String) -> Gte;
	fn contains(_: String) -> Contains;
	fn starts_with(_: String) -> StartsWith;
	fn ends_with(_: String) -> EndsWith;
	fn not(_: Option<String>) -> Not;
});
pub struct Order(SortOrder);
pub fn order<T: From<Order>>(v: SortOrder) -> T {
	Order(v).into()
}
impl From<Order> for super::OrderByWithRelationParam {
	fn from(Order(v): Order) -> Self {
		Self::Name(v)
	}
}
pub struct Set(pub Option<String>);
impl From<Set> for SetParam {
	fn from(Set(v): Set) -> Self {
		Self::Name(_prisma::write_params::StringNullableParam::Set(v))
	}
}
pub fn set<T: From<Set>>(value: Option<String>) -> T {
	Set(value).into()
}
pub struct UpdateOperation(pub _prisma::write_params::StringNullableParam);
impl From<UpdateOperation> for SetParam {
	fn from(UpdateOperation(v): UpdateOperation) -> Self {
		Self::Name(v)
	}
}
impl From<Set> for UncheckedSetParam {
	fn from(Set(v): Set) -> Self {
		Self::Name(_prisma::write_params::StringNullableParam::Set(v))
	}
}
impl From<UpdateOperation> for UncheckedSetParam {
	fn from(UpdateOperation(v): UpdateOperation) -> Self {
		Self::Name(v)
	}
}
pub struct Select;
impl Into<super::SelectParam> for Select {
	fn into(self) -> super::SelectParam {
		super::SelectParam::Name(self)
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
		super::IncludeParam::Name(self)
	}
}
impl Into<::prisma_client_rust::Selection> for Include {
	fn into(self) -> ::prisma_client_rust::Selection {
		::prisma_client_rust::sel(NAME)
	}
}
