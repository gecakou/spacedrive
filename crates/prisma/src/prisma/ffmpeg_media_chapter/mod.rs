// File generated by Prisma Client Rust. DO NOT EDIT

pub mod _count;
pub mod chapter_id;
pub mod end;
pub mod ffmpeg_data;
pub mod ffmpeg_data_id;
pub mod metadata;
pub mod start;
pub mod time_base_den;
pub mod time_base_num;
pub mod title;
use super::{_prisma::*, *};
pub const NAME: &str = "FfmpegMediaChapter";
pub fn ffmpeg_data_id_chapter_id<T: From<UniqueWhereParam>>(
	ffmpeg_data_id: Int,
	chapter_id: Int,
) -> T {
	UniqueWhereParam::FfmpegDataIdChapterIdEquals(ffmpeg_data_id, chapter_id).into()
}
#[derive(Debug, Clone)]
pub enum WhereParam {
	Not(Vec<WhereParam>),
	Or(Vec<WhereParam>),
	And(Vec<WhereParam>),
	ChapterId(super::_prisma::read_filters::IntFilter),
	Start(super::_prisma::read_filters::BytesFilter),
	End(super::_prisma::read_filters::BytesFilter),
	TimeBaseDen(super::_prisma::read_filters::IntFilter),
	TimeBaseNum(super::_prisma::read_filters::IntFilter),
	Title(super::_prisma::read_filters::StringNullableFilter),
	Metadata(super::_prisma::read_filters::BytesNullableFilter),
	FfmpegDataIs(Vec<super::ffmpeg_data::WhereParam>),
	FfmpegDataIsNot(Vec<super::ffmpeg_data::WhereParam>),
	FfmpegDataId(super::_prisma::read_filters::IntFilter),
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
			Self::ChapterId(value) => (chapter_id::NAME, value.into()),
			Self::Start(value) => (start::NAME, value.into()),
			Self::End(value) => (end::NAME, value.into()),
			Self::TimeBaseDen(value) => (time_base_den::NAME, value.into()),
			Self::TimeBaseNum(value) => (time_base_num::NAME, value.into()),
			Self::Title(value) => (title::NAME, value.into()),
			Self::Metadata(value) => (metadata::NAME, value.into()),
			Self::FfmpegDataIs(where_params) => (
				ffmpeg_data::NAME,
				::prisma_client_rust::SerializedWhereValue::Object(vec![(
					"is".to_string(),
					::prisma_client_rust::PrismaValue::Object(
						where_params
							.into_iter()
							.map(::prisma_client_rust::WhereInput::serialize)
							.map(::prisma_client_rust::SerializedWhereInput::transform_equals)
							.collect(),
					),
				)]),
			),
			Self::FfmpegDataIsNot(where_params) => (
				ffmpeg_data::NAME,
				::prisma_client_rust::SerializedWhereValue::Object(vec![(
					"isNot".to_string(),
					::prisma_client_rust::PrismaValue::Object(
						where_params
							.into_iter()
							.map(::prisma_client_rust::WhereInput::serialize)
							.map(::prisma_client_rust::SerializedWhereInput::transform_equals)
							.collect(),
					),
				)]),
			),
			Self::FfmpegDataId(value) => (ffmpeg_data_id::NAME, value.into()),
		};
		::prisma_client_rust::SerializedWhereInput::new(name.to_string(), value.into())
	}
}
#[derive(Debug, Clone)]
pub enum UniqueWhereParam {
	FfmpegDataIdChapterIdEquals(Int, Int),
}
impl ::prisma_client_rust::WhereInput for UniqueWhereParam {
	fn serialize(self) -> ::prisma_client_rust::SerializedWhereInput {
		let (name, value) = match self {
			Self::FfmpegDataIdChapterIdEquals(ffmpeg_data_id, chapter_id) => (
				"ffmpeg_data_id_chapter_id",
				::prisma_client_rust::SerializedWhereValue::Object(vec![
					(
						ffmpeg_data_id::NAME.to_string(),
						::prisma_client_rust::PrismaValue::Int(ffmpeg_data_id),
					),
					(
						chapter_id::NAME.to_string(),
						::prisma_client_rust::PrismaValue::Int(chapter_id),
					),
				]),
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
	ChapterId(super::SortOrder),
	Start(super::SortOrder),
	End(super::SortOrder),
	TimeBaseDen(super::SortOrder),
	TimeBaseNum(super::SortOrder),
	Title(super::SortOrder),
	Metadata(super::SortOrder),
	FfmpegDataId(super::SortOrder),
	FfmpegData(Vec<super::ffmpeg_data::OrderByWithRelationParam>),
}
impl Into<(String, ::prisma_client_rust::PrismaValue)> for OrderByWithRelationParam {
	fn into(self) -> (String, ::prisma_client_rust::PrismaValue) {
		let (k, v) = match self {
			Self::ChapterId(param) => ("chapter_id", param.into()),
			Self::Start(param) => ("start", param.into()),
			Self::End(param) => ("end", param.into()),
			Self::TimeBaseDen(param) => ("time_base_den", param.into()),
			Self::TimeBaseNum(param) => ("time_base_num", param.into()),
			Self::Title(param) => ("title", param.into()),
			Self::Metadata(param) => ("metadata", param.into()),
			Self::FfmpegDataId(param) => ("ffmpeg_data_id", param.into()),
			Self::FfmpegData(param) => (
				"ffmpeg_data",
				::prisma_client_rust::PrismaValue::Object(
					param.into_iter().map(Into::into).collect(),
				),
			),
		};
		(k.to_string(), v)
	}
}
#[derive(Debug, Clone)]
pub enum OrderByRelationAggregateParam {
	_Count(super::SortOrder),
}
impl Into<(String, ::prisma_client_rust::PrismaValue)> for OrderByRelationAggregateParam {
	fn into(self) -> (String, ::prisma_client_rust::PrismaValue) {
		let (k, v) = match self {
			Self::_Count(param) => ("_count", param.into()),
		};
		(k.to_string(), v)
	}
}
#[derive(Debug, Clone)]
pub enum WithParam {
	FfmpegData(super::ffmpeg_data::UniqueArgs),
}
impl Into<::prisma_client_rust::Selection> for WithParam {
	fn into(self) -> ::prisma_client_rust::Selection {
		match self {
			Self::FfmpegData(args) => {
				let mut selections = < super :: ffmpeg_data :: Types as :: prisma_client_rust :: ModelTypes > :: scalar_selections () ;
				selections.extend(
					args.with_params
						.into_iter()
						.map(Into::<::prisma_client_rust::Selection>::into),
				);
				::prisma_client_rust::Selection::new(ffmpeg_data::NAME, None, [], selections)
			}
		}
	}
}
#[derive(Debug, Clone)]
pub enum SetParam {
	ChapterId(super::_prisma::write_params::IntParam),
	Start(super::_prisma::write_params::BytesParam),
	End(super::_prisma::write_params::BytesParam),
	TimeBaseDen(super::_prisma::write_params::IntParam),
	TimeBaseNum(super::_prisma::write_params::IntParam),
	Title(super::_prisma::write_params::StringNullableParam),
	Metadata(super::_prisma::write_params::BytesNullableParam),
	ConnectFfmpegData(super::ffmpeg_data::UniqueWhereParam),
	FfmpegDataId(super::_prisma::write_params::IntParam),
}
impl Into<(String, ::prisma_client_rust::PrismaValue)> for SetParam {
	fn into(self) -> (String, ::prisma_client_rust::PrismaValue) {
		let (k, v) = match self {
			Self::ChapterId(value) => (chapter_id::NAME, value.into()),
			Self::Start(value) => (start::NAME, value.into()),
			Self::End(value) => (end::NAME, value.into()),
			Self::TimeBaseDen(value) => (time_base_den::NAME, value.into()),
			Self::TimeBaseNum(value) => (time_base_num::NAME, value.into()),
			Self::Title(value) => (title::NAME, value.into()),
			Self::Metadata(value) => (metadata::NAME, value.into()),
			Self::ConnectFfmpegData(where_param) => (
				ffmpeg_data::NAME,
				::prisma_client_rust::PrismaValue::Object(vec![(
					"connect".to_string(),
					::prisma_client_rust::PrismaValue::Object(
						[where_param]
							.into_iter()
							.map(::prisma_client_rust::WhereInput::serialize)
							.map(::prisma_client_rust::SerializedWhereInput::transform_equals)
							.collect(),
					),
				)]),
			),
			Self::FfmpegDataId(value) => (ffmpeg_data_id::NAME, value.into()),
		};
		(k.to_string(), v)
	}
}
#[derive(Debug, Clone)]
pub enum UncheckedSetParam {
	ChapterId(super::_prisma::write_params::IntParam),
	Start(super::_prisma::write_params::BytesParam),
	End(super::_prisma::write_params::BytesParam),
	TimeBaseDen(super::_prisma::write_params::IntParam),
	TimeBaseNum(super::_prisma::write_params::IntParam),
	Title(super::_prisma::write_params::StringNullableParam),
	Metadata(super::_prisma::write_params::BytesNullableParam),
	FfmpegDataId(super::_prisma::write_params::IntParam),
}
impl Into<(String, ::prisma_client_rust::PrismaValue)> for UncheckedSetParam {
	fn into(self) -> (String, ::prisma_client_rust::PrismaValue) {
		let (k, v) = match self {
			Self::ChapterId(value) => ("chapter_id", value.into()),
			Self::Start(value) => ("start", value.into()),
			Self::End(value) => ("end", value.into()),
			Self::TimeBaseDen(value) => ("time_base_den", value.into()),
			Self::TimeBaseNum(value) => ("time_base_num", value.into()),
			Self::Title(value) => ("title", value.into()),
			Self::Metadata(value) => ("metadata", value.into()),
			Self::FfmpegDataId(value) => ("ffmpeg_data_id", value.into()),
		};
		(k.to_string(), v)
	}
}
::prisma_client_rust::macros::select_factory!(
	_select_ffmpeg_media_chapter,
	select,
	prisma::ffmpeg_media_chapter,
	struct Data {
		#[serde(rename = "chapter_id")]
		chapter_id: chapter_id::Type,
		#[serde(rename = "start")]
		start: start::Type,
		#[serde(rename = "end")]
		end: end::Type,
		#[serde(rename = "time_base_den")]
		time_base_den: time_base_den::Type,
		#[serde(rename = "time_base_num")]
		time_base_num: time_base_num::Type,
		#[serde(rename = "title")]
		title: title::Type,
		#[serde(rename = "metadata")]
		metadata: metadata::Type,
		#[serde(rename = "ffmpeg_data")]
		ffmpeg_data: ffmpeg_data::Type,
		#[serde(rename = "ffmpeg_data_id")]
		ffmpeg_data_id: ffmpeg_data_id::Type,
	},
	[
		(chapter_id, Scalar),
		(start, Scalar),
		(end, Scalar),
		(time_base_den, Scalar),
		(time_base_num, Scalar),
		(title, Scalar),
		(metadata, Scalar),
		(ffmpeg_data, Relation(prisma::ffmpeg_data, One)),
		(ffmpeg_data_id, Scalar)
	]
);
pub enum SelectParam {
	ChapterId(chapter_id::Select),
	Start(start::Select),
	End(end::Select),
	TimeBaseDen(time_base_den::Select),
	TimeBaseNum(time_base_num::Select),
	Title(title::Select),
	Metadata(metadata::Select),
	FfmpegData(ffmpeg_data::Select),
	FfmpegDataId(ffmpeg_data_id::Select),
}
impl Into<::prisma_client_rust::Selection> for SelectParam {
	fn into(self) -> ::prisma_client_rust::Selection {
		match self {
			Self::ChapterId(data) => data.into(),
			Self::Start(data) => data.into(),
			Self::End(data) => data.into(),
			Self::TimeBaseDen(data) => data.into(),
			Self::TimeBaseNum(data) => data.into(),
			Self::Title(data) => data.into(),
			Self::Metadata(data) => data.into(),
			Self::FfmpegData(data) => data.into(),
			Self::FfmpegDataId(data) => data.into(),
		}
	}
}
::prisma_client_rust::macros::include_factory!(
	_include_ffmpeg_media_chapter,
	include,
	prisma::ffmpeg_media_chapter,
	struct Data {
		#[serde(rename = "chapter_id")]
		chapter_id: chapter_id::Type,
		#[serde(rename = "start")]
		start: start::Type,
		#[serde(rename = "end")]
		end: end::Type,
		#[serde(rename = "time_base_den")]
		time_base_den: time_base_den::Type,
		#[serde(rename = "time_base_num")]
		time_base_num: time_base_num::Type,
		#[serde(rename = "title")]
		title: title::Type,
		#[serde(rename = "metadata")]
		metadata: metadata::Type,
		#[serde(rename = "ffmpeg_data")]
		ffmpeg_data: ffmpeg_data::Type,
		#[serde(rename = "ffmpeg_data_id")]
		ffmpeg_data_id: ffmpeg_data_id::Type,
	},
	[(ffmpeg_data, Relation(prisma::ffmpeg_data, One))]
);
pub enum IncludeParam {
	ChapterId(chapter_id::Include),
	Start(start::Include),
	End(end::Include),
	TimeBaseDen(time_base_den::Include),
	TimeBaseNum(time_base_num::Include),
	Title(title::Include),
	Metadata(metadata::Include),
	FfmpegData(ffmpeg_data::Include),
	FfmpegDataId(ffmpeg_data_id::Include),
}
impl Into<::prisma_client_rust::Selection> for IncludeParam {
	fn into(self) -> ::prisma_client_rust::Selection {
		match self {
			Self::ChapterId(data) => data.into(),
			Self::Start(data) => data.into(),
			Self::End(data) => data.into(),
			Self::TimeBaseDen(data) => data.into(),
			Self::TimeBaseNum(data) => data.into(),
			Self::Title(data) => data.into(),
			Self::Metadata(data) => data.into(),
			Self::FfmpegData(data) => data.into(),
			Self::FfmpegDataId(data) => data.into(),
		}
	}
}
#[derive(Debug, Clone)]
pub struct Create {
	pub chapter_id: Int,
	pub start: Bytes,
	pub end: Bytes,
	pub time_base_den: Int,
	pub time_base_num: Int,
	pub ffmpeg_data: super::ffmpeg_data::UniqueWhereParam,
	pub _params: Vec<SetParam>,
}
impl Create {
	pub fn to_query<'a>(self, client: &'a PrismaClient) -> CreateQuery<'a> {
		client.ffmpeg_media_chapter().create(
			self.chapter_id,
			self.start,
			self.end,
			self.time_base_den,
			self.time_base_num,
			self.ffmpeg_data,
			self._params,
		)
	}
	pub fn to_params(mut self) -> Vec<SetParam> {
		self._params.extend([
			chapter_id::set(self.chapter_id),
			start::set(self.start),
			end::set(self.end),
			time_base_den::set(self.time_base_den),
			time_base_num::set(self.time_base_num),
			ffmpeg_data::connect(self.ffmpeg_data),
		]);
		self._params
	}
}
pub fn create(
	chapter_id: Int,
	start: Bytes,
	end: Bytes,
	time_base_den: Int,
	time_base_num: Int,
	ffmpeg_data: super::ffmpeg_data::UniqueWhereParam,
	_params: Vec<SetParam>,
) -> Create {
	Create {
		chapter_id,
		start,
		end,
		time_base_den,
		time_base_num,
		ffmpeg_data,
		_params,
	}
}
#[derive(Debug, Clone)]
pub struct CreateUnchecked {
	pub chapter_id: Int,
	pub start: Bytes,
	pub end: Bytes,
	pub time_base_den: Int,
	pub time_base_num: Int,
	pub ffmpeg_data_id: Int,
	pub _params: Vec<UncheckedSetParam>,
}
impl CreateUnchecked {
	pub fn to_query<'a>(self, client: &'a PrismaClient) -> CreateUncheckedQuery<'a> {
		client.ffmpeg_media_chapter().create_unchecked(
			self.chapter_id,
			self.start,
			self.end,
			self.time_base_den,
			self.time_base_num,
			self.ffmpeg_data_id,
			self._params,
		)
	}
	pub fn to_params(mut self) -> Vec<UncheckedSetParam> {
		self._params.extend([
			chapter_id::set(self.chapter_id),
			start::set(self.start),
			end::set(self.end),
			time_base_den::set(self.time_base_den),
			time_base_num::set(self.time_base_num),
			ffmpeg_data_id::set(self.ffmpeg_data_id),
		]);
		self._params
	}
}
pub fn create_unchecked(
	chapter_id: Int,
	start: Bytes,
	end: Bytes,
	time_base_den: Int,
	time_base_num: Int,
	ffmpeg_data_id: Int,
	_params: Vec<UncheckedSetParam>,
) -> CreateUnchecked {
	CreateUnchecked {
		chapter_id,
		start,
		end,
		time_base_den,
		time_base_num,
		ffmpeg_data_id,
		_params,
	}
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
			::prisma_client_rust::sel(chapter_id::NAME),
			::prisma_client_rust::sel(start::NAME),
			::prisma_client_rust::sel(end::NAME),
			::prisma_client_rust::sel(time_base_den::NAME),
			::prisma_client_rust::sel(time_base_num::NAME),
			::prisma_client_rust::sel(title::NAME),
			::prisma_client_rust::sel(metadata::NAME),
			::prisma_client_rust::sel(ffmpeg_data_id::NAME),
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
# [specta (rename = "FfmpegMediaChapter" , crate = prisma_client_rust :: specta)]
pub struct Data {
	#[serde(rename = "chapter_id")]
	pub chapter_id: chapter_id::Type,
	#[serde(rename = "start")]
	pub start: start::Type,
	#[serde(rename = "end")]
	pub end: end::Type,
	#[serde(rename = "time_base_den")]
	pub time_base_den: time_base_den::Type,
	#[serde(rename = "time_base_num")]
	pub time_base_num: time_base_num::Type,
	#[serde(rename = "title")]
	pub title: title::Type,
	#[serde(rename = "metadata")]
	pub metadata: metadata::Type,
	#[serde(rename = "ffmpeg_data")]
	#[specta(skip)]
	pub ffmpeg_data: Option<ffmpeg_data::RecursiveSafeType>,
	#[serde(rename = "ffmpeg_data_id")]
	pub ffmpeg_data_id: ffmpeg_data_id::Type,
}
impl Data {
	pub fn ffmpeg_data(
		&self,
	) -> Result<&super::ffmpeg_data::Data, ::prisma_client_rust::RelationNotFetchedError> {
		self.ffmpeg_data
			.as_ref()
			.ok_or(::prisma_client_rust::RelationNotFetchedError::new(
				stringify!(ffmpeg_data),
			))
			.map(|v| v.as_ref())
	}
}
::prisma_client_rust::macros::partial_unchecked_factory!(
	_partial_unchecked_ffmpeg_media_chapter,
	prisma::ffmpeg_media_chapter,
	struct Data {
		#[serde(rename = "chapter_id")]
		pub chapter_id: prisma::ffmpeg_media_chapter::chapter_id::Type,
		#[serde(rename = "start")]
		pub start: prisma::ffmpeg_media_chapter::start::Type,
		#[serde(rename = "end")]
		pub end: prisma::ffmpeg_media_chapter::end::Type,
		#[serde(rename = "time_base_den")]
		pub time_base_den: prisma::ffmpeg_media_chapter::time_base_den::Type,
		#[serde(rename = "time_base_num")]
		pub time_base_num: prisma::ffmpeg_media_chapter::time_base_num::Type,
		#[serde(rename = "title")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub title: prisma::ffmpeg_media_chapter::title::Type,
		#[serde(rename = "metadata")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub metadata: prisma::ffmpeg_media_chapter::metadata::Type,
		#[serde(rename = "ffmpeg_data_id")]
		pub ffmpeg_data_id: prisma::ffmpeg_media_chapter::ffmpeg_data_id::Type,
	}
);
::prisma_client_rust::macros::filter_factory!(
	_ffmpeg_media_chapter_filter,
	prisma::ffmpeg_media_chapter,
	[
		(chapter_id, Scalar),
		(start, Scalar),
		(end, Scalar),
		(time_base_den, Scalar),
		(time_base_num, Scalar),
		(title, Scalar),
		(metadata, Scalar),
		(ffmpeg_data, Relation(prisma::ffmpeg_data, One)),
		(ffmpeg_data_id, Scalar)
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
	pub fn create(
		self,
		chapter_id: Int,
		start: Bytes,
		end: Bytes,
		time_base_den: Int,
		time_base_num: Int,
		ffmpeg_data: super::ffmpeg_data::UniqueWhereParam,
		mut _params: Vec<SetParam>,
	) -> CreateQuery<'a> {
		_params.extend([
			chapter_id::set(chapter_id),
			start::set(start),
			end::set(end),
			time_base_den::set(time_base_den),
			time_base_num::set(time_base_num),
			ffmpeg_data::connect(ffmpeg_data),
		]);
		CreateQuery::new(self.client, _params)
	}
	pub fn create_unchecked(
		self,
		chapter_id: Int,
		start: Bytes,
		end: Bytes,
		time_base_den: Int,
		time_base_num: Int,
		ffmpeg_data_id: Int,
		mut _params: Vec<UncheckedSetParam>,
	) -> CreateUncheckedQuery<'a> {
		_params.extend([
			chapter_id::set(chapter_id),
			start::set(start),
			end::set(end),
			time_base_den::set(time_base_den),
			time_base_num::set(time_base_num),
			ffmpeg_data_id::set(ffmpeg_data_id),
		]);
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
