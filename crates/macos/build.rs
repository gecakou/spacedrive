use std::env;

fn main() {
	if cfg!(target_os = "macos") {
		let deployment_target =
			env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| String::from("10.15"));

		swift_rs::SwiftLinker::new(deployment_target.as_str())
			.with_package("sd-macos", "./")
			.link()
	}
}
