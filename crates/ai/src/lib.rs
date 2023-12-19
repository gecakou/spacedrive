use std::path::Path;

use ort::{EnvironmentBuilder, LoggingLevel};
use thiserror::Error;
use tracing::{debug, error};

pub mod image_labeler;
mod utils;

// This path must be relative to the running binary
#[cfg(windows)]
const BINDING_LOCATION: &str = ".";
#[cfg(unix)]
const BINDING_LOCATION: &str = if cfg!(target_os = "macos") {
	"../Frameworks/Spacedrive.framework/Libraries"
} else {
	"../lib/spacedrive"
};

#[cfg(target_os = "windows")]
const LIB_NAME: &str = "onnxruntime.dll";

#[cfg(any(target_os = "macos", target_os = "ios"))]
const LIB_NAME: &str = "libonnxruntime.dylib";

#[cfg(any(target_os = "linux", target_os = "android"))]
const LIB_NAME: &str = "libonnxruntime.so";

pub fn init() -> Result<(), Error> {
	let path = utils::get_path_relative_to_exe(Path::new(BINDING_LOCATION).join(LIB_NAME));

	std::env::set_var("ORT_DYLIB_PATH", path);

	// Initialize AI stuff
	EnvironmentBuilder::default()
		.with_name("spacedrive")
		.with_log_level(if cfg!(debug_assertions) {
			LoggingLevel::Verbose
		} else {
			LoggingLevel::Info
		})
		.with_execution_providers({
			#[cfg(any(target_os = "macos", target_os = "ios"))]
			{
				use ort::{CoreMLExecutionProvider, XNNPACKExecutionProvider};

				[
					CoreMLExecutionProvider::default().build(),
					XNNPACKExecutionProvider::default().build(),
				]
			}

			#[cfg(target_os = "windows")]
			{
				use ort::DirectMLExecutionProvider;

				[DirectMLExecutionProvider::default().build()]
			}

			#[cfg(target_os = "linux")]
			{
				use ort::XNNPACKExecutionProvider;

				[XNNPACKExecutionProvider::default().build()]
			}

			// #[cfg(target_os = "android")]
			// {
			// 	use ort::{
			// 		ACLExecutionProvider, ArmNNExecutionProvider, NNAPIExecutionProvider,
			// 		QNNExecutionProvider, XNNPACKExecutionProvider,
			// 	};
			// 	[
			// 		QNNExecutionProvider::default().build(),
			// 		NNAPIExecutionProvider::default().build(),
			// 		XNNPACKExecutionProvider::default().build(),
			// 		ACLExecutionProvider::default().build(),
			// 		ArmNNExecutionProvider::default().build(),
			// 	]
			// }
		})
		.commit()?;
	debug!("Initialized AI environment");

	Ok(())
}

#[derive(Error, Debug)]
pub enum Error {
	#[error("failed to initialize AI environment: {0}")]
	Init(#[from] ort::Error),
	#[error(transparent)]
	ImageLabeler(#[from] image_labeler::ImageLabelerError),
}
