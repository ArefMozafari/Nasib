//! The GTK side of the app. Everything here only draws and forwards clicks; the decisions live in the
//! library's tested modules.

pub mod app_model;
pub mod settings_tab;
pub mod status_tab;
pub mod tray;
pub mod window;

use gtk::gdk;

const STYLE: &str = include_str!("style.css");

/// Loads the app's colors and classes for the whole display.
pub fn load_style() {
    let Some(display) = gdk::Display::default() else { return };
    let provider = gtk::CssProvider::new();
    provider.load_from_data(STYLE);
    gtk::style_context_add_provider_for_display(&display, &provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
}
