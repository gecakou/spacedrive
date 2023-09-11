use crate::{film_strip_filter, MovieDecoder, ThumbnailSize, ThumbnailerError, VideoFrame};

use std::{io, ops::Deref, path::Path};
use tokio::{fs, task::spawn_blocking};
use tracing::error;
use webp::Encoder;

/// `Thumbnailer` struct holds data from a `ThumbnailerBuilder`, exposing methods
/// to generate thumbnails from video files.
#[derive(Debug, Clone)]
pub struct Thumbnailer {
	builder: ThumbnailerBuilder,
}

impl Thumbnailer {
	/// Processes an video input file and write to file system a thumbnail with webp format
	pub async fn process(
		&self,
		video_file_path: impl AsRef<Path> + Send,
		output_thumbnail_path: impl AsRef<Path> + Send,
	) -> Result<(), ThumbnailerError> {
		let path = output_thumbnail_path.as_ref().parent().ok_or_else(|| {
			io::Error::new(
				io::ErrorKind::InvalidInput,
				"Cannot determine parent directory",
			)
		})?;

		fs::create_dir_all(path).await?;

		fs::write(
			output_thumbnail_path,
			&*self.process_to_webp_bytes(video_file_path).await?,
		)
		.await
		.map_err(Into::into)
	}

	/// Processes an video input file and returns a webp encoded thumbnail as bytes
	pub async fn process_to_webp_bytes(
		&self,
		video_file_path: impl AsRef<Path> + Send,
	) -> Result<Vec<u8>, ThumbnailerError> {
		let video_file_path = video_file_path.as_ref().to_path_buf();
		let prefer_embedded_metadata = self.builder.prefer_embedded_metadata;
		let seek_percentage = self.builder.seek_percentage;
		let size = self.builder.size;
		let maintain_aspect_ratio = self.builder.maintain_aspect_ratio;
		let with_film_strip = self.builder.with_film_strip;
		let quality = self.builder.quality;

		spawn_blocking(move || -> Result<Vec<u8>, ThumbnailerError> {
			let mut decoder = MovieDecoder::new(video_file_path.clone(), prefer_embedded_metadata)?;
			// We actually have to decode a frame to get some metadata before we can start decoding for real
			decoder.decode_video_frame()?;

			#[allow(clippy::cast_possible_truncation)]
			#[allow(clippy::cast_precision_loss)]
			if !decoder.embedded_metadata_is_available() {
				let result = decoder.seek(
					(decoder.get_video_duration().as_secs() as f64 * f64::from(seek_percentage))
						.round() as i64,
				);

				if let Err(err) = result {
					error!("Failed to seek: {err:#?}");
					// seeking failed, try the first frame again
					decoder = MovieDecoder::new(video_file_path, prefer_embedded_metadata)?;
					decoder.decode_video_frame()?;
				}
			}

			let mut video_frame = VideoFrame::default();

			decoder.get_scaled_video_frame(Some(size), maintain_aspect_ratio, &mut video_frame)?;

			if with_film_strip {
				film_strip_filter(&mut video_frame);
			}

			// Type WebPMemory is !Send, which makes the Future in this function !Send,
			// this make us `deref` to have a `&[u8]` and then `to_owned` to make a Vec<u8>
			// which implies on a unwanted clone...
			Ok(
				Encoder::from_rgb(&video_frame.data, video_frame.width, video_frame.height)
					.encode(quality)
					.deref()
					.to_vec(),
			)
		})
		.await?
	}
}

/// `ThumbnailerBuilder` struct holds data to build a `Thumbnailer` struct, exposing many methods
/// to configure how a thumbnail must be generated.
#[derive(Debug, Clone)]
#[must_use]
pub struct ThumbnailerBuilder {
	maintain_aspect_ratio: bool,
	size: ThumbnailSize,
	seek_percentage: f32,
	quality: f32,
	prefer_embedded_metadata: bool,
	with_film_strip: bool,
}

impl Default for ThumbnailerBuilder {
	fn default() -> Self {
		Self {
			maintain_aspect_ratio: true,
			size: ThumbnailSize::Size(128),
			seek_percentage: 0.1,
			quality: 80.0,
			prefer_embedded_metadata: true,
			with_film_strip: true,
		}
	}
}

impl ThumbnailerBuilder {
	/// Creates a new `ThumbnailerBuilder` with default values:
	/// - `maintain_aspect_ratio`: true
	/// - `size`: 128 pixels
	/// - `seek_percentage`: 10%
	/// - `quality`: 80
	/// - `prefer_embedded_metadata`: true
	/// - `with_film_strip`: true
	pub fn new() -> Self {
		Self::default()
	}

	/// To respect or not the aspect ratio from the video file in the generated thumbnail
	pub const fn maintain_aspect_ratio(mut self, maintain_aspect_ratio: bool) -> Self {
		self.maintain_aspect_ratio = maintain_aspect_ratio;
		self
	}

	/// To set a thumbnail size, respecting or not its aspect ratio, according to `maintain_aspect_ratio` value
	pub const fn size(mut self, size: u32) -> Self {
		self.size = ThumbnailSize::Size(size);
		self
	}

	/// To specify width and height of the thumbnail
	pub const fn width_and_height(mut self, width: u32, height: u32) -> Self {
		self.size = ThumbnailSize::Dimensions { width, height };
		self
	}

	/// Seek percentage must be a value between 0.0 and 1.0
	pub fn seek_percentage(mut self, seek_percentage: f32) -> Result<Self, ThumbnailerError> {
		if !(0.0..=1.0).contains(&seek_percentage) {
			return Err(ThumbnailerError::InvalidSeekPercentage(seek_percentage));
		}
		self.seek_percentage = seek_percentage;
		Ok(self)
	}

	/// Quality must be a value between 0.0 and 100.0
	pub fn quality(mut self, quality: f32) -> Result<Self, ThumbnailerError> {
		if !(0.0..=100.0).contains(&quality) {
			return Err(ThumbnailerError::InvalidQuality(quality));
		}
		self.quality = quality;
		Ok(self)
	}

	/// To use embedded metadata in the video file, if available, instead of getting a frame as a
	/// thumbnail
	pub const fn prefer_embedded_metadata(mut self, prefer_embedded_metadata: bool) -> Self {
		self.prefer_embedded_metadata = prefer_embedded_metadata;
		self
	}

	/// If `with_film_strip` is true, a film strip will be added to the thumbnail borders
	pub const fn with_film_strip(mut self, with_film_strip: bool) -> Self {
		self.with_film_strip = with_film_strip;
		self
	}

	/// Builds a `Thumbnailer` struct
	#[must_use]
	pub const fn build(self) -> Thumbnailer {
		Thumbnailer { builder: self }
	}
}
