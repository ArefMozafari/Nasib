//! The running app's state on the main thread: the controller, the engine run and the timers. Widgets
//! subscribe to hear about every change and read the controller to redraw.

use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::{Rc, Weak};

use gtk::{gio, glib};
use nasib_engine::paths::Paths;
use nasib_linux::controller::Controller;
use nasib_linux::engine_process::{engine_path, EngineProcess};
use nasib_linux::notifier;
use nasib_linux::preferences::Preferences;

const STATUS_ARGUMENT: &str = "--status";
const CATALOG_ARGUMENT: &str = "--catalog";
const SURVEY_ARGUMENT: &str = "--survey";
const WATCH_ARGUMENT: &str = "--watch";
/// Reading the CLI status costs no traffic, so it runs far more often than the blocked-site check.
const STATUS_INTERVAL_SECONDS: u32 = 3;
const WATCH_INTERVAL_SECONDS: u32 = 60;

/// What the engine's threads send to the main thread.
enum EngineMessage {
    Line(String),
    Exited,
}

pub struct AppModel {
    pub controller: RefCell<Controller>,
    pub preferences: RefCell<Preferences>,
    pub paths: Paths,
    engine: Option<PathBuf>,
    process: RefCell<Option<EngineProcess>>,
    is_refreshing: Cell<bool>,
    listeners: RefCell<Vec<Box<dyn Fn()>>>,
    sender: async_channel::Sender<EngineMessage>,
}

impl AppModel {
    fn changed(&self) {
        for listener in self.listeners.borrow().iter() {
            listener();
        }
    }

    fn handle(&self, message: EngineMessage) {
        match message {
            EngineMessage::Line(line) => {
                let notification = self.controller.borrow_mut().handle_line(&line);
                if let Some(notification) = notification {
                    notifier::post(notification);
                }
            }
            EngineMessage::Exited => {
                self.process.replace(None);
                self.controller.borrow_mut().engine_exited();
            }
        }
        self.changed();
    }

    /// Delivers the engine's lines on the main thread for as long as the model lives.
    fn receive(model: Weak<AppModel>, receiver: async_channel::Receiver<EngineMessage>) {
        glib::spawn_future_local(async move {
            while let Ok(message) = receiver.recv().await {
                let Some(model) = model.upgrade() else { break };
                let exited = matches!(message, EngineMessage::Exited);
                model.handle(message);
                if exited {
                    model.refresh_status();
                }
            }
        });
    }

    fn start_timers(model: &Rc<AppModel>) {
        let weak = Rc::downgrade(model);
        glib::timeout_add_seconds_local(STATUS_INTERVAL_SECONDS, move || match weak.upgrade() {
            Some(model) => {
                model.refresh_status();
                glib::ControlFlow::Continue
            }
            None => glib::ControlFlow::Break,
        });
        let weak = Rc::downgrade(model);
        glib::timeout_add_seconds_local(WATCH_INTERVAL_SECONDS, move || match weak.upgrade() {
            Some(model) => {
                model.watch_tick();
                glib::ControlFlow::Continue
            }
            None => glib::ControlFlow::Break,
        });
    }

    pub fn new() -> Rc<AppModel> {
        let paths = Paths::for_current_user();
        let engine = engine_path();
        let (sender, receiver) = async_channel::unbounded();
        let model = Rc::new(AppModel {
            controller: RefCell::new(Controller::new(engine.is_none())),
            preferences: RefCell::new(Preferences::load(&paths.state_directory)),
            paths,
            engine,
            process: RefCell::new(None),
            is_refreshing: Cell::new(false),
            listeners: RefCell::new(Vec::new()),
            sender,
        });
        AppModel::receive(Rc::downgrade(&model), receiver);
        AppModel::start_timers(&model);
        model.refresh_status();
        model
    }

    pub fn subscribe(&self, listener: impl Fn() + 'static) {
        self.listeners.borrow_mut().push(Box::new(listener));
    }

    /// Starts the engine when nothing else runs; `begin` marks what kind of run it is.
    fn run(&self, arguments: &[&str], begin: fn(&mut Controller)) {
        let Some(engine) = self.engine.as_ref() else { return };
        if !self.controller.borrow().can_run() {
            return;
        }
        let line_sender = self.sender.clone();
        let exit_sender = self.sender.clone();
        let started = EngineProcess::start(
            engine,
            arguments,
            move |line| {
                let _ = line_sender.send_blocking(EngineMessage::Line(line));
            },
            move |_| {
                let _ = exit_sender.send_blocking(EngineMessage::Exited);
            },
        );
        match started {
            Ok(process) => {
                self.process.replace(Some(process));
                begin(&mut self.controller.borrow_mut());
            }
            Err(error) => {
                self.controller
                    .borrow_mut()
                    .handle_line(&format!("could not start engine: {error}"));
            }
        }
        self.changed();
    }

    pub fn find_now(&self) {
        self.run(&[], Controller::begin_find);
    }

    pub fn start_survey(&self) {
        self.run(&[SURVEY_ARGUMENT], Controller::begin_survey);
    }

    /// Runs the engine's watch mode, which exits at once when the tunnel is healthy.
    pub fn watch_tick(&self) {
        if self.preferences.borrow().auto_reconnect() {
            self.run(&[WATCH_ARGUMENT], Controller::begin_watch);
        }
    }

    /// A stopped find disconnects; a stopped survey reconnects the last server that worked.
    pub fn stop(&self) {
        if let Some(process) = self.process.borrow_mut().as_mut() {
            process.stop();
        }
    }

    pub fn set_auto_reconnect(&self, enabled: bool) {
        self.preferences.borrow_mut().set_auto_reconnect(enabled);
    }

    /// Reads the current tunnel off the main thread without touching it, and starts the watch at once
    /// when the tunnel just dropped, instead of waiting for the next minute's check.
    pub fn refresh_status(self: &Rc<Self>) {
        let Some(engine) = self.engine.clone() else { return };
        if self.is_refreshing.get() || self.controller.borrow().is_busy {
            return;
        }
        self.is_refreshing.set(true);
        let model = Rc::downgrade(self);
        glib::spawn_future_local(async move {
            let line = gio::spawn_blocking(move || EngineProcess::run_once(&engine, STATUS_ARGUMENT))
                .await
                .unwrap_or_default();
            let Some(model) = model.upgrade() else { return };
            model.is_refreshing.set(false);
            let update = model.controller.borrow_mut().apply_status(&line);
            if update.changed {
                model.changed();
            }
            if update.dropped {
                model.watch_tick();
            }
        });
    }

    /// Reads the engine's datacenter catalog off the main thread; empty when the engine can't run.
    pub async fn read_catalog(&self) -> String {
        let Some(engine) = self.engine.clone() else { return String::new() };
        gio::spawn_blocking(move || EngineProcess::run_once(&engine, CATALOG_ARGUMENT))
            .await
            .unwrap_or_default()
    }
}
