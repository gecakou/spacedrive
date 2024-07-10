// File generated by Prisma Client Rust. DO NOT EDIT

pub mod _count;
pub mod aspect_ratio_den;
pub mod aspect_ratio_num;
pub mod codec;
pub mod dispositions;
pub mod duration;
pub mod encoder;
pub mod ffmpeg_data_id;
pub mod frames_per_second_den;
pub mod frames_per_second_num;
pub mod language;
pub mod metadata;
pub mod name;
pub mod program;
pub mod program_id;
pub mod stream_id;
pub mod time_base_real_den;
pub mod time_base_real_num;
pub mod title;
use super::{_prisma::*, *};
pub const NAME: &str = "FfmpegMediaStream";
pub fn ffmpeg_data_id_program_id_stream_id<T: From<UniqueWhereParam>>(
	ffmpeg_data_id: Int,
	program_id: Int,
	stream_id: Int,
) -> T {
	UniqueWhereParam::FfmpegDataIdProgramIdStreamIdEquals(ffmpeg_data_id, program_id, stream_id)
		.into()
}
#[derive(Debug, Clone)]
pub enum WhereParam {
	Not(Vec<WhereParam>),
	Or(Vec<WhereParam>),
	And(Vec<WhereParam>),
	StreamId(super::_prisma::read_filters::IntFilter),
	Name(super::_prisma::read_filters::StringNullableFilter),
	CodecIsNull,
	CodecIs(Vec<super::ffmpeg_media_codec::WhereParam>),
	CodecIsNot(Vec<super::ffmpeg_media_codec::WhereParam>),
	AspectRatioNum(super::_prisma::read_filters::IntFilter),
	AspectRatioDen(super::_prisma::read_filters::IntFilter),
	FramesPerSecondNum(super::_prisma::read_filters::IntFilter),
	FramesPerSecondDen(super::_prisma::read_filters::IntFilter),
	TimeBaseRealDen(super::_prisma::read_filters::IntFilter),
	TimeBaseRealNum(super::_prisma::read_filters::IntFilter),
	Dispositions(super::_prisma::read_filters::StringNullableFilter),
	Title(super::_prisma::read_filters::StringNullableFilter),
	Encoder(super::_prisma::read_filters::StringNullableFilter),
	Language(super::_prisma::read_filters::StringNullableFilter),
	Duration(super::_prisma::read_filters::BytesNullableFilter),
	Metadata(super::_prisma::read_filters::BytesNullableFilter),
	ProgramIs(Vec<super::ffmpeg_media_program::WhereParam>),
	ProgramIsNot(Vec<super::ffmpeg_media_program::WhereParam>),
	ProgramId(super::_prisma::read_filters::IntFilter),
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
			Self::StreamId(value) => (stream_id::NAME, value.into()),
			Self::Name(value) => (name::NAME, value.into()),
			Self::CodecIsNull => (
				codec::NAME,
				::prisma_client_rust::SerializedWhereValue::Value(
					::prisma_client_rust::PrismaValue::Null,
				),
			),
			Self::CodecIs(where_params) => (
				codec::NAME,
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
			Self::CodecIsNot(where_params) => (
				codec::NAME,
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
			Self::AspectRatioNum(value) => (aspect_ratio_num::NAME, value.into()),
			Self::AspectRatioDen(value) => (aspect_ratio_den::NAME, value.into()),
			Self::FramesPerSecondNum(value) => (frames_per_second_num::NAME, value.into()),
			Self::FramesPerSecondDen(value) => (frames_per_second_den::NAME, value.into()),
			Self::TimeBaseRealDen(value) => (time_base_real_den::NAME, value.into()),
			Self::TimeBaseRealNum(value) => (time_base_real_num::NAME, value.into()),
			Self::Dispositions(value) => (dispositions::NAME, value.into()),
			Self::Title(value) => (title::NAME, value.into()),
			Self::Encoder(value) => (encoder::NAME, value.into()),
			Self::Language(value) => (language::NAME, value.into()),
			Self::Duration(value) => (duration::NAME, value.into()),
			Self::Metadata(value) => (metadata::NAME, value.into()),
			Self::ProgramIs(where_params) => (
				program::NAME,
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
			Self::ProgramIsNot(where_params) => (
				program::NAME,
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
			Self::ProgramId(value) => (program_id::NAME, value.into()),
			Self::FfmpegDataId(value) => (ffmpeg_data_id::NAME, value.into()),
		};
		::prisma_client_rust::SerializedWhereInput::new(name.to_string(), value.into())
	}
}
#[derive(Debug, Clone)]
pub enum UniqueWhereParam {
	FfmpegDataIdProgramIdStreamIdEquals(Int, Int, Int),
}
impl ::prisma_client_rust::WhereInput for UniqueWhereParam {
	fn serialize(self) -> ::prisma_client_rust::SerializedWhereInput {
		let (name, value) = match self {
			Self::FfmpegDataIdProgramIdStreamIdEquals(ffmpeg_data_id, program_id, stream_id) => (
				"ffmpeg_data_id_program_id_stream_id",
				::prisma_client_rust::SerializedWhereValue::Object(vec![
					(
						ffmpeg_data_id::NAME.to_string(),
						::prisma_client_rust::PrismaValue::Int(ffmpeg_data_id),
					),
					(
						program_id::NAME.to_string(),
						::prisma_client_rust::PrismaValue::Int(program_id),
					),
					(
						stream_id::NAME.to_string(),
						::prisma_client_rust::PrismaValue::Int(stream_id),
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
	StreamId(super::SortOrder),
	Name(super::SortOrder),
	AspectRatioNum(super::SortOrder),
	AspectRatioDen(super::SortOrder),
	FramesPerSecondNum(super::SortOrder),
	FramesPerSecondDen(super::SortOrder),
	TimeBaseRealDen(super::SortOrder),
	TimeBaseRealNum(super::SortOrder),
	Dispositions(super::SortOrder),
	Title(super::SortOrder),
	Encoder(super::SortOrder),
	Language(super::SortOrder),
	Duration(super::SortOrder),
	Metadata(super::SortOrder),
	ProgramId(super::SortOrder),
	FfmpegDataId(super::SortOrder),
	Codec(Vec<super::ffmpeg_media_codec::OrderByWithRelationParam>),
	Program(Vec<super::ffmpeg_media_program::OrderByWithRelationParam>),
}
impl Into<(String, ::prisma_client_rust::PrismaValue)> for OrderByWithRelationParam {
	fn into(self) -> (String, ::prisma_client_rust::PrismaValue) {
		let (k, v) = match self {
			Self::StreamId(param) => ("stream_id", param.into()),
			Self::Name(param) => ("name", param.into()),
			Self::AspectRatioNum(param) => ("aspect_ratio_num", param.into()),
			Self::AspectRatioDen(param) => ("aspect_ratio_den", param.into()),
			Self::FramesPerSecondNum(param) => ("frames_per_second_num", param.into()),
			Self::FramesPerSecondDen(param) => ("frames_per_second_den", param.into()),
			Self::TimeBaseRealDen(param) => ("time_base_real_den", param.into()),
			Self::TimeBaseRealNum(param) => ("time_base_real_num", param.into()),
			Self::Dispositions(param) => ("dispositions", param.into()),
			Self::Title(param) => ("title", param.into()),
			Self::Encoder(param) => ("encoder", param.into()),
			Self::Language(param) => ("language", param.into()),
			Self::Duration(param) => ("duration", param.into()),
			Self::Metadata(param) => ("metadata", param.into()),
			Self::ProgramId(param) => ("program_id", param.into()),
			Self::FfmpegDataId(param) => ("ffmpeg_data_id", param.into()),
			Self::Codec(param) => (
				"codec",
				::prisma_client_rust::PrismaValue::Object(
					param.into_iter().map(Into::into).collect(),
				),
			),
			Self::Program(param) => (
				"program",
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
	Codec(super::ffmpeg_media_codec::UniqueArgs),
	Program(super::ffmpeg_media_program::UniqueArgs),
}
impl Into<::prisma_client_rust::Selection> for WithParam {
	fn into(self) -> ::prisma_client_rust::Selection {
		match self {
			Self::Codec(args) => {
				let mut selections = < super :: ffmpeg_media_codec :: Types as :: prisma_client_rust :: ModelTypes > :: scalar_selections () ;
				selections.extend(
					args.with_params
						.into_iter()
						.map(Into::<::prisma_client_rust::Selection>::into),
				);
				::prisma_client_rust::Selection::new(codec::NAME, None, [], selections)
			}
			Self::Program(args) => {
				let mut selections = < super :: ffmpeg_media_program :: Types as :: prisma_client_rust :: ModelTypes > :: scalar_selections () ;
				selections.extend(
					args.with_params
						.into_iter()
						.map(Into::<::prisma_client_rust::Selection>::into),
				);
				::prisma_client_rust::Selection::new(program::NAME, None, [], selections)
			}
		}
	}
}
#[derive(Debug, Clone)]
pub enum SetParam {
	StreamId(super::_prisma::write_params::IntParam),
	Name(super::_prisma::write_params::StringNullableParam),
	ConnectCodec(super::ffmpeg_media_codec::UniqueWhereParam),
	DisconnectCodec,
	AspectRatioNum(super::_prisma::write_params::IntParam),
	AspectRatioDen(super::_prisma::write_params::IntParam),
	FramesPerSecondNum(super::_prisma::write_params::IntParam),
	FramesPerSecondDen(super::_prisma::write_params::IntParam),
	TimeBaseRealDen(super::_prisma::write_params::IntParam),
	TimeBaseRealNum(super::_prisma::write_params::IntParam),
	Dispositions(super::_prisma::write_params::StringNullableParam),
	Title(super::_prisma::write_params::StringNullableParam),
	Encoder(super::_prisma::write_params::StringNullableParam),
	Language(super::_prisma::write_params::StringNullableParam),
	Duration(super::_prisma::write_params::BytesNullableParam),
	Metadata(super::_prisma::write_params::BytesNullableParam),
	ConnectProgram(super::ffmpeg_media_program::UniqueWhereParam),
	ProgramId(super::_prisma::write_params::IntParam),
	FfmpegDataId(super::_prisma::write_params::IntParam),
}
impl Into<(String, ::prisma_client_rust::PrismaValue)> for SetParam {
	fn into(self) -> (String, ::prisma_client_rust::PrismaValue) {
		let (k, v) = match self {
			Self::StreamId(value) => (stream_id::NAME, value.into()),
			Self::Name(value) => (name::NAME, value.into()),
			Self::ConnectCodec(where_param) => (
				codec::NAME,
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
			Self::DisconnectCodec => (
				codec::NAME,
				::prisma_client_rust::PrismaValue::Object(vec![(
					"disconnect".to_string(),
					::prisma_client_rust::PrismaValue::Boolean(true),
				)]),
			),
			Self::AspectRatioNum(value) => (aspect_ratio_num::NAME, value.into()),
			Self::AspectRatioDen(value) => (aspect_ratio_den::NAME, value.into()),
			Self::FramesPerSecondNum(value) => (frames_per_second_num::NAME, value.into()),
			Self::FramesPerSecondDen(value) => (frames_per_second_den::NAME, value.into()),
			Self::TimeBaseRealDen(value) => (time_base_real_den::NAME, value.into()),
			Self::TimeBaseRealNum(value) => (time_base_real_num::NAME, value.into()),
			Self::Dispositions(value) => (dispositions::NAME, value.into()),
			Self::Title(value) => (title::NAME, value.into()),
			Self::Encoder(value) => (encoder::NAME, value.into()),
			Self::Language(value) => (language::NAME, value.into()),
			Self::Duration(value) => (duration::NAME, value.into()),
			Self::Metadata(value) => (metadata::NAME, value.into()),
			Self::ConnectProgram(where_param) => (
				program::NAME,
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
			Self::ProgramId(value) => (program_id::NAME, value.into()),
			Self::FfmpegDataId(value) => (ffmpeg_data_id::NAME, value.into()),
		};
		(k.to_string(), v)
	}
}
#[derive(Debug, Clone)]
pub enum UncheckedSetParam {
	StreamId(super::_prisma::write_params::IntParam),
	Name(super::_prisma::write_params::StringNullableParam),
	AspectRatioNum(super::_prisma::write_params::IntParam),
	AspectRatioDen(super::_prisma::write_params::IntParam),
	FramesPerSecondNum(super::_prisma::write_params::IntParam),
	FramesPerSecondDen(super::_prisma::write_params::IntParam),
	TimeBaseRealDen(super::_prisma::write_params::IntParam),
	TimeBaseRealNum(super::_prisma::write_params::IntParam),
	Dispositions(super::_prisma::write_params::StringNullableParam),
	Title(super::_prisma::write_params::StringNullableParam),
	Encoder(super::_prisma::write_params::StringNullableParam),
	Language(super::_prisma::write_params::StringNullableParam),
	Duration(super::_prisma::write_params::BytesNullableParam),
	Metadata(super::_prisma::write_params::BytesNullableParam),
	ProgramId(super::_prisma::write_params::IntParam),
	FfmpegDataId(super::_prisma::write_params::IntParam),
}
impl Into<(String, ::prisma_client_rust::PrismaValue)> for UncheckedSetParam {
	fn into(self) -> (String, ::prisma_client_rust::PrismaValue) {
		let (k, v) = match self {
			Self::StreamId(value) => ("stream_id", value.into()),
			Self::Name(value) => ("name", value.into()),
			Self::AspectRatioNum(value) => ("aspect_ratio_num", value.into()),
			Self::AspectRatioDen(value) => ("aspect_ratio_den", value.into()),
			Self::FramesPerSecondNum(value) => ("frames_per_second_num", value.into()),
			Self::FramesPerSecondDen(value) => ("frames_per_second_den", value.into()),
			Self::TimeBaseRealDen(value) => ("time_base_real_den", value.into()),
			Self::TimeBaseRealNum(value) => ("time_base_real_num", value.into()),
			Self::Dispositions(value) => ("dispositions", value.into()),
			Self::Title(value) => ("title", value.into()),
			Self::Encoder(value) => ("encoder", value.into()),
			Self::Language(value) => ("language", value.into()),
			Self::Duration(value) => ("duration", value.into()),
			Self::Metadata(value) => ("metadata", value.into()),
			Self::ProgramId(value) => ("program_id", value.into()),
			Self::FfmpegDataId(value) => ("ffmpeg_data_id", value.into()),
		};
		(k.to_string(), v)
	}
}
::prisma_client_rust::macros::select_factory!(
	_select_ffmpeg_media_stream,
	select,
	prisma::ffmpeg_media_stream,
	struct Data {
		#[serde(rename = "stream_id")]
		stream_id: stream_id::Type,
		#[serde(rename = "name")]
		name: name::Type,
		#[serde(rename = "codec")]
		codec: codec::Type,
		#[serde(rename = "aspect_ratio_num")]
		aspect_ratio_num: aspect_ratio_num::Type,
		#[serde(rename = "aspect_ratio_den")]
		aspect_ratio_den: aspect_ratio_den::Type,
		#[serde(rename = "frames_per_second_num")]
		frames_per_second_num: frames_per_second_num::Type,
		#[serde(rename = "frames_per_second_den")]
		frames_per_second_den: frames_per_second_den::Type,
		#[serde(rename = "time_base_real_den")]
		time_base_real_den: time_base_real_den::Type,
		#[serde(rename = "time_base_real_num")]
		time_base_real_num: time_base_real_num::Type,
		#[serde(rename = "dispositions")]
		dispositions: dispositions::Type,
		#[serde(rename = "title")]
		title: title::Type,
		#[serde(rename = "encoder")]
		encoder: encoder::Type,
		#[serde(rename = "language")]
		language: language::Type,
		#[serde(rename = "duration")]
		duration: duration::Type,
		#[serde(rename = "metadata")]
		metadata: metadata::Type,
		#[serde(rename = "program")]
		program: program::Type,
		#[serde(rename = "program_id")]
		program_id: program_id::Type,
		#[serde(rename = "ffmpeg_data_id")]
		ffmpeg_data_id: ffmpeg_data_id::Type,
	},
	[
		(stream_id, Scalar),
		(name, Scalar),
		(codec, Relation(prisma::ffmpeg_media_codec, Optional)),
		(aspect_ratio_num, Scalar),
		(aspect_ratio_den, Scalar),
		(frames_per_second_num, Scalar),
		(frames_per_second_den, Scalar),
		(time_base_real_den, Scalar),
		(time_base_real_num, Scalar),
		(dispositions, Scalar),
		(title, Scalar),
		(encoder, Scalar),
		(language, Scalar),
		(duration, Scalar),
		(metadata, Scalar),
		(program, Relation(prisma::ffmpeg_media_program, One)),
		(program_id, Scalar),
		(ffmpeg_data_id, Scalar)
	]
);
pub enum SelectParam {
	StreamId(stream_id::Select),
	Name(name::Select),
	Codec(codec::Select),
	AspectRatioNum(aspect_ratio_num::Select),
	AspectRatioDen(aspect_ratio_den::Select),
	FramesPerSecondNum(frames_per_second_num::Select),
	FramesPerSecondDen(frames_per_second_den::Select),
	TimeBaseRealDen(time_base_real_den::Select),
	TimeBaseRealNum(time_base_real_num::Select),
	Dispositions(dispositions::Select),
	Title(title::Select),
	Encoder(encoder::Select),
	Language(language::Select),
	Duration(duration::Select),
	Metadata(metadata::Select),
	Program(program::Select),
	ProgramId(program_id::Select),
	FfmpegDataId(ffmpeg_data_id::Select),
}
impl Into<::prisma_client_rust::Selection> for SelectParam {
	fn into(self) -> ::prisma_client_rust::Selection {
		match self {
			Self::StreamId(data) => data.into(),
			Self::Name(data) => data.into(),
			Self::Codec(data) => data.into(),
			Self::AspectRatioNum(data) => data.into(),
			Self::AspectRatioDen(data) => data.into(),
			Self::FramesPerSecondNum(data) => data.into(),
			Self::FramesPerSecondDen(data) => data.into(),
			Self::TimeBaseRealDen(data) => data.into(),
			Self::TimeBaseRealNum(data) => data.into(),
			Self::Dispositions(data) => data.into(),
			Self::Title(data) => data.into(),
			Self::Encoder(data) => data.into(),
			Self::Language(data) => data.into(),
			Self::Duration(data) => data.into(),
			Self::Metadata(data) => data.into(),
			Self::Program(data) => data.into(),
			Self::ProgramId(data) => data.into(),
			Self::FfmpegDataId(data) => data.into(),
		}
	}
}
::prisma_client_rust::macros::include_factory!(
	_include_ffmpeg_media_stream,
	include,
	prisma::ffmpeg_media_stream,
	struct Data {
		#[serde(rename = "stream_id")]
		stream_id: stream_id::Type,
		#[serde(rename = "name")]
		name: name::Type,
		#[serde(rename = "codec")]
		codec: codec::Type,
		#[serde(rename = "aspect_ratio_num")]
		aspect_ratio_num: aspect_ratio_num::Type,
		#[serde(rename = "aspect_ratio_den")]
		aspect_ratio_den: aspect_ratio_den::Type,
		#[serde(rename = "frames_per_second_num")]
		frames_per_second_num: frames_per_second_num::Type,
		#[serde(rename = "frames_per_second_den")]
		frames_per_second_den: frames_per_second_den::Type,
		#[serde(rename = "time_base_real_den")]
		time_base_real_den: time_base_real_den::Type,
		#[serde(rename = "time_base_real_num")]
		time_base_real_num: time_base_real_num::Type,
		#[serde(rename = "dispositions")]
		dispositions: dispositions::Type,
		#[serde(rename = "title")]
		title: title::Type,
		#[serde(rename = "encoder")]
		encoder: encoder::Type,
		#[serde(rename = "language")]
		language: language::Type,
		#[serde(rename = "duration")]
		duration: duration::Type,
		#[serde(rename = "metadata")]
		metadata: metadata::Type,
		#[serde(rename = "program")]
		program: program::Type,
		#[serde(rename = "program_id")]
		program_id: program_id::Type,
		#[serde(rename = "ffmpeg_data_id")]
		ffmpeg_data_id: ffmpeg_data_id::Type,
	},
	[
		(codec, Relation(prisma::ffmpeg_media_codec, Optional)),
		(program, Relation(prisma::ffmpeg_media_program, One))
	]
);
pub enum IncludeParam {
	StreamId(stream_id::Include),
	Name(name::Include),
	Codec(codec::Include),
	AspectRatioNum(aspect_ratio_num::Include),
	AspectRatioDen(aspect_ratio_den::Include),
	FramesPerSecondNum(frames_per_second_num::Include),
	FramesPerSecondDen(frames_per_second_den::Include),
	TimeBaseRealDen(time_base_real_den::Include),
	TimeBaseRealNum(time_base_real_num::Include),
	Dispositions(dispositions::Include),
	Title(title::Include),
	Encoder(encoder::Include),
	Language(language::Include),
	Duration(duration::Include),
	Metadata(metadata::Include),
	Program(program::Include),
	ProgramId(program_id::Include),
	FfmpegDataId(ffmpeg_data_id::Include),
}
impl Into<::prisma_client_rust::Selection> for IncludeParam {
	fn into(self) -> ::prisma_client_rust::Selection {
		match self {
			Self::StreamId(data) => data.into(),
			Self::Name(data) => data.into(),
			Self::Codec(data) => data.into(),
			Self::AspectRatioNum(data) => data.into(),
			Self::AspectRatioDen(data) => data.into(),
			Self::FramesPerSecondNum(data) => data.into(),
			Self::FramesPerSecondDen(data) => data.into(),
			Self::TimeBaseRealDen(data) => data.into(),
			Self::TimeBaseRealNum(data) => data.into(),
			Self::Dispositions(data) => data.into(),
			Self::Title(data) => data.into(),
			Self::Encoder(data) => data.into(),
			Self::Language(data) => data.into(),
			Self::Duration(data) => data.into(),
			Self::Metadata(data) => data.into(),
			Self::Program(data) => data.into(),
			Self::ProgramId(data) => data.into(),
			Self::FfmpegDataId(data) => data.into(),
		}
	}
}
#[derive(Debug, Clone)]
pub struct Create {
	pub stream_id: Int,
	pub aspect_ratio_num: Int,
	pub aspect_ratio_den: Int,
	pub frames_per_second_num: Int,
	pub frames_per_second_den: Int,
	pub time_base_real_den: Int,
	pub time_base_real_num: Int,
	pub program: super::ffmpeg_media_program::UniqueWhereParam,
	pub _params: Vec<SetParam>,
}
impl Create {
	pub fn to_query<'a>(self, client: &'a PrismaClient) -> CreateQuery<'a> {
		client.ffmpeg_media_stream().create(
			self.stream_id,
			self.aspect_ratio_num,
			self.aspect_ratio_den,
			self.frames_per_second_num,
			self.frames_per_second_den,
			self.time_base_real_den,
			self.time_base_real_num,
			self.program,
			self._params,
		)
	}
	pub fn to_params(mut self) -> Vec<SetParam> {
		self._params.extend([
			stream_id::set(self.stream_id),
			aspect_ratio_num::set(self.aspect_ratio_num),
			aspect_ratio_den::set(self.aspect_ratio_den),
			frames_per_second_num::set(self.frames_per_second_num),
			frames_per_second_den::set(self.frames_per_second_den),
			time_base_real_den::set(self.time_base_real_den),
			time_base_real_num::set(self.time_base_real_num),
			program::connect(self.program),
		]);
		self._params
	}
}
pub fn create(
	stream_id: Int,
	aspect_ratio_num: Int,
	aspect_ratio_den: Int,
	frames_per_second_num: Int,
	frames_per_second_den: Int,
	time_base_real_den: Int,
	time_base_real_num: Int,
	program: super::ffmpeg_media_program::UniqueWhereParam,
	_params: Vec<SetParam>,
) -> Create {
	Create {
		stream_id,
		aspect_ratio_num,
		aspect_ratio_den,
		frames_per_second_num,
		frames_per_second_den,
		time_base_real_den,
		time_base_real_num,
		program,
		_params,
	}
}
#[derive(Debug, Clone)]
pub struct CreateUnchecked {
	pub stream_id: Int,
	pub aspect_ratio_num: Int,
	pub aspect_ratio_den: Int,
	pub frames_per_second_num: Int,
	pub frames_per_second_den: Int,
	pub time_base_real_den: Int,
	pub time_base_real_num: Int,
	pub program_id: Int,
	pub ffmpeg_data_id: Int,
	pub _params: Vec<UncheckedSetParam>,
}
impl CreateUnchecked {
	pub fn to_query<'a>(self, client: &'a PrismaClient) -> CreateUncheckedQuery<'a> {
		client.ffmpeg_media_stream().create_unchecked(
			self.stream_id,
			self.aspect_ratio_num,
			self.aspect_ratio_den,
			self.frames_per_second_num,
			self.frames_per_second_den,
			self.time_base_real_den,
			self.time_base_real_num,
			self.program_id,
			self.ffmpeg_data_id,
			self._params,
		)
	}
	pub fn to_params(mut self) -> Vec<UncheckedSetParam> {
		self._params.extend([
			stream_id::set(self.stream_id),
			aspect_ratio_num::set(self.aspect_ratio_num),
			aspect_ratio_den::set(self.aspect_ratio_den),
			frames_per_second_num::set(self.frames_per_second_num),
			frames_per_second_den::set(self.frames_per_second_den),
			time_base_real_den::set(self.time_base_real_den),
			time_base_real_num::set(self.time_base_real_num),
			program_id::set(self.program_id),
			ffmpeg_data_id::set(self.ffmpeg_data_id),
		]);
		self._params
	}
}
pub fn create_unchecked(
	stream_id: Int,
	aspect_ratio_num: Int,
	aspect_ratio_den: Int,
	frames_per_second_num: Int,
	frames_per_second_den: Int,
	time_base_real_den: Int,
	time_base_real_num: Int,
	program_id: Int,
	ffmpeg_data_id: Int,
	_params: Vec<UncheckedSetParam>,
) -> CreateUnchecked {
	CreateUnchecked {
		stream_id,
		aspect_ratio_num,
		aspect_ratio_den,
		frames_per_second_num,
		frames_per_second_den,
		time_base_real_den,
		time_base_real_num,
		program_id,
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
			::prisma_client_rust::sel(stream_id::NAME),
			::prisma_client_rust::sel(name::NAME),
			::prisma_client_rust::sel(aspect_ratio_num::NAME),
			::prisma_client_rust::sel(aspect_ratio_den::NAME),
			::prisma_client_rust::sel(frames_per_second_num::NAME),
			::prisma_client_rust::sel(frames_per_second_den::NAME),
			::prisma_client_rust::sel(time_base_real_den::NAME),
			::prisma_client_rust::sel(time_base_real_num::NAME),
			::prisma_client_rust::sel(dispositions::NAME),
			::prisma_client_rust::sel(title::NAME),
			::prisma_client_rust::sel(encoder::NAME),
			::prisma_client_rust::sel(language::NAME),
			::prisma_client_rust::sel(duration::NAME),
			::prisma_client_rust::sel(metadata::NAME),
			::prisma_client_rust::sel(program_id::NAME),
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
# [specta (rename = "FfmpegMediaStream" , crate = prisma_client_rust :: specta)]
pub struct Data {
	#[serde(rename = "stream_id")]
	pub stream_id: stream_id::Type,
	#[serde(rename = "name")]
	pub name: name::Type,
	#[serde(
		rename = "codec",
		default,
		skip_serializing_if = "Option::is_none",
		with = "prisma_client_rust::serde::double_option"
	)]
	#[specta(skip)]
	pub codec: Option<codec::RecursiveSafeType>,
	#[serde(rename = "aspect_ratio_num")]
	pub aspect_ratio_num: aspect_ratio_num::Type,
	#[serde(rename = "aspect_ratio_den")]
	pub aspect_ratio_den: aspect_ratio_den::Type,
	#[serde(rename = "frames_per_second_num")]
	pub frames_per_second_num: frames_per_second_num::Type,
	#[serde(rename = "frames_per_second_den")]
	pub frames_per_second_den: frames_per_second_den::Type,
	#[serde(rename = "time_base_real_den")]
	pub time_base_real_den: time_base_real_den::Type,
	#[serde(rename = "time_base_real_num")]
	pub time_base_real_num: time_base_real_num::Type,
	#[serde(rename = "dispositions")]
	pub dispositions: dispositions::Type,
	#[serde(rename = "title")]
	pub title: title::Type,
	#[serde(rename = "encoder")]
	pub encoder: encoder::Type,
	#[serde(rename = "language")]
	pub language: language::Type,
	#[serde(rename = "duration")]
	pub duration: duration::Type,
	#[serde(rename = "metadata")]
	pub metadata: metadata::Type,
	#[serde(rename = "program")]
	#[specta(skip)]
	pub program: Option<program::RecursiveSafeType>,
	#[serde(rename = "program_id")]
	pub program_id: program_id::Type,
	#[serde(rename = "ffmpeg_data_id")]
	pub ffmpeg_data_id: ffmpeg_data_id::Type,
}
impl Data {
	pub fn codec(
		&self,
	) -> Result<
		Option<&super::ffmpeg_media_codec::Data>,
		::prisma_client_rust::RelationNotFetchedError,
	> {
		self.codec
			.as_ref()
			.ok_or(::prisma_client_rust::RelationNotFetchedError::new(
				stringify!(codec),
			))
			.map(|v| v.as_ref().map(|v| v.as_ref()))
	}
	pub fn program(
		&self,
	) -> Result<&super::ffmpeg_media_program::Data, ::prisma_client_rust::RelationNotFetchedError> {
		self.program
			.as_ref()
			.ok_or(::prisma_client_rust::RelationNotFetchedError::new(
				stringify!(program),
			))
			.map(|v| v.as_ref())
	}
}
::prisma_client_rust::macros::partial_unchecked_factory!(
	_partial_unchecked_ffmpeg_media_stream,
	prisma::ffmpeg_media_stream,
	struct Data {
		#[serde(rename = "stream_id")]
		pub stream_id: prisma::ffmpeg_media_stream::stream_id::Type,
		#[serde(rename = "name")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub name: prisma::ffmpeg_media_stream::name::Type,
		#[serde(rename = "aspect_ratio_num")]
		pub aspect_ratio_num: prisma::ffmpeg_media_stream::aspect_ratio_num::Type,
		#[serde(rename = "aspect_ratio_den")]
		pub aspect_ratio_den: prisma::ffmpeg_media_stream::aspect_ratio_den::Type,
		#[serde(rename = "frames_per_second_num")]
		pub frames_per_second_num: prisma::ffmpeg_media_stream::frames_per_second_num::Type,
		#[serde(rename = "frames_per_second_den")]
		pub frames_per_second_den: prisma::ffmpeg_media_stream::frames_per_second_den::Type,
		#[serde(rename = "time_base_real_den")]
		pub time_base_real_den: prisma::ffmpeg_media_stream::time_base_real_den::Type,
		#[serde(rename = "time_base_real_num")]
		pub time_base_real_num: prisma::ffmpeg_media_stream::time_base_real_num::Type,
		#[serde(rename = "dispositions")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub dispositions: prisma::ffmpeg_media_stream::dispositions::Type,
		#[serde(rename = "title")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub title: prisma::ffmpeg_media_stream::title::Type,
		#[serde(rename = "encoder")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub encoder: prisma::ffmpeg_media_stream::encoder::Type,
		#[serde(rename = "language")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub language: prisma::ffmpeg_media_stream::language::Type,
		#[serde(rename = "duration")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub duration: prisma::ffmpeg_media_stream::duration::Type,
		#[serde(rename = "metadata")]
		#[serde(default, with = "::prisma_client_rust::serde::double_option")]
		pub metadata: prisma::ffmpeg_media_stream::metadata::Type,
		#[serde(rename = "program_id")]
		pub program_id: prisma::ffmpeg_media_stream::program_id::Type,
		#[serde(rename = "ffmpeg_data_id")]
		pub ffmpeg_data_id: prisma::ffmpeg_media_stream::ffmpeg_data_id::Type,
	}
);
::prisma_client_rust::macros::filter_factory!(
	_ffmpeg_media_stream_filter,
	prisma::ffmpeg_media_stream,
	[
		(stream_id, Scalar),
		(name, Scalar),
		(codec, Relation(prisma::ffmpeg_media_codec, Optional)),
		(aspect_ratio_num, Scalar),
		(aspect_ratio_den, Scalar),
		(frames_per_second_num, Scalar),
		(frames_per_second_den, Scalar),
		(time_base_real_den, Scalar),
		(time_base_real_num, Scalar),
		(dispositions, Scalar),
		(title, Scalar),
		(encoder, Scalar),
		(language, Scalar),
		(duration, Scalar),
		(metadata, Scalar),
		(program, Relation(prisma::ffmpeg_media_program, One)),
		(program_id, Scalar),
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
		stream_id: Int,
		aspect_ratio_num: Int,
		aspect_ratio_den: Int,
		frames_per_second_num: Int,
		frames_per_second_den: Int,
		time_base_real_den: Int,
		time_base_real_num: Int,
		program: super::ffmpeg_media_program::UniqueWhereParam,
		mut _params: Vec<SetParam>,
	) -> CreateQuery<'a> {
		_params.extend([
			stream_id::set(stream_id),
			aspect_ratio_num::set(aspect_ratio_num),
			aspect_ratio_den::set(aspect_ratio_den),
			frames_per_second_num::set(frames_per_second_num),
			frames_per_second_den::set(frames_per_second_den),
			time_base_real_den::set(time_base_real_den),
			time_base_real_num::set(time_base_real_num),
			program::connect(program),
		]);
		CreateQuery::new(self.client, _params)
	}
	pub fn create_unchecked(
		self,
		stream_id: Int,
		aspect_ratio_num: Int,
		aspect_ratio_den: Int,
		frames_per_second_num: Int,
		frames_per_second_den: Int,
		time_base_real_den: Int,
		time_base_real_num: Int,
		program_id: Int,
		ffmpeg_data_id: Int,
		mut _params: Vec<UncheckedSetParam>,
	) -> CreateUncheckedQuery<'a> {
		_params.extend([
			stream_id::set(stream_id),
			aspect_ratio_num::set(aspect_ratio_num),
			aspect_ratio_den::set(aspect_ratio_den),
			frames_per_second_num::set(frames_per_second_num),
			frames_per_second_den::set(frames_per_second_den),
			time_base_real_den::set(time_base_real_den),
			time_base_real_num::set(time_base_real_num),
			program_id::set(program_id),
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
