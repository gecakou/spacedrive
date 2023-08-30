use crate::{
	consts::HEIF_MAXIMUM_FILE_SIZE,
	error::{Error, Result},
};
use image::DynamicImage;
use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};
use std::{
	fs,
	io::{Cursor, Read, Seek, SeekFrom},
	path::Path,
};

pub fn heif_to_dynamic_image(path: &Path) -> Result<DynamicImage> {
	if fs::metadata(path).map_err(|_| Error::Io)?.len() > HEIF_MAXIMUM_FILE_SIZE {
		return Err(Error::TooLarge);
	}

	let img = {
		// do this in a separate block so we drop the raw (potentially huge) image handle
		let ctx = HeifContext::read_from_file(path.to_str().ok_or(Error::InvalidPath)?)?;
		let heif = LibHeif::new();
		let handle = ctx.primary_image_handle()?;

		heif.decode(&handle, ColorSpace::Rgb(RgbChroma::Rgb), None)?
	};

	// TODO(brxken128): add support for images with individual r/g/b channels
	// i'm unable to find a sample to test with, but it should follow the same principles as this one
	if let Some(i) = img.planes().interleaved {
		if i.bits_per_pixel != 8 {
			return Err(Error::InvalidBitDepth);
		}

		let data = i.data.to_vec();
		let mut reader = Cursor::new(data);

		let mut sequence = vec![];
		let mut buffer = [0u8; 3]; // [r, g, b]

		// this is the interpolation stuff, it essentially just makes the image correct
		// in regards to stretching/resolution, etc
		for y in 0..img.height() {
			reader
				.seek(SeekFrom::Start((i.stride * y as usize) as u64))
				.map_err(|_| Error::Io)?;

			for _ in 0..img.width() {
				reader.read_exact(&mut buffer).map_err(|_| Error::Io)?;
				sequence.extend_from_slice(&buffer);
			}
		}

		let rgb_img = image::RgbImage::from_raw(img.width(), img.height(), sequence)
			.ok_or(Error::RgbImageConversion)?;

		Ok(DynamicImage::ImageRgb8(rgb_img))
	} else {
		Err(Error::Unsupported)
	}
}
