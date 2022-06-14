use super::native::NSObject;
use swift_rs::*;

pub_swift_fn!(lock_app_theme(theme_type: Int));
pub_swift_fn!(blur_window_background(window: NSObject));
pub_swift_fn!(add_invisible_toolbar(window: NSObject, shown: Bool));
pub_swift_fn!(set_transparent_titlebar(
	window: NSObject,
	transparent: Bool,
	large: Bool
));
