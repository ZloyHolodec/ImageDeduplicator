mod ui;

use gtk;
use gtk::glib;
use gtk::prelude::*;
use gtk::Application;
use ui::build_ui;

const APP_ID: &str = "org.gtk_rs.deduplicator";

fn main() -> glib::ExitCode {
    let app = Application::new(Some(APP_ID), Default::default());

    app.connect_activate(build_ui);
    app.run()
}
