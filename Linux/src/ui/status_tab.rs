//! The Status tab: the server line, the run buttons, the two switches and the recent engine output.

use std::cell::Cell;
use std::fs::{self, OpenOptions};
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};
use nasib_linux::controller::{is_striped, ConnectionState, Controller};
use nasib_linux::layout;
use nasib_linux::output_parser::{parse, OutputTone};
use nasib_linux::text::{self, tooltip};
use nasib_linux::APP_ICON_NAME;

use super::app_model::AppModel;

const DOT_CLASSES: [&str; 3] = ["down", "searching", "connected"];

/// The CSS class that colors a line of engine output.
fn tone_class(tone: OutputTone) -> &'static str {
    match tone {
        OutputTone::Primary => "tone-primary",
        OutputTone::Success => "tone-success",
        OutputTone::Muted => "tone-muted",
        OutputTone::Warning => "tone-warning",
        OutputTone::Error => "tone-error",
        OutputTone::Accent => "tone-accent",
        OutputTone::Detail => "tone-detail",
        OutputTone::Phase => "tone-phase",
    }
}

fn dot_class(state: &ConnectionState) -> Option<&'static str> {
    match state {
        ConnectionState::Unknown => None,
        ConnectionState::Down => Some("down"),
        ConnectionState::Searching { .. } => Some("searching"),
        ConnectionState::Connected { .. } => Some("connected"),
    }
}

/// Opens the attempt log in the user's default app, creating it first so there's something to open.
fn show_log(model: &AppModel) {
    let log = model.paths.log_file();
    let _ = fs::create_dir_all(&model.paths.state_directory);
    let _ = OpenOptions::new().create(true).append(true).open(&log);
    let uri = gio::File::for_path(&log).uri();
    let _ = gio::AppInfo::launch_default_for_uri(&uri, None::<&gio::AppLaunchContext>);
}

fn header(update_button: &gtk::Button) -> gtk::Box {
    let icon = gtk::Image::from_icon_name(APP_ICON_NAME);
    icon.set_pixel_size(layout::HEADER_ICON_SIZE);
    let name = gtk::Label::new(Some(text::APP_NAME));
    name.add_css_class("app-name");
    name.set_hexpand(true);
    name.set_xalign(0.0);
    let row = gtk::Box::new(gtk::Orientation::Horizontal, layout::ROW_SPACING);
    row.append(&icon);
    row.append(&name);
    update_button.set_tooltip_text(Some(tooltip::UPDATE));
    update_button.add_css_class("suggested-action");
    update_button.set_visible(false);
    row.append(update_button);
    row
}

fn log_row(line: &str, striped: bool) -> gtk::Label {
    let label = gtk::Label::new(Some(line));
    label.set_xalign(0.0);
    label.set_wrap(true);
    label.set_selectable(true);
    label.add_css_class("log-line");
    label.add_css_class(tone_class(parse(line).tone()));
    if striped {
        label.add_css_class("striped");
    }
    label
}

/// Appends the lines the pane hasn't shown yet and drops the ones the controller no longer keeps, so
/// existing rows keep their place and any selection. It follows the newest line only when the pane was
/// already at the bottom, so reading further up isn't interrupted.
fn show_new_lines(
    list: &gtk::Box,
    scroller: &gtk::ScrolledWindow,
    controller: &Controller,
    shown_through: &Cell<usize>,
) {
    let new_count = controller.new_line_count(shown_through.get());
    if new_count == 0 {
        return;
    }
    let adjustment = scroller.vadjustment();
    let was_at_bottom = adjustment.value() + adjustment.page_size() >= adjustment.upper();
    let first_new = controller.recent_lines.len() - new_count;
    for (offset, line) in controller.recent_lines.iter().skip(first_new).enumerate() {
        // Stripes count from the first line ever shown, so a row's stripe never changes.
        let absolute_index = controller.dropped_line_count + first_new + offset;
        list.append(&log_row(line, is_striped(absolute_index, 0)));
    }
    let mut row_count = 0;
    let mut child = list.first_child();
    while let Some(row) = child {
        row_count += 1;
        child = row.next_sibling();
    }
    for _ in controller.recent_lines.len()..row_count {
        if let Some(oldest) = list.first_child() {
            list.remove(&oldest);
        }
    }
    shown_through.set(controller.shown_line_total());
    if was_at_bottom {
        glib::idle_add_local_once(move || adjustment.set_value(adjustment.upper()));
    }
}

