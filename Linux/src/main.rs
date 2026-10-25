//! `nasib`: the Linux app. One instance runs at a time: a launch by hand while it runs shows the running
//! one's window, and a launch from autostart does nothing.

mod ui;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;
use gtk::{gio, glib};
use ksni::blocking::Handle;
use nasib_linux::launch::{is_background_launch, should_show_window};
use nasib_linux::update_checker::{self, Release, CHECK_INTERVAL_SECONDS, CURRENT_VERSION};
use nasib_linux::{appearance, login_item, notifier, text, APPLICATION_ID, APP_ICON_NAME};

use ui::app_model::AppModel;
use ui::tray::{self, Tray, TrayAction};

/// Everything the running app keeps, created on the first activation.
struct Running {
    model: Rc<AppModel>,
    window: gtk::ApplicationWindow,
    tray: Option<Handle<Tray>>,
    release: RefCell<Option<Release>>,
    /// Keeps the app running while its window is hidden in the tray.
    _hold: Option<gio::ApplicationHoldGuard>,
}

fn open_uri(uri: &str) {
    let _ = gio::AppInfo::launch_default_for_uri(uri, None::<&gio::AppLaunchContext>);
}

fn show_about(window: &gtk::ApplicationWindow) {
    gtk::AboutDialog::builder()
        .transient_for(window)
        .modal(true)
        .program_name(text::APP_NAME)
        .version(CURRENT_VERSION)
        .comments(text::APP_SUMMARY)
        .website(text::WEBSITE)
        .logo_icon_name(APP_ICON_NAME)
        .build()
        .present();
}

fn handle_tray_action(application: &gtk::Application, running: &Running, action: TrayAction) {
    match action {
        TrayAction::Open => running.window.present(),
        TrayAction::FindNow => running.model.find_now(),
        TrayAction::Stop => running.model.stop(),
        TrayAction::Update => {
            if let Some(release) = running.release.borrow().as_ref() {
                open_uri(&release.page);
            }
        }
        TrayAction::About => show_about(&running.window),
        TrayAction::Quit => application.quit(),
    }
}

/// Offers a newer release in the window and the tray, and notifies once per version.
fn show_release(running: &Running, update_button: &gtk::Button, release: Release) {
    update_button.set_label(&text::update_to(&release.version));
    update_button.set_visible(true);
    if let Some(tray) = running.tray.as_ref() {
        tray::show_update(tray, release.version.clone());
    }
    let mut preferences = running.model.preferences.borrow_mut();
    if update_checker::should_notify(&release.version, preferences.notified_update_version()) {
        preferences.set_notified_update_version(&release.version);
        notifier::post(nasib_linux::controller::Notification {
            title: text::UPDATE_AVAILABLE_TITLE.to_string(),
            body: text::update_ready(&release.version),
        });
    }
    running.release.replace(Some(release));
}

fn check_for_update(running: &Rc<Running>, update_button: &gtk::Button) {
    let running = Rc::downgrade(running);
    let update_button = update_button.clone();
    glib::spawn_future_local(async move {
        let release = gio::spawn_blocking(update_checker::check).await.ok().flatten();
        if let (Some(running), Some(release)) = (running.upgrade(), release) {
            show_release(&running, &update_button, release);
        }
    });
}

fn login_switch() -> gtk::Switch {
    let directory = login_item::configuration_directory();
    let switch = gtk::Switch::new();
    switch.set_active(login_item::is_enabled(&directory));
    switch.set_valign(gtk::Align::Center);
    let executable = std::env::current_exe().unwrap_or_default();
    switch.connect_state_set(move |switch, enabled| {
        if login_item::set_enabled(&directory, enabled, &executable).is_err() {
            switch.set_active(login_item::is_enabled(&directory));
            return glib::Propagation::Stop;
        }
        glib::Propagation::Proceed
    });
    switch
}

fn start(application: &gtk::Application, background: bool) -> Rc<Running> {
    ui::load_style();
    if let Some(settings) = gtk::Settings::default() {
        // "No preference" from the desktop goes back to what the user's own GTK settings asked for.
        let own_choice = settings.is_gtk_application_prefer_dark_theme();
        appearance::watch(move |prefers_dark| {
            settings.set_gtk_application_prefer_dark_theme(prefers_dark.unwrap_or(own_choice));
        });
    }
    let model = AppModel::new();
    let (tray_sender, tray_actions) = async_channel::unbounded();
    let tray = tray::spawn(tray_sender);
    let update_button = gtk::Button::new();
    let window = ui::window::build(application, &model, &update_button, &login_switch(), tray.is_some());
    let running = Rc::new(Running {
        model: model.clone(),
        window,
        _hold: tray.as_ref().map(|_| application.hold()),
        tray,
        release: RefCell::new(None),
    });

    let weak = Rc::downgrade(&running);
    model.subscribe(move || {
        if let Some(running) = weak.upgrade() {
            if let Some(tray) = running.tray.as_ref() {
                tray::show(tray, &running.model.controller.borrow());
            }
        }
    });
    let weak = Rc::downgrade(&running);
    let action_application = application.clone();
    glib::spawn_future_local(async move {
        while let Ok(action) = tray_actions.recv().await {
            let Some(running) = weak.upgrade() else { break };
            handle_tray_action(&action_application, &running, action);
        }
    });
    let weak = Rc::downgrade(&running);
    update_button.connect_clicked(move |_| {
        if let Some(release) = weak.upgrade().and_then(|running| running.release.borrow().clone()) {
            open_uri(&release.page);
        }
    });
    check_for_update(&running, &update_button);
    let weak = Rc::downgrade(&running);
    glib::timeout_add_seconds_local(CHECK_INTERVAL_SECONDS, move || match weak.upgrade() {
        Some(running) => {
            check_for_update(&running, &update_button);
            glib::ControlFlow::Continue
        }
        None => glib::ControlFlow::Break,
    });

    if should_show_window(background, running.tray.is_some()) {
        running.window.present();
    }
    running
}

fn main() -> glib::ExitCode {
    let arguments: Vec<String> = std::env::args().collect();
    let background = is_background_launch(&arguments);
    let application = gtk::Application::builder().application_id(APPLICATION_ID).build();
    // Autostart while Nasib already runs leaves the running one as it is.
    if background && application.register(None::<&gio::Cancellable>).is_ok() && application.is_remote() {
        return glib::ExitCode::SUCCESS;
    }
    let running: Rc<RefCell<Option<Rc<Running>>>> = Rc::new(RefCell::new(None));
    application.connect_activate(move |application| {
        let existing = running.borrow().clone();
        match existing {
            Some(running) => running.window.present(),
            None => {
                running.replace(Some(start(application, background)));
            }
        }
    });
    // GTK would refuse the flag it doesn't know, so it only gets the program name.
    application.run_with_args(&arguments[..1])
}
