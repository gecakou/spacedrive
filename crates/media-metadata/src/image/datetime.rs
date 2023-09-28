use super::{
	consts::{OFFSET_TAGS, TIME_TAGS},
	ExifReader,
};
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use serde::de::{self, Deserialize, Deserializer, Visitor};

pub const UTC_FORMAT_STR: &str = "%F %T %z";
pub const NAIVE_FORMAT_STR: &str = "%F %T";

/// This can be either naive with no TZ (`YYYY-MM-DD HH-MM-SS`) or UTC with a fixed offset (`rfc3339`).
#[derive(Clone, Debug, PartialEq, Eq, specta::Type)]
pub enum MediaDate {
	Naive(NaiveDateTime),
	Utc(DateTime<FixedOffset>),
}

impl MediaDate {
	/// This iterates over all 3 pairs of time/offset tags in an attempt to create a UTC time.
	///
	/// If the above fails, we fall back to Naive time - if that's not present this is `Undefined`.
	pub fn from_reader(reader: &ExifReader) -> Option<Self> {
		let z = TIME_TAGS
			.into_iter()
			.zip(OFFSET_TAGS)
			.filter_map(|(time_tag, offset_tag)| {
				let time = reader.get_tag::<String>(time_tag);
				let offset = reader.get_tag::<String>(offset_tag);

				if let (Some(t), Some(o)) = (time.clone(), offset) {
					DateTime::parse_from_str(&(format!("{t} {o}")), UTC_FORMAT_STR)
						.ok()
						.map(Self::Utc)
				} else if let Some(t) = time {
					NaiveDateTime::parse_from_str(&t, NAIVE_FORMAT_STR)
						.map_or(None, |x| Some(Self::Naive(x)))
				} else {
					None
				}
			})
			.collect::<Vec<_>>();

		z.iter()
			.find(|x| matches!(x, Self::Utc(_) | Self::Naive(_)))
			.map(Clone::clone)
	}

	/// Returns the amount of non-leap secods since the Unix Epoch (1970-01-01T00:00:00+00:00)
	///
	/// This is for search ordering/sorting
	#[must_use]
	pub fn unix_timestamp(&self) -> i64 {
		match self {
			Self::Utc(t) => t.timestamp(),
			Self::Naive(t) => t.timestamp(),
		}
	}
}

impl serde::Serialize for MediaDate {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		match self {
			Self::Utc(t) => serializer.serialize_str(&t.format(UTC_FORMAT_STR).to_string()),
			Self::Naive(t) => serializer.serialize_str(&t.format(NAIVE_FORMAT_STR).to_string()),
		}
	}
}

struct MediaDateVisitor;

impl<'de> Visitor<'de> for MediaDateVisitor {
	type Value = MediaDate;

	fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		formatter.write_str("either `UTC_FORMAT_STR` or `NAIVE_FORMAT_STR`")
	}

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: de::Error,
	{
		DateTime::parse_from_str(v, UTC_FORMAT_STR).map_or_else(
			|_| {
				NaiveDateTime::parse_from_str(v, NAIVE_FORMAT_STR).map_or_else(
					|_| Err(E::custom("unable to parse utc or naive from str")),
					|time| Ok(Self::Value::Naive(time)),
				)
			},
			|time| Ok(Self::Value::Utc(time)),
		)
	}
}

impl<'de> Deserialize<'de> for MediaDate {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		deserializer.deserialize_str(MediaDateVisitor)
	}
}

// #[cfg(test)]
// mod tests {
// 	use crate::ImageMetadata;

// 	use super::*;
// 	#[test]
// 	fn x() {
// 		let z = ImageMetadata::from_path(
// 			"/Users/broken/exif/PXL_20230714_222933902.ACTION_PAN-02.ORIGINAL.jpg",
// 		)
// 		.unwrap();

// 		// let st = z.date_taken.map(|x| x.tostri);

// 		// println!("{st:?}");

// 		// println!("{:?}", serde_json::from_slice::<MediaDate>(&st).unwrap());
// 	}
// }
