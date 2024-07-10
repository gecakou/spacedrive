// File generated by Prisma Client Rust. DO NOT EDIT

pub mod key;
pub mod value;
use super::{_prisma::*, *};
pub const NAME: &str = "Preference";
#[derive(Debug, Clone)]
pub enum WhereParam {
	Not(Vec<WhereParam>),
	Or(Vec<WhereParam>),
	And(Vec<WhereParam>),
	Key(super::_prisma::read_filters::StringFilter),
	Value(super::_prisma::read_filters::BytesNullableFilter),
}
impl ::prisma_client_rust::WhereInput for WhereParam {
	fn serialize(self) -> ::prisma_client_rust::SerializedWhereInput {
		let (name, value) = match self {
			Self::Not(value) => (
				"NOT",
				::prisma_client_rust::SerializedWhereValue::Object(
					::prisma_client_rust::merge_fields(
						value
							.into_iter()
							.map(::prisma_client_rust::WhereInput::serialize)
							.map(Into::into)
							.collect(),
					),
				),
			),
			Self::Or(value) => (
				"OR",
				::prisma_client_rust::SerializedWhereValue::List(
					value
						.into_iter()
						.map(::prisma_client_rust::WhereInput::serialize)
						.map(|p| ::prisma_client_rust::PrismaValue::Object(vec![p.into()]))
						.collect(),
				),
			),
			Self::And(value) => (
				"AND",
				::prisma_client_rust::SerializedWhereValue::Object(
					::prisma_client_rust::merge_fields(
						value
							.into_iter()
							.map(::prisma_client_rust::WhereInput::serialize)
							.map(Into::into)
							.collect(),
					),
				),
			),
			Self::Key(value) => (key::NAME, value.into()),
			Self::Value(value) => (value::NAME, value.into()),
		};
		::prisma_client_rust::SerializedWhereInput::new(name.to_string(), value.into())
	}
}
#[derive(Debug, Clone)]
pub enum UniqueWhereParam {
	KeyEquals(String),
}
impl ::prisma_client_rust::WhereInput for UniqueWhereParam {
	fn serialize(self) -> ::prisma_client_rust::SerializedWhereInput {
		let (name, value) = match self {
			UniqueWhereParam::KeyEquals(value) => (
				"key",
				::prisma_client_rust::SerializedWhereValue::Value(
					::prisma_client_rust::PrismaValue::String(value),
				),
			),
		};
		::prisma_client_rust::SerializedWhereInput::new(name.to_string(), value.into())
	}
}
impl From<::prisma_client_rust::Operator<Self>> for WhereParam {
	fn from(op: ::prisma_client_rust::Operator<Self>) -> Self {
		match op {
			::prisma_client_rust::Operator::Not(value) => Self::Not(value),
			::prisma_client_rust::Operator::And(value) => Self::And(value),
			::prisma_client_rust::Operator::Or(value) => Self::Or(value),
		}
	}
}
#[derive(Debug, Clone)]
pub enum OrderByWithRelationParam {
	Key(super::SortOrder),
	Value(super::SortOrder),
}
impl Into<(String, ::prisma_client_rust::PrismaValue)> for OrderByWithRelationParam {
	fn into(self) -> (String, ::prisma_client_rust::PrismaValue) {
		let (k, v) = match self {
			Self::Key(param) => ("key", param.into()),
			Self::Value(param) => ("value", param.into()),
		};
		(k.to_string(), v)
	}
}
#[derive(Debug, Clone)]
pub enum WithParam {}
impl Into<::prisma_client_rust::Selection> for WithParam {
	fn into(self) -> ::prisma_client_rust::Selection {
		match self {}
	}
}
#[derive(Debug, Clone)]
pub enum SetParam {
	Key(super::_prisma::write_params::StringParam),
	Value(super::_prisma::write_params::BytesNullableParam),
}
impl Into<(String, ::prisma_client_rust::PrismaValue)> for SetParam {
	fn into(self) -> (String, ::prisma_client_rust::PrismaValue) {
		let (k, v) = match self {
			Self::Key(value) => (key::NAME, value.into()),
			Self::Value(value) => (value::NAME, value.into()),
		};
		(k.to_string(), v)
	}
}
#[derive(Debug, Clone)]
pub enum UncheckedSetParam {
	Key(super::_prisma::write_params::StringParam),
	Value(super::_prisma::write_params::BytesNullableParam),
}
impl Into<(String, ::prisma_client_rust::PrismaValue)> for UncheckedSetParam {
	fn into(self) -> (String, ::prisma_client_rust::PrismaValue) {
		let (k, v) = match self {
			Self::Key(value) => ("key", value.into()),
			Self::Value(value) => ("value", value.into()),
		};
		(k.to_string(), v)
	}
}
::prisma_client_rust::macros::select_factory!(
	_select_preference,
	select,
	prisma::preference,
	struct Data {
		#[serde(rename = "key")]
		key: key::Type,
		#[serde(rename = "value")]
		value: value::Type,
	},
	[(key, Scalar), (value, Scalar)]
);
pub enum SelectParam {
	Key(key::Select),
	Value(value::Select),
}
impl Into<::prisma_client_rust::Selection> for SelectParam {
	fn into(self) -> ::prisma_client_rust::Selection {
		match self {
			Self::Key(data) => data.into(),
			Self::Value(data) => data.into(),
		}
	}
}
::prisma_client_rust::macros::include_factory!(
	_include_preference,
	include,
	prisma::preference,
	struct Data {
		#[serde(rename = "key")]
		key: key::Type,
		#[serde(rename = "value")]
		value: value::Type,
	},
	[]
);
pub enum IncludeParam {
	Key(key::Include),
	Value(value::Include),
}
impl Into<::prisma_client_rust::Selection> for IncludeParam {
	fn into(self) -> ::prisma_client_rust::Selection {
		match self {
			Self::Key(data) => data.into(),
			Self::Value(data) => data.into(),
		}
	}
}
#[derive(Debug, Clone)]
pub struct Create {
	pub key: String,
	pub _params: Vec<SetParam>,
}
impl Create {
	pub fn to_query<'a>(self, client: &'a PrismaClient) -> CreateQuery<'a> {
		client.preference().create(self.key, self._params)
	}
	pub fn to_params(mut self) -> Vec<SetParam> {
		self._params.extend([key::set(self.key)]);
		self._params
	}
}
pub fn create(key: String, _params: Vec<SetParam>) -> Create {
	Create { key, _params }
}
#[derive(Debug, Clone)]
pub struct CreateUnchecked {
	pub key: String,
	pub _params: Vec<UncheckedSetParam>,
}
impl CreateUnchecked {
	pub fn to_query<'a>(self, client: &'a PrismaClient) -> CreateUncheckedQuery<'a> {
		client.preference().create_unchecked(self.key, self._params)
	}
	pub fn to_params(mut self) -> Vec<UncheckedSetParam> {
		self._params.extend([key::set(self.key)]);
		self._params
	}
}
pub fn create_unchecked(key: String, _params: Vec<UncheckedSetParam>) -> CreateUnchecked {
	CreateUnchecked { key, _params }
}
#[derive(Debug, Clone)]
pub struct Types;
impl ::prisma_client_rust::ModelTypes for Types {
	type Data = Data;
	type Where = WhereParam;
	type WhereUnique = UniqueWhereParam;
	type UncheckedSet = UncheckedSetParam;
	type Set = SetParam;
	type With = WithParam;
	type OrderBy = OrderByWithRelationParam;
	type Cursor = UniqueWhereParam;
	const MODEL: &'static str = NAME;
	fn scalar_selections() -> Vec<::prisma_client_rust::Selection> {
		vec![
			::prisma_client_rust::sel(key::NAME),
			::prisma_client_rust::sel(value::NAME),
		]
	}
}
#[derive(
	Debug,
	Clone,
	:: serde :: Serialize,
	:: serde :: Deserialize,
	:: prisma_client_rust :: specta :: Type,
)]
# [specta (rename = "Preference" , crate = prisma_client_rust :: specta)]
pub struct Data {
	#[serde(rename = "key")]
	pub key: key::Type,
	#[serde(rename = "value")]
	pub value: value::Type,
}
impl Data {}
::prisma_client_rust::macros::partial_unchecked_factory!(
	_partial_unchecked_preference,
	prisma::preference,
	struct Data {
		#[serde(rename = "key")]
		pub key: prisma::preference::key::Type,
		#[serde(rename = "value")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub value: prisma::preference::value::Type,
	}
);
::prisma_client_rust::macros::filter_factory!(
	_preference_filter,
	prisma::preference,
	[(key, Scalar), (value, Scalar)]
);
pub type UniqueArgs = ::prisma_client_rust::UniqueArgs<Types>;
pub type ManyArgs = ::prisma_client_rust::ManyArgs<Types>;
pub type CountQuery<'a> = ::prisma_client_rust::Count<'a, Types>;
pub type CreateQuery<'a> = ::prisma_client_rust::Create<'a, Types>;
pub type CreateUncheckedQuery<'a> = ::prisma_client_rust::CreateUnchecked<'a, Types>;
pub type CreateManyQuery<'a> = ::prisma_client_rust::CreateMany<'a, Types>;
pub type FindUniqueQuery<'a> = ::prisma_client_rust::FindUnique<'a, Types>;
pub type FindManyQuery<'a> = ::prisma_client_rust::FindMany<'a, Types>;
pub type FindFirstQuery<'a> = ::prisma_client_rust::FindFirst<'a, Types>;
pub type UpdateQuery<'a> = ::prisma_client_rust::Update<'a, Types>;
pub type UpdateUncheckedQuery<'a> = ::prisma_client_rust::UpdateUnchecked<'a, Types>;
pub type UpdateManyQuery<'a> = ::prisma_client_rust::UpdateMany<'a, Types>;
pub type UpsertQuery<'a> = ::prisma_client_rust::Upsert<'a, Types>;
pub type DeleteQuery<'a> = ::prisma_client_rust::Delete<'a, Types>;
pub type DeleteManyQuery<'a> = ::prisma_client_rust::DeleteMany<'a, Types>;
#[derive(Clone)]
pub struct Actions<'a> {
	pub client: &'a ::prisma_client_rust::PrismaClientInternals,
}
impl<'a> Actions<'a> {
	pub fn find_unique(self, _where: UniqueWhereParam) -> FindUniqueQuery<'a> {
		FindUniqueQuery::new(self.client, _where)
	}
	pub fn find_first(self, _where: Vec<WhereParam>) -> FindFirstQuery<'a> {
		FindFirstQuery::new(self.client, _where)
	}
	pub fn find_many(self, _where: Vec<WhereParam>) -> FindManyQuery<'a> {
		FindManyQuery::new(self.client, _where)
	}
	pub fn create(self, key: String, mut _params: Vec<SetParam>) -> CreateQuery<'a> {
		_params.extend([key::set(key)]);
		CreateQuery::new(self.client, _params)
	}
	pub fn create_unchecked(
		self,
		key: String,
		mut _params: Vec<UncheckedSetParam>,
	) -> CreateUncheckedQuery<'a> {
		_params.extend([key::set(key)]);
		CreateUncheckedQuery::new(self.client, _params.into_iter().map(Into::into).collect())
	}
	pub fn create_many(self, data: Vec<CreateUnchecked>) -> CreateManyQuery<'a> {
		let data = data.into_iter().map(CreateUnchecked::to_params).collect();
		CreateManyQuery::new(self.client, data)
	}
	pub fn update(self, _where: UniqueWhereParam, _params: Vec<SetParam>) -> UpdateQuery<'a> {
		UpdateQuery::new(self.client, _where, _params, vec![])
	}
	pub fn update_unchecked(
		self,
		_where: UniqueWhereParam,
		_params: Vec<UncheckedSetParam>,
	) -> UpdateUncheckedQuery<'a> {
		UpdateUncheckedQuery::new(
			self.client,
			_where,
			_params.into_iter().map(Into::into).collect(),
			vec![],
		)
	}
	pub fn update_many(
		self,
		_where: Vec<WhereParam>,
		_params: Vec<SetParam>,
	) -> UpdateManyQuery<'a> {
		UpdateManyQuery::new(self.client, _where, _params)
	}
	pub fn upsert(
		self,
		_where: UniqueWhereParam,
		_create: Create,
		_update: Vec<SetParam>,
	) -> UpsertQuery<'a> {
		UpsertQuery::new(self.client, _where, _create.to_params(), _update)
	}
	pub fn delete(self, _where: UniqueWhereParam) -> DeleteQuery<'a> {
		DeleteQuery::new(self.client, _where, vec![])
	}
	pub fn delete_many(self, _where: Vec<WhereParam>) -> DeleteManyQuery<'a> {
		DeleteManyQuery::new(self.client, _where)
	}
	pub fn count(self, _where: Vec<WhereParam>) -> CountQuery<'a> {
		CountQuery::new(self.client, _where)
	}
}
