// File generated by Prisma Client Rust. DO NOT EDIT

use super::super::{_prisma::*, *};
use super::{SetParam, UncheckedSetParam, UniqueWhereParam, WhereParam, WithParam};
pub const NAME: &str = "_count";
pub struct Order(super::SortOrder);
pub fn order<T: From<Order>>(v: super::SortOrder) -> T {
	Order(v).into()
}
impl From<Order> for super::OrderByRelationAggregateParam {
	fn from(Order(v): Order) -> Self {
		Self::_Count(v)
	}
}
