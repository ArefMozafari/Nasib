//! Every sentence the app shows. The macOS and Windows apps use the same wording.

use crate::controller::ConnectionState;

pub const APP_NAME: &str = "Nasib";
pub const STATUS_TAB: &str = "Status";
pub const SETTINGS_TAB: &str = "Settings";

pub const CHECKING: &str = "Checking…";
pub const NOT_CONNECTED: &str = "Not connected";
pub const ENGINE_MISSING: &str = "Engine missing from the app bundle.";

pub const FIND_NOW: &str = "Find Now";
pub const STOP: &str = "Stop";
pub const START_SURVEY: &str = "Start Survey";
pub const STOP_SURVEY: &str = "Stop Survey";
pub const SHOW_LOG: &str = "Show Log";
pub const AUTO_RECONNECT: &str = "Auto-reconnect when the tunnel drops";
pub const LAUNCH_AT_LOGIN: &str = "Launch at login";

pub const OPEN_APP: &str = "Open Nasib";
pub const STOP_SEARCH: &str = "Stop Search";
pub const QUIT_APP: &str = "Quit Nasib";
pub const ABOUT_APP: &str = "About Nasib";
pub const APP_SUMMARY: &str = "Keeps Windscribe connected from networks that block most of it.";
pub const WEBSITE: &str = "https://github.com/ArefMozafari/Nasib";

pub const CONNECTED_TITLE: &str = "Connected";
pub const NOTHING_WORKED_TITLE: &str = "No working server";
pub const NOTHING_WORKED_BODY: &str = "Every datacenter and protocol failed.";
pub const TUNNEL_DOWN_TITLE: &str = "Tunnel down";
pub const TUNNEL_DOWN_BODY: &str = "Searching for a working server…";
pub const UPDATE_AVAILABLE_TITLE: &str = "Update available";

pub const LOCATIONS_HEADING: &str = "Locations, in the order to try";
pub const LOCATIONS_HINT: &str = "Drag to reorder. Uncheck one to stop trying it.";
pub const YOUR_ORDER: &str = "Your order";
pub const ADD_LOCATIONS: &str = "Add locations";
pub const SEARCH_ORDER: &str = "Search your order";
pub const SEARCH_CATALOG: &str = "Search cities, nicknames or regions";
pub const RANK_BY_SUCCESS: &str = "Rank by recent success";
pub const SELECT_ALL: &str = "Select all";
pub const DESELECT_ALL: &str = "Deselect all";
pub const PROTOCOLS_HEADING: &str = "Protocols, in the order to try";
pub const PROTOCOLS_HINT: &str =
    "Tried in exactly this order. Drag to reorder; unchecked protocols are never tried.";
pub const RESET_TO_DEFAULTS: &str = "Reset to Defaults";
pub const SAVE: &str = "Save";
pub const SAVED: &str = "Saved. The next search uses these settings.";
pub const NO_LOCATION_PROBLEM: &str = "Check at least one location.";
pub const NO_PROTOCOL_PROBLEM: &str = "Check at least one protocol.";
pub const UNREADABLE_CATALOG: &str = "Couldn't read the datacenter list.";
pub const CITIES_KEPT: &str =
    "Couldn't read the datacenter list, so your saved cities stay as they are.";

/// The line under the catalog column, such as "12 of 200 checked".
pub fn checked_count(checked: usize, total: usize) -> String {
    format!("{checked} of {total} checked")
}

/// Hover help for every control.
pub mod tooltip {
    pub const FIND_NOW: &str =
        "Search for a working server and connect to the first one that loads a blocked site.";
    pub const STOP_FIND: &str = "Stop searching and disconnect.";
    pub const START_SURVEY: &str = "Try every server on every protocol and port and record the results, without stopping at the first success.";
    pub const STOP_SURVEY: &str = "Stop the survey and reconnect to the last server that worked.";
    pub const SHOW_LOG: &str = "Open the list of every connection attempt.";
    pub const AUTO_RECONNECT: &str = "Check the tunnel every 3 seconds and a blocked site every minute, and search for a new server when either fails.";
    pub const LAUNCH_AT_LOGIN: &str = "Start Nasib in the background when you sign in.";
    pub const SAVE: &str = "Use these locations and protocols from the next search on.";
    pub const UPDATE: &str = "Open the release page to download the new version.";
    pub const RESET_TO_DEFAULTS: &str = "Restore the default locations, ranking and protocol order. Nothing changes until you save.";
    pub const RANK_BY_SUCCESS: &str = "Try the checked locations in the order that worked best lately, instead of exactly your order.";
}

/// The server line on the Status tab and in the tray menu.
pub fn state_title(state: &ConnectionState) -> String {
    match state {
        ConnectionState::Unknown => CHECKING.to_string(),
        ConnectionState::Down => NOT_CONNECTED.to_string(),
        ConnectionState::Searching { location, protocol } => {
            format!("Trying {location} / {protocol}")
        }
        ConnectionState::Connected { location, protocol } => format!("{location} / {protocol}"),
    }
}

pub fn update_to(version: &str) -> String {
    format!("Update to {version}")
}

pub fn update_ready(version: &str) -> String {
    format!("Nasib {version} is ready to download.")
}
