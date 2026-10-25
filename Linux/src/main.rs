//! `nasib`: the Linux app. For now it only opens an empty window, so the build image and toolchain can be
//! checked before the real window exists.

use gtk::prelude::*;
use nasib_linux::APPLICATION_ID;

fn main() -> gtk::glib::ExitCode {
    let application = gtk::Application::builder()
        .application_id(APPLICATION_ID)
        .build();
    application.connect_activate(|application| {
        gtk::ApplicationWindow::builder()
            .application(application)
            .title("Nasib")
            .build()
            .present();
    });
    application.run()
}
