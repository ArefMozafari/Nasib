//! The main window: the Status and Settings tabs, each with its own width.

use std::rc::Rc;

use gtk::prelude::*;
use nasib_linux::layout;
use nasib_linux::text;

use super::app_model::AppModel;
use super::{settings_tab, status_tab};

const STATUS_PAGE: &str = "status";
const SETTINGS_PAGE: &str = "settings";
/// Lets a tab take its content's own height.
const NATURAL_HEIGHT: i32 = -1;

/// Builds the window. With a tray, closing it only hides it and the app keeps running; without one,
/// closing it quits, since nothing else could bring it back.
pub fn build(
    application: &gtk::Application,
    model: &Rc<AppModel>,
    update_button: &gtk::Button,
    login_switch: &gtk::Switch,
    hide_on_close: bool,
) -> gtk::ApplicationWindow {
    // Each tab's padding sits outside its size request, so the request leaves room for it.
    let status = status_tab::build(model, update_button, login_switch);
    status.set_size_request(layout::STATUS_WIDTH - 2 * layout::PADDING, NATURAL_HEIGHT);
    let settings = settings_tab::build(model);
    settings.set_size_request(layout::SETTINGS_WIDTH - 2 * layout::PADDING, NATURAL_HEIGHT);
    // A stack that isn't homogeneous takes the shown tab's size, and the fixed-size window follows it,
    // so Status stays narrow while Settings is wide.
    let stack = gtk::Stack::builder()
        .hhomogeneous(false)
        .vhomogeneous(false)
        .build();
    stack.add_titled(&status, Some(STATUS_PAGE), text::STATUS_TAB);
    stack.add_titled(&settings, Some(SETTINGS_PAGE), text::SETTINGS_TAB);
    let switcher = gtk::StackSwitcher::builder()
        .stack(&stack)
        .halign(gtk::Align::Center)
        .margin_top(layout::ROW_SPACING)
        .build();
    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&switcher);
    content.append(&stack);
    gtk::ApplicationWindow::builder()
        .application(application)
        .title(text::APP_NAME)
        .icon_name(nasib_linux::APP_ICON_NAME)
        .resizable(false)
        .hide_on_close(hide_on_close)
        .child(&content)
        .build()
}
