//! The tray icon and its menu, through the StatusNotifierItem protocol that KDE, Cinnamon, Xfce, MATE and
//! Ubuntu's GNOME show. It runs on its own thread; its clicks reach the main thread as `TrayAction`s.

use ksni::blocking::{Handle, TrayMethods};
use ksni::menu::StandardItem;
use ksni::MenuItem;
use nasib_linux::controller::{ConnectionState, Controller};
use nasib_linux::text;

const TRAY_ID: &str = "nasib";
// Mid-gray glyphs that read on light and dark panels; see `packaging/package.sh`.
const CONNECTED_ICON: &str = "nasib-connected";
const DOWN_ICON: &str = "nasib-down";
const SEARCHING_ICON: &str = "nasib-searching";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayAction {
    Open,
    FindNow,
    Stop,
    Update,
    About,
    Quit,
}

pub struct Tray {
    title: String,
    icon: &'static str,
    is_running_search: bool,
    is_busy: bool,
    update_version: Option<String>,
    sender: async_channel::Sender<TrayAction>,
}

/// Unknown shows as searching until the first status read lands.
fn icon_for(state: &ConnectionState) -> &'static str {
    match state {
        ConnectionState::Unknown | ConnectionState::Searching { .. } => SEARCHING_ICON,
        ConnectionState::Down => DOWN_ICON,
        ConnectionState::Connected { .. } => CONNECTED_ICON,
    }
}

impl Tray {
    fn item(&self, label: &str, action: TrayAction, enabled: bool) -> MenuItem<Tray> {
        StandardItem {
            label: label.to_string(),
            enabled,
            activate: Box::new(move |tray: &mut Tray| {
                let _ = tray.sender.send_blocking(action.clone());
            }),
            ..Default::default()
        }
        .into()
    }
}

impl ksni::Tray for Tray {
    fn id(&self) -> String {
        TRAY_ID.to_string()
    }

    fn title(&self) -> String {
        text::APP_NAME.to_string()
    }

    fn icon_name(&self) -> String {
        self.icon.to_string()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        ksni::ToolTip {
            title: text::APP_NAME.to_string(),
            description: self.title.clone(),
            ..Default::default()
        }
    }

    /// A left click opens the window, as the menu's first item does.
    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.sender.send_blocking(TrayAction::Open);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let mut items = vec![
            StandardItem {
                label: self.title.clone(),
                enabled: false,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            self.item(text::OPEN_APP, TrayAction::Open, true),
        ];
        if self.is_running_search {
            items.push(self.item(text::STOP_SEARCH, TrayAction::Stop, true));
        } else {
            items.push(self.item(text::FIND_NOW, TrayAction::FindNow, !self.is_busy));
        }
        items.push(MenuItem::Separator);
        if let Some(version) = &self.update_version {
            items.push(self.item(
                &format!("{}…", text::update_to(version)),
                TrayAction::Update,
                true,
            ));
        }
        items.push(self.item(text::ABOUT_APP, TrayAction::About, true));
        items.push(self.item(text::QUIT_APP, TrayAction::Quit, true));
        items
    }
}

/// Starts the tray, or returns `None` when the desktop has no tray host to show it.
pub fn spawn(sender: async_channel::Sender<TrayAction>) -> Option<Handle<Tray>> {
    Tray {
        title: text::CHECKING.to_string(),
        icon: SEARCHING_ICON,
        is_running_search: false,
        is_busy: false,
        update_version: None,
        sender,
    }
    .spawn()
    .ok()
}

/// Mirrors the controller in the icon and menu.
pub fn show(handle: &Handle<Tray>, controller: &Controller) {
    let title = text::state_title(&controller.state);
    let icon = icon_for(&controller.state);
    let is_running_search = controller.is_finding || controller.is_surveying;
    let is_busy = !controller.can_run();
    handle.update(move |tray| {
        tray.title = title;
        tray.icon = icon;
        tray.is_running_search = is_running_search;
        tray.is_busy = is_busy;
    });
}

pub fn show_update(handle: &Handle<Tray>, version: String) {
    handle.update(move |tray| tray.update_version = Some(version));
}
