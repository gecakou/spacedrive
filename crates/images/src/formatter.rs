use crate::{
	consts,
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

#[cfg(feature = "heif")]
use crate::heif::HeifHandler;

pub fn format_image(path: impl AsRef<Path>) -> Result<DynamicImage> {
	let ext = path
		.as_ref()
		.extension()
		.map_or_else(|| Err(Error::NoExtension), |e| Ok(e.to_ascii_lowercase()))?;
	match_to_handler(&ext).handle_image(path.as_ref())
}

#[allow(clippy::useless_let_if_seq)]
fn match_to_handler(ext: &OsStr) -> Box<dyn ToImage> {
	let mut handler: Box<dyn ToImage> = Box::new(GenericHandler {});

	#[cfg(feature = "heif")]
	if consts::HEIF_EXTENSIONS
		.iter()
		.map(OsString::from)
		.any(|x| x == ext)
	{
		handler = Box::new(HeifHandler {});
	}

	if consts::SVG_EXTENSIONS
		.iter()
		.map(OsString::from)
		.any(|x| x == ext)
	{
		handler = Box::new(SvgHandler {});
	}

	handler
}
