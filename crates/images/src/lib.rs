#![warn(
	clippy::all,
	clippy::pedantic,
	clippy::correctness,
	clippy::perf,
	clippy::style,
	clippy::suspicious,
	clippy::complexity,
	clippy::nursery,
	clippy::unwrap_used,
	unused_qualifications,
	rust_2018_idioms,
	clippy::expect_used,
	trivial_casts,
	trivial_numeric_casts,
	unused_allocation,
	clippy::as_conversions,
	clippy::dbg_macro
)]
#![forbid(unsafe_code)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

use std::{fs, path::Path};

mod consts;
mod error;
mod generic;
mod handler;
#[cfg(feature = "heif")]
mod heif;
mod pdf;
mod svg;

use consts::MAXIMUM_FILE_SIZE;

// Re-exports
pub use consts::{all_compatible_extensions, ConvertableExtension};
pub use error::{Error, Result};
pub use handler::{convert_image, format_image};
pub use image::DynamicImage;

pub trait ImageHandler {
	#[inline]
	fn get_data(&self, path: &Path) -> Result<Vec<u8>>
	where
		Self: Sized,
	{
		self.validate_image(path)?;

		fs::read(path).map_err(|e| Error::Io(e, path.to_path_buf().into_boxed_path()))
	}

	fn validate_image(&self, path: &Path) -> Result<()>
	where
		Self: Sized,
	{
		if fs::metadata(path)
			.map_err(|e| Error::Io(e, path.to_path_buf().into_boxed_path()))?
			.len() <= MAXIMUM_FILE_SIZE
		{
			Ok(())
		} else {
			Err(Error::TooLarge)
		}
	}

	fn handle_image(&self, path: &Path) -> Result<DynamicImage>;

	#[inline]
	fn convert_image(
		&self,
		opposing_handler: Box<dyn ImageHandler>,
		path: &Path,
	) -> Result<DynamicImage> {
		opposing_handler.handle_image(path)
	}
}

/// This takes in a width and a height, and returns a scaled width and height
/// It is scaled proportionally to the [`TARGET_PX`], so smaller images will be upscaled,
/// and larger images will be downscaled. This approach also maintains the aspect ratio of the image.
#[allow(
	clippy::as_conversions,
	clippy::cast_precision_loss,
	clippy::cast_possible_truncation,
	clippy::cast_sign_loss
)]
#[must_use]
pub fn scale_dimensions(w: f32, h: f32, target_px: f32) -> (u32, u32) {
	let sf = (target_px / (w * h)).sqrt();
	((w * sf).round() as u32, (h * sf).round() as u32)
}
