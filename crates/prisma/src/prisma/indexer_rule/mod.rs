// File generated by Prisma Client Rust. DO NOT EDIT

pub mod date_created;
pub mod date_modified;
pub mod default;
pub mod id;
pub mod locations;
pub mod name;
pub mod pub_id;
pub mod rules_per_kind;
use super::{_prisma::*, *};
pub const NAME: &str = "IndexerRule";
#[derive(Debug, Clone)]
pub enum WhereParam {
	Not(Vec<WhereParam>),
	Or(Vec<WhereParam>),
	And(Vec<WhereParam>),
	Id(super::_prisma::read_filters::IntFilter),
	PubId(super::_prisma::read_filters::BytesFilter),
	Name(super::_prisma::read_filters::StringNullableFilter),
	Default(super::_prisma::read_filters::BooleanNullableFilter),
	RulesPerKind(super::_prisma::read_filters::BytesNullableFilter),
	DateCreated(super::_prisma::read_filters::DateTimeNullableFilter),
	DateModified(super::_prisma::read_filters::DateTimeNullableFilter),
	LocationsSome(Vec<super::indexer_rules_in_location::WhereParam>),
	LocationsEvery(Vec<super::indexer_rules_in_location::WhereParam>),
	LocationsNone(Vec<super::indexer_rules_in_location::WhereParam>),
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
			Self::Id(value) => (id::NAME, value.into()),
			Self::PubId(value) => (pub_id::NAME, value.into()),
			Self::Name(value) => (name::NAME, value.into()),
			Self::Default(value) => (default::NAME, value.into()),
			Self::RulesPerKind(value) => (rules_per_kind::NAME, value.into()),
			Self::DateCreated(value) => (date_created::NAME, value.into()),
			Self::DateModified(value) => (date_modified::NAME, value.into()),
			Self::LocationsSome(where_params) => (
				locations::NAME,
				::prisma_client_rust::SerializedWhereValue::Object(vec![(
					"some".to_string(),
					::prisma_client_rust::PrismaValue::Object(
						where_params
							.into_iter()
							.map(::prisma_client_rust::WhereInput::serialize)
							.map(::prisma_client_rust::SerializedWhereInput::transform_equals)
							.collect(),
					),
				)]),
			),
			Self::LocationsEvery(where_params) => (
				locations::NAME,
				::prisma_client_rust::SerializedWhereValue::Object(vec![(
					"every".to_string(),
					::prisma_client_rust::PrismaValue::Object(
						where_params
							.into_iter()
							.map(::prisma_client_rust::WhereInput::serialize)
							.map(::prisma_client_rust::SerializedWhereInput::transform_equals)
							.collect(),
					),
				)]),
			),
			Self::LocationsNone(where_params) => (
				locations::NAME,
				::prisma_client_rust::SerializedWhereValue::Object(vec![(
					"none".to_string(),
					::prisma_client_rust::PrismaValue::Object(
						where_params
							.into_iter()
							.map(::prisma_client_rust::WhereInput::serialize)
							.map(::prisma_client_rust::SerializedWhereInput::transform_equals)
							.collect(),
					),
				)]),
			),
		};
		::prisma_client_rust::SerializedWhereInput::new(name.to_string(), value.into())
	}
}
#[derive(Debug, Clone)]
pub enum UniqueWhereParam {
	PubIdEquals(Bytes),
	NameEquals(String),
	IdEquals(Int),
}
impl ::prisma_client_rust::WhereInput for UniqueWhereParam {
	fn serialize(self) -> ::prisma_client_rust::SerializedWhereInput {
		let (name, value) = match self {
			UniqueWhereParam::PubIdEquals(value) => (
				"pub_id",
				::prisma_client_rust::SerializedWhereValue::Value(
					::prisma_client_rust::PrismaValue::Bytes(value),
				),
			),
			UniqueWhereParam::NameEquals(value) => (
				"name",
				::prisma_client_rust::SerializedWhereValue::Value(
					::prisma_client_rust::PrismaValue::String(value),
				),
			),
			UniqueWhereParam::IdEquals(value) => (
				"id",
				::prisma_client_rust::SerializedWhereValue::Value(
					::prisma_client_rust::PrismaValue::Int(value),
				),
			),
		};
		::prisma_client_rust::SerializedWhereInput::new(name.to_string(), value.into())
	}
}
impl ::prisma_client_rust::FromOptionalUniqueArg<name::Equals> for WhereParam {
	type Arg = Option<String>;
	fn from_arg(arg: Self::Arg) -> Self
	where
		Self: Sized,
	{
		Self::Name(super::_prisma::read_filters::StringNullableFilter::Equals(
			arg,
		))
	}
}
impl ::prisma_client_rust::FromOptionalUniqueArg<name::Equals> for UniqueWhereParam {
	type Arg = String;
	fn from_arg(arg: Self::Arg) -> Self
	where
		Self: Sized,
	{
		Self::NameEquals(arg)
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
	Id(super::SortOrder),
	PubId(super::SortOrder),
	Name(super::SortOrder),
	Default(super::SortOrder),
	RulesPerKind(super::SortOrder),
	DateCreated(super::SortOrder),
	DateModified(super::SortOrder),
	Locations(Vec<super::indexer_rules_in_location::OrderByRelationAggregateParam>),
}
impl Into<(String, ::prisma_client_rust::PrismaValue)> for OrderByWithRelationParam {
	fn into(self) -> (String, ::prisma_client_rust::PrismaValue) {
		let (k, v) = match self {
			Self::Id(param) => ("id", param.into()),
			Self::PubId(param) => ("pub_id", param.into()),
			Self::Name(param) => ("name", param.into()),
			Self::Default(param) => ("default", param.into()),
			Self::RulesPerKind(param) => ("rules_per_kind", param.into()),
			Self::DateCreated(param) => ("date_created", param.into()),
			Self::DateModified(param) => ("date_modified", param.into()),
			Self::Locations(param) => (
				"locations",
				::prisma_client_rust::PrismaValue::Object(
					param.into_iter().map(Into::into).collect(),
				),
			),
		};
		(k.to_string(), v)
	}
}
#[derive(Debug, Clone)]
pub enum WithParam {
	Locations(super::indexer_rules_in_location::ManyArgs),
}
impl Into<::prisma_client_rust::Selection> for WithParam {
	fn into(self) -> ::prisma_client_rust::Selection {
		match self {
			Self::Locations(args) => {
				let (arguments, mut nested_selections) = args.to_graphql();
				nested_selections . extend (< super :: indexer_rules_in_location :: Types as :: prisma_client_rust :: ModelTypes > :: scalar_selections ()) ;
				::prisma_client_rust::Selection::new(
					locations::NAME,
					None,
					arguments,
					nested_selections,
				)
			}
		}
	}
}
#[derive(Debug, Clone)]
pub enum SetParam {
	Id(super::_prisma::write_params::IntParam),
	PubId(super::_prisma::write_params::BytesParam),
	Name(super::_prisma::write_params::StringNullableParam),
	Default(super::_prisma::write_params::BooleanNullableParam),
	RulesPerKind(super::_prisma::write_params::BytesNullableParam),
	DateCreated(super::_prisma::write_params::DateTimeNullableParam),
	DateModified(super::_prisma::write_params::DateTimeNullableParam),
	ConnectLocations(Vec<super::indexer_rules_in_location::UniqueWhereParam>),
	DisconnectLocations(Vec<super::indexer_rules_in_location::UniqueWhereParam>),
	SetLocations(Vec<super::indexer_rules_in_location::UniqueWhereParam>),
}
impl Into<(String, ::prisma_client_rust::PrismaValue)> for SetParam {
	fn into(self) -> (String, ::prisma_client_rust::PrismaValue) {
		let (k, v) = match self {
			Self::Id(value) => (id::NAME, value.into()),
			Self::PubId(value) => (pub_id::NAME, value.into()),
			Self::Name(value) => (name::NAME, value.into()),
			Self::Default(value) => (default::NAME, value.into()),
			Self::RulesPerKind(value) => (rules_per_kind::NAME, value.into()),
			Self::DateCreated(value) => (date_created::NAME, value.into()),
			Self::DateModified(value) => (date_modified::NAME, value.into()),
			Self::ConnectLocations(where_params) => (
				locations::NAME,
				::prisma_client_rust::PrismaValue::Object(vec![(
					"connect".to_string(),
					::prisma_client_rust::PrismaValue::List(
						where_params
							.into_iter()
							.map(::prisma_client_rust::WhereInput::serialize)
							.map(::prisma_client_rust::SerializedWhereInput::transform_equals)
							.map(|v| ::prisma_client_rust::PrismaValue::Object(vec![v]))
							.collect(),
					),
				)]),
			),
			Self::DisconnectLocations(where_params) => (
				locations::NAME,
				::prisma_client_rust::PrismaValue::Object(vec![(
					"disconnect".to_string(),
					::prisma_client_rust::PrismaValue::List(
						where_params
							.into_iter()
							.map(::prisma_client_rust::WhereInput::serialize)
							.map(::prisma_client_rust::SerializedWhereInput::transform_equals)
							.map(|v| ::prisma_client_rust::PrismaValue::Object(vec![v]))
							.collect(),
					),
				)]),
			),
			Self::SetLocations(where_params) => (
				locations::NAME,
				::prisma_client_rust::PrismaValue::Object(vec![(
					"set".to_string(),
					::prisma_client_rust::PrismaValue::List(
						where_params
							.into_iter()
							.map(::prisma_client_rust::WhereInput::serialize)
							.map(::prisma_client_rust::SerializedWhereInput::transform_equals)
							.map(|v| ::prisma_client_rust::PrismaValue::Object(vec![v]))
							.collect(),
					),
				)]),
			),
		};
		(k.to_string(), v)
	}
}
#[derive(Debug, Clone)]
pub enum UncheckedSetParam {
	Id(super::_prisma::write_params::IntParam),
	PubId(super::_prisma::write_params::BytesParam),
	Name(super::_prisma::write_params::StringNullableParam),
	Default(super::_prisma::write_params::BooleanNullableParam),
	RulesPerKind(super::_prisma::write_params::BytesNullableParam),
	DateCreated(super::_prisma::write_params::DateTimeNullableParam),
	DateModified(super::_prisma::write_params::DateTimeNullableParam),
}
impl Into<(String, ::prisma_client_rust::PrismaValue)> for UncheckedSetParam {
	fn into(self) -> (String, ::prisma_client_rust::PrismaValue) {
		let (k, v) = match self {
			Self::Id(value) => ("id", value.into()),
			Self::PubId(value) => ("pub_id", value.into()),
			Self::Name(value) => ("name", value.into()),
			Self::Default(value) => ("default", value.into()),
			Self::RulesPerKind(value) => ("rules_per_kind", value.into()),
			Self::DateCreated(value) => ("date_created", value.into()),
			Self::DateModified(value) => ("date_modified", value.into()),
		};
		(k.to_string(), v)
	}
}
::prisma_client_rust::macros::select_factory!(
	_select_indexer_rule,
	select,
	prisma::indexer_rule,
	struct Data {
		#[serde(rename = "id")]
		id: id::Type,
		#[serde(rename = "pub_id")]
		pub_id: pub_id::Type,
		#[serde(rename = "name")]
		name: name::Type,
		#[serde(rename = "default")]
		default: default::Type,
		#[serde(rename = "rules_per_kind")]
		rules_per_kind: rules_per_kind::Type,
		#[serde(rename = "date_created")]
		date_created: date_created::Type,
		#[serde(rename = "date_modified")]
		date_modified: date_modified::Type,
		#[serde(rename = "locations")]
		locations: locations::Type,
	},
	[
		(id, Scalar),
		(pub_id, Scalar),
		(name, Scalar),
		(default, Scalar),
		(rules_per_kind, Scalar),
		(date_created, Scalar),
		(date_modified, Scalar),
		(locations, Relation(prisma::indexer_rules_in_location, Many))
	]
);
pub enum SelectParam {
	Id(id::Select),
	PubId(pub_id::Select),
	Name(name::Select),
	Default(default::Select),
	RulesPerKind(rules_per_kind::Select),
	DateCreated(date_created::Select),
	DateModified(date_modified::Select),
	Locations(locations::Select),
}
impl Into<::prisma_client_rust::Selection> for SelectParam {
	fn into(self) -> ::prisma_client_rust::Selection {
		match self {
			Self::Id(data) => data.into(),
			Self::PubId(data) => data.into(),
			Self::Name(data) => data.into(),
			Self::Default(data) => data.into(),
			Self::RulesPerKind(data) => data.into(),
			Self::DateCreated(data) => data.into(),
			Self::DateModified(data) => data.into(),
			Self::Locations(data) => data.into(),
		}
	}
}
::prisma_client_rust::macros::include_factory!(
	_include_indexer_rule,
	include,
	prisma::indexer_rule,
	struct Data {
		#[serde(rename = "id")]
		id: id::Type,
		#[serde(rename = "pub_id")]
		pub_id: pub_id::Type,
		#[serde(rename = "name")]
		name: name::Type,
		#[serde(rename = "default")]
		default: default::Type,
		#[serde(rename = "rules_per_kind")]
		rules_per_kind: rules_per_kind::Type,
		#[serde(rename = "date_created")]
		date_created: date_created::Type,
		#[serde(rename = "date_modified")]
		date_modified: date_modified::Type,
		#[serde(rename = "locations")]
		locations: locations::Type,
	},
	[(locations, Relation(prisma::indexer_rules_in_location, Many))]
);
pub enum IncludeParam {
	Id(id::Include),
	PubId(pub_id::Include),
	Name(name::Include),
	Default(default::Include),
	RulesPerKind(rules_per_kind::Include),
	DateCreated(date_created::Include),
	DateModified(date_modified::Include),
	Locations(locations::Include),
}
impl Into<::prisma_client_rust::Selection> for IncludeParam {
	fn into(self) -> ::prisma_client_rust::Selection {
		match self {
			Self::Id(data) => data.into(),
			Self::PubId(data) => data.into(),
			Self::Name(data) => data.into(),
			Self::Default(data) => data.into(),
			Self::RulesPerKind(data) => data.into(),
			Self::DateCreated(data) => data.into(),
			Self::DateModified(data) => data.into(),
			Self::Locations(data) => data.into(),
		}
	}
}
#[derive(Debug, Clone)]
pub struct Create {
	pub pub_id: Bytes,
	pub _params: Vec<SetParam>,
}
impl Create {
	pub fn to_query<'a>(self, client: &'a PrismaClient) -> CreateQuery<'a> {
		client.indexer_rule().create(self.pub_id, self._params)
	}
	pub fn to_params(mut self) -> Vec<SetParam> {
		self._params.extend([pub_id::set(self.pub_id)]);
		self._params
	}
}
pub fn create(pub_id: Bytes, _params: Vec<SetParam>) -> Create {
	Create { pub_id, _params }
}
#[derive(Debug, Clone)]
pub struct CreateUnchecked {
	pub pub_id: Bytes,
	pub _params: Vec<UncheckedSetParam>,
}
impl CreateUnchecked {
	pub fn to_query<'a>(self, client: &'a PrismaClient) -> CreateUncheckedQuery<'a> {
		client
			.indexer_rule()
			.create_unchecked(self.pub_id, self._params)
	}
	pub fn to_params(mut self) -> Vec<UncheckedSetParam> {
		self._params.extend([pub_id::set(self.pub_id)]);
		self._params
	}
}
pub fn create_unchecked(pub_id: Bytes, _params: Vec<UncheckedSetParam>) -> CreateUnchecked {
	CreateUnchecked { pub_id, _params }
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
			::prisma_client_rust::sel(id::NAME),
			::prisma_client_rust::sel(pub_id::NAME),
			::prisma_client_rust::sel(name::NAME),
			::prisma_client_rust::sel(default::NAME),
			::prisma_client_rust::sel(rules_per_kind::NAME),
			::prisma_client_rust::sel(date_created::NAME),
			::prisma_client_rust::sel(date_modified::NAME),
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
# [specta (rename = "IndexerRule" , crate = prisma_client_rust :: specta)]
pub struct Data {
	#[serde(rename = "id")]
	pub id: id::Type,
	#[serde(rename = "pub_id")]
	pub pub_id: pub_id::Type,
	#[serde(rename = "name")]
	pub name: name::Type,
	#[serde(rename = "default")]
	pub default: default::Type,
	#[serde(rename = "rules_per_kind")]
	pub rules_per_kind: rules_per_kind::Type,
	#[serde(rename = "date_created")]
	pub date_created: date_created::Type,
	#[serde(rename = "date_modified")]
	pub date_modified: date_modified::Type,
	#[serde(rename = "locations")]
	#[specta(skip)]
	pub locations: Option<locations::RecursiveSafeType>,
}
impl Data {
	pub fn locations(
		&self,
	) -> Result<&locations::Type, ::prisma_client_rust::RelationNotFetchedError> {
		self.locations
			.as_ref()
			.ok_or(::prisma_client_rust::RelationNotFetchedError::new(
				stringify!(locations),
			))
	}
}
::prisma_client_rust::macros::partial_unchecked_factory!(
	_partial_unchecked_indexer_rule,
	prisma::indexer_rule,
	struct Data {
		#[serde(rename = "id")]
		pub id: prisma::indexer_rule::id::Type,
		#[serde(rename = "pub_id")]
		pub pub_id: prisma::indexer_rule::pub_id::Type,
		#[serde(rename = "name")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub name: prisma::indexer_rule::name::Type,
		#[serde(rename = "default")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub default: prisma::indexer_rule::default::Type,
		#[serde(rename = "rules_per_kind")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub rules_per_kind: prisma::indexer_rule::rules_per_kind::Type,
		#[serde(rename = "date_created")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub date_created: prisma::indexer_rule::date_created::Type,
		#[serde(rename = "date_modified")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub date_modified: prisma::indexer_rule::date_modified::Type,
	}
);
::prisma_client_rust::macros::filter_factory!(
	_indexer_rule_filter,
	prisma::indexer_rule,
	[
		(id, Scalar),
		(pub_id, Scalar),
		(name, Scalar),
		(default, Scalar),
		(rules_per_kind, Scalar),
		(date_created, Scalar),
		(date_modified, Scalar),
		(locations, Relation(prisma::indexer_rules_in_location, Many))
	]
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
	pub fn create(self, pub_id: Bytes, mut _params: Vec<SetParam>) -> CreateQuery<'a> {
		_params.extend([pub_id::set(pub_id)]);
		CreateQuery::new(self.client, _params)
	}
	pub fn create_unchecked(
		self,
		pub_id: Bytes,
		mut _params: Vec<UncheckedSetParam>,
	) -> CreateUncheckedQuery<'a> {
		_params.extend([pub_id::set(pub_id)]);
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
