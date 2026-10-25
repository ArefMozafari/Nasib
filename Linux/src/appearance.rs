//! Follows the desktop's light or dark preference through the settings portal. GTK 4.6 doesn't read it
//! by itself, so without this the window stays light on a dark desktop.

use std::thread;

use gtk::glib;
use zbus::zvariant::{OwnedValue, Value};

const PORTAL_DESTINATION: &str = "org.freedesktop.portal.Desktop";
const PORTAL_PATH: &str = "/org/freedesktop/portal/desktop";
const SETTINGS_INTERFACE: &str = "org.freedesktop.portal.Settings";
const APPEARANCE_NAMESPACE: &str = "org.freedesktop.appearance";
const COLOR_SCHEME_KEY: &str = "color-scheme";
const READ_ONE_METHOD: &str = "ReadOne";
/// The older method, which wraps the value in one more variant.
const READ_METHOD: &str = "Read";
const SETTING_CHANGED_SIGNAL: &str = "SettingChanged";
const PREFERS_DARK: u32 = 1;
const PREFERS_LIGHT: u32 = 2;

/// Reads the portal's `color-scheme`: 1 asks for dark, 2 for light, and 0 or anything unknown leaves the
/// theme's own default.
pub fn color_scheme_prefers_dark(value: u32) -> Option<bool> {
    match value {
        PREFERS_DARK => Some(true),
        PREFERS_LIGHT => Some(false),
        _ => None,
    }
}

/// Unwraps the variants around the number, however many the portal version adds.
fn number(value: &Value) -> Option<u32> {
    match value {
        Value::U32(number) => Some(*number),
        Value::Value(inner) => number(inner),
        _ => None,
    }
}

fn read(proxy: &zbus::blocking::Proxy) -> Option<u32> {
    let arguments = (APPEARANCE_NAMESPACE, COLOR_SCHEME_KEY);
    let value: OwnedValue = proxy
        .call(READ_ONE_METHOD, &arguments)
        .or_else(|_| proxy.call(READ_METHOD, &arguments))
        .ok()?;
    number(&value)
}

/// Sends the current preference, then every change, until the portal goes away.
fn listen(sender: async_channel::Sender<Option<bool>>) -> zbus::Result<()> {
    let connection = zbus::blocking::Connection::session()?;
    let proxy = zbus::blocking::Proxy::new(&connection, PORTAL_DESTINATION, PORTAL_PATH, SETTINGS_INTERFACE)?;
    if let Some(value) = read(&proxy) {
        let _ = sender.send_blocking(color_scheme_prefers_dark(value));
    }
    for message in proxy.receive_signal(SETTING_CHANGED_SIGNAL)? {
        let Ok((namespace, key, value)) = message.body().deserialize::<(String, String, OwnedValue)>() else {
            continue;
        };
        if namespace == APPEARANCE_NAMESPACE && key == COLOR_SCHEME_KEY {
            if let Some(value) = number(&value) {
                if sender.send_blocking(color_scheme_prefers_dark(value)).is_err() {
                    break;
                }
            }
        }
    }
    Ok(())
}

/// Calls `apply` on the main thread with the desktop's preference now and whenever it changes; `None`
/// means no preference. Without a portal it never calls, and GTK's default theme stays.
pub fn watch(apply: impl Fn(Option<bool>) + 'static) {
    let (sender, receiver) = async_channel::unbounded();
    thread::spawn(move || {
        let _ = listen(sender);
    });
    glib::spawn_future_local(async move {
        while let Ok(prefers_dark) = receiver.recv().await {
            apply(prefers_dark);
        }
    });
}
