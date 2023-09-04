use crate::{
	consts::{HEIF_EXTENSIONS, SVG_EXTENSIONS},
	error::{Error, Result},
	generic::GenericHandler,
	svg::SvgHandler,
	ToImage,
};
use image::DynamicImage;
use std::{
	ffi::{OsStr, OsString},
	path::Path,
};

#[cfg(not(target_os = "linux"))]
use crate::heif::HeifHandler;

pub fn format_image(path: impl AsRef<Path>) -> Result<DynamicImage> {
	let ext = path.as_ref().extension().ok_or(Error::NoExtension)?;
	match_to_handler(ext).handle_image(path.as_ref())
}

fn match_to_handler(ext: &OsStr) -> Box<dyn ToImage> {
	let mut handler: Box<dyn ToImage> = Box::new(GenericHandler {});

	#[cfg(not(target_os = "linux"))]
	if HEIF_EXTENSIONS.iter().map(OsString::from).any(|x| x == ext) {
		handler = Box::new(HeifHandler {})
	}

	// raw next

	if SVG_EXTENSIONS.iter().map(OsString::from).any(|x| x == ext) {
		handler = Box::new(SvgHandler {})
	}

	handler
}
