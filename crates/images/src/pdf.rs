use std::{
	borrow::ToOwned,
	env::current_exe,
	path::{Path, PathBuf},
};

use crate::{consts::PDF_RENDER_WIDTH, Error::PdfiumBinding, ImageHandler, Result};
use image::DynamicImage;
use pdfium_render::prelude::{PdfPageRenderRotation, PdfRenderConfig, Pdfium};
use tracing::error;

// This path must be relative to the running binary
#[cfg(windows)]
const BINDING_LOCATION: &str = ".";
#[cfg(unix)]
const BINDING_LOCATION: &str = if cfg!(target_os = "macos") {
	"../Frameworks/FFMpeg.framework/Libraries"
} else {
	"../lib/spacedrive"
};

// FIX-ME: This is slow, but using Lazy with thread_safe was causing concurrency bugs that crashed the app
thread_local! {
	static PDFIUM: Option<Pdfium> = {
		let lib_name = Pdfium::pdfium_platform_library_name();
		let lib_path = current_exe()
			.ok()
			.and_then(|exe_path| {
				exe_path.parent().and_then(|parent_path| {
					match parent_path
						.join(BINDING_LOCATION)
						.join(&lib_name)
						.canonicalize()
					{
						Ok(lib_path) => lib_path.to_str().map(ToOwned::to_owned),
						Err(err) => {
							error!("{err:#?}");
							None
						}
					}
				})
			})
			.unwrap_or_else(|| {
				#[allow(clippy::expect_used)]
				PathBuf::from(BINDING_LOCATION)
					.join(&lib_name)
					.to_str()
					.expect("We are converting valid strs to PathBuf then back, it should not fail")
					.to_owned()
			});

		Pdfium::bind_to_library(lib_path)
			.or_else(|err| {
				error!("{err:#?}");
				Pdfium::bind_to_system_library()
			})
			.map(Pdfium::new)
			.map_err(|err| error!("{err:#?}"))
			.ok()
	};
}

pub struct PdfHandler {}

impl ImageHandler for PdfHandler {
	fn handle_image(&self, path: &Path) -> Result<DynamicImage> {
		PDFIUM.with(|maybe_pdfium| {
			let pdfium = maybe_pdfium.as_ref().ok_or(PdfiumBinding)?;

			let render_config = PdfRenderConfig::new()
				.set_target_width(PDF_RENDER_WIDTH)
				.rotate_if_landscape(PdfPageRenderRotation::Degrees90, true);

			Ok(pdfium
				.load_pdf_from_file(path, None)?
				.pages()
				.first()?
				.render_with_config(&render_config)?
				.as_image())
		})
	}
}
