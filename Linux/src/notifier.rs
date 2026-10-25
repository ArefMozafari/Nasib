//! Posts desktop notifications through the notification server, when there is one.

use std::thread;

use crate::controller::Notification;
use crate::text::APP_NAME;

/// Posts off the calling thread, since the server can be slow to answer; a missing server is ignored.
pub fn post(notification: Notification) {
    thread::spawn(move || {
        let _ = notify_rust::Notification::new()
            .appname(APP_NAME)
            .summary(&notification.title)
            .body(&notification.body)
            .icon(crate::APP_ICON_NAME)
            .show();
    });
}
