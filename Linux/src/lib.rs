//! Nasib for Linux: a GTK 4 app around the shared `nasib-engine`. The modules here hold everything that
//! isn't drawing, so it can be tested without a display.

/// The D-Bus name that makes a second launch activate the first.
pub const APPLICATION_ID: &str = "com.arefmozafari.nasib";
/// The icon installed under `share/icons/hicolor`.
pub const APP_ICON_NAME: &str = "nasib";
