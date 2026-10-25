//! Decides whether a launch shows the window.

/// The flag the autostart entry passes, so signing in starts the app in the tray.
pub const BACKGROUND_FLAG: &str = "--background";

pub fn is_background_launch(arguments: &[String]) -> bool {
    arguments
        .iter()
        .skip(1)
        .any(|argument| argument == BACKGROUND_FLAG)
}

/// A launch by hand shows the window. A background launch stays in the tray, unless there's no tray to
/// bring the window back from.
pub fn should_show_window(background: bool, has_tray: bool) -> bool {
    !background || !has_tray
}