/// Builds the tab. The update button is returned with it so the update check can show it.
pub fn build(
    model: &Rc<AppModel>,
    update_button: &gtk::Button,
    login_switch: &gtk::Switch,
) -> gtk::Box {
    let tab = gtk::Box::new(gtk::Orientation::Vertical, layout::SECTION_SPACING);
    tab.set_margin_top(layout::PADDING);
    tab.set_margin_bottom(layout::PADDING);
    tab.set_margin_start(layout::PADDING);
    tab.set_margin_end(layout::PADDING);
    tab.append(&header(update_button));

    let dot = gtk::Box::new(gtk::Orientation::Horizontal, 0);
    dot.add_css_class("status-dot");
    dot.set_size_request(layout::STATUS_DOT_SIZE, layout::STATUS_DOT_SIZE);
    dot.set_valign(gtk::Align::Center);
    let server = gtk::Label::new(Some(text::CHECKING));
    server.add_css_class("server-line");
    server.set_ellipsize(gtk::pango::EllipsizeMode::End);
    server.set_hexpand(true);
    server.set_xalign(0.0);
    let spinner = gtk::Spinner::new();
    let server_row = gtk::Box::new(gtk::Orientation::Horizontal, layout::ROW_SPACING);
    server_row.append(&dot);
    server_row.append(&server);
    server_row.append(&spinner);
    tab.append(&server_row);

    let engine_missing = gtk::Label::new(Some(text::ENGINE_MISSING));
    engine_missing.add_css_class("engine-missing");
    engine_missing.set_xalign(0.0);
    tab.append(&engine_missing);

    let find_button = gtk::Button::new();
    let survey_button = gtk::Button::new();
    let log_button = gtk::Button::with_label(text::SHOW_LOG);
    log_button.set_tooltip_text(Some(tooltip::SHOW_LOG));
    log_button.set_hexpand(true);
    log_button.set_halign(gtk::Align::End);
    let action_row = gtk::Box::new(gtk::Orientation::Horizontal, layout::ROW_SPACING);
    action_row.append(&find_button);
    action_row.append(&survey_button);
    action_row.append(&log_button);
    tab.append(&action_row);
    tab.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    let auto_reconnect = gtk::CheckButton::with_label(text::AUTO_RECONNECT);
    auto_reconnect.set_tooltip_text(Some(tooltip::AUTO_RECONNECT));
    auto_reconnect.set_active(model.preferences.borrow().auto_reconnect());
    let launch_at_login = gtk::Box::new(gtk::Orientation::Horizontal, layout::ROW_SPACING);
    let launch_label = gtk::Label::new(Some(text::LAUNCH_AT_LOGIN));
    launch_label.set_hexpand(true);
    launch_label.set_xalign(0.0);
    launch_at_login.append(&launch_label);
    launch_at_login.append(login_switch);
    launch_at_login.set_tooltip_text(Some(tooltip::LAUNCH_AT_LOGIN));
    let switches = gtk::Box::new(gtk::Orientation::Vertical, layout::ROW_SPACING);
    switches.append(&auto_reconnect);
    switches.append(&launch_at_login);
    tab.append(&switches);
    tab.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    let lines = gtk::Box::new(gtk::Orientation::Vertical, 0);
    let scroller = gtk::ScrolledWindow::builder()
        .child(&lines)
        .min_content_height(layout::LOG_HEIGHT)
        .max_content_height(layout::LOG_HEIGHT)
        .hscrollbar_policy(gtk::PolicyType::Never)
        .build();
    tab.append(&scroller);

    let weak = Rc::downgrade(model);
    find_button.connect_clicked(move |_| {
        let Some(model) = weak.upgrade() else { return };
        if model.controller.borrow().is_finding {
            model.stop();
        } else {
            model.find_now();
        }
    });
    let weak = Rc::downgrade(model);
    survey_button.connect_clicked(move |_| {
        let Some(model) = weak.upgrade() else { return };
        if model.controller.borrow().is_surveying {
            model.stop();
        } else {
            model.start_survey();
        }
    });
    let weak = Rc::downgrade(model);
    log_button.connect_clicked(move |_| {
        if let Some(model) = weak.upgrade() {
            show_log(&model);
        }
    });
    let weak = Rc::downgrade(model);
    auto_reconnect.connect_toggled(move |button| {
        if let Some(model) = weak.upgrade() {
            model.set_auto_reconnect(button.is_active());
        }
    });

    let redraw = {
        let weak = Rc::downgrade(model);
        let shown_through = Cell::new(0);
        move || {
            let Some(model) = weak.upgrade() else { return };
            let controller = model.controller.borrow();
            server.set_text(&text::state_title(&controller.state));
            for class in DOT_CLASSES {
                dot.remove_css_class(class);
            }
            if let Some(class) = dot_class(&controller.state) {
                dot.add_css_class(class);
            }
            spinner.set_spinning(controller.is_busy);
            spinner.set_visible(controller.is_busy);
            engine_missing.set_visible(controller.engine_missing);
            if controller.is_finding {
                find_button.set_label(text::STOP);
                find_button.set_tooltip_text(Some(tooltip::STOP_FIND));
                find_button.set_sensitive(true);
            } else {
                find_button.set_label(text::FIND_NOW);
                find_button.set_tooltip_text(Some(tooltip::FIND_NOW));
                find_button.set_sensitive(controller.can_run());
            }
            if controller.is_surveying {
                survey_button.set_label(text::STOP_SURVEY);
                survey_button.set_tooltip_text(Some(tooltip::STOP_SURVEY));
                survey_button.set_sensitive(true);
            } else {
                survey_button.set_label(text::START_SURVEY);
                survey_button.set_tooltip_text(Some(tooltip::START_SURVEY));
                survey_button.set_sensitive(controller.can_run());
            }
            show_new_lines(&lines, &scroller, &controller, &shown_through);
        }
    };
    redraw();
    model.subscribe(redraw);
    tab
}
