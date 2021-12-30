use gtk::gdk::Display;
use gtk::prelude::*;
use gtk4 as gtk;
use gtk4::CssProvider;
use gtk4::StyleContext;
use once_cell::sync::OnceCell;
use x11rb::rust_connection::RustConnection;

use window::Window;

mod app_group;
mod grid_item;
mod utils;
mod window;

static X11_CONN: OnceCell<RustConnection> = OnceCell::new();

fn main() {
    let app = gtk::Application::new(Some("com.cosmic.app_library"), Default::default());
    app.connect_startup(|app| {
        load_css();
        build_ui(&app);
    });

    // app.connect_activate(|app| {
    //     build_ui(app);
    // });

    app.run();
}

fn load_css() {
    // Load the css file and add it to the provider
    let provider = CssProvider::new();
    provider.load_from_data(include_bytes!("style.css"));

    // Add the provider to the default screen
    StyleContext::add_provider_for_display(
        &Display::default().expect("Error initializing GTK CSS provider."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &gtk::Application) {
    // Create a new custom window and show it
    let window = Window::new(app);
    let (conn, _screen_num) = x11rb::connect(None).expect("Failed to connect to X");
    if X11_CONN.set(conn).is_err() {
        println!("failed to set X11_CONN. Exiting");
        std::process::exit(1);
    };
    window.show();
}
