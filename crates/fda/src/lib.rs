// #![doc = include_str!("../README.md")]
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
	clippy::dbg_macro,
	clippy::deprecated_cfg_attr,
	clippy::separated_literal_suffix,
	deprecated
)]
#![forbid(unsafe_code, deprecated_in_future)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

pub mod error;

use dirs::home_dir;
use error::Result;
use std::{fs, path::PathBuf};

pub struct DiskAccess;
const RESTRICTED_PATHS: [&str; 3] = ["Library", "Library/Mail", "Library/Safari"];

impl DiskAccess {
	/// This checks if a path is writeable or not. If not, it is read-only.
	#[must_use]
	pub fn is_path_writeable(path: PathBuf) -> bool {
		!fs::metadata(path).map_or(false, |md| !md.permissions().readonly())
	}

	/// This function checks if a path is readable, or at least exists.
	#[must_use]
	pub fn is_path_readable(path: PathBuf) -> bool {
		fs::metadata(path).is_ok()
	}

	/// This function checks to see if we can actually read "protected" directories that reequire full disk access
	///
	/// It returns `true` on all non-MacOS systems as permissions aren't something we need to worry about there just yet.
	#[must_use]
	fn can_read_library_dirs() -> bool {
		#[cfg(target_os = "macos")]
		{
			home_dir().map_or(false, |home| {
				RESTRICTED_PATHS
					.into_iter()
					.all(|p| fs::read_dir(home.join(p)).is_ok())
			})
		}
		#[cfg(not(target_os = "macos"))]
		{
			true
		}
	}

	/// This function is a no-op on non-MacOS systems.
	///
	/// This checks if we have full disk access available on `MacOS` or not.
	#[must_use]
	pub fn has_fda() -> bool {
		Self::can_read_library_dirs()
	}

	/// This function is a no-op on non-MacOS systems.
	///
	/// Once ran, it will open the "Full Disk Access" prompt.
	pub fn request_fda() -> Result<()> {
		#[cfg(target_os = "macos")]
		{
			use crate::error::Error;
			use std::process::Command;

			Command::new("open")
				.arg("x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles")
				.status()
				.map_err(|_| Error::FDAPromptError)?;
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::DiskAccess;

	#[test]
	#[cfg_attr(miri, ignore = "Miri can't run this test")]
	#[ignore = "CI can't run this due to lack of a GUI"]
	fn macos_open_full_disk_prompt() {
		DiskAccess::request_fda().unwrap();
	}

	#[test]
	fn has_fda() {
		assert!(DiskAccess::has_fda());
	}
}
