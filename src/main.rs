mod database;
mod filesystem;
mod ui;

use database::Database;
use gtk;
use gtk::glib;
use gtk::prelude::*;
use gtk::Application;
use simple_logger::SimpleLogger;
use tokio;
use ui::window::build_ui;

const APP_ID: &str = "org.gtk_rs.deduplicator";

#[tokio::main]
async fn main() -> glib::ExitCode {
    SimpleLogger::new().init().unwrap();

    initialize_db().await;

    let app = Application::new(Some(APP_ID), Default::default());
    app.connect_activate(build_ui);

    app.run()
}

async fn initialize_db() {
    let database = Database::connect_default().await;
    database.migrate().await;
}
