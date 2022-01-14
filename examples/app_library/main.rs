use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::CssProvider;
use gtk4::StyleContext;

use window::AppLibraryWindow;

mod app_grid;
mod app_group;
mod grid_item;
mod group_grid;
mod utils;
mod window;

fn main() {
    let app = gtk4::Application::new(Some("com.cosmic.app_library"), Default::default());
    app.connect_startup(|_app| {
        load_css();
    });

    app.connect_activate(|app| {
        build_ui(app);
    });

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
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn build_ui(app: &gtk4::Application) {
    // Create a new custom window and show it
    let window = AppLibraryWindow::new(app);

    window.show();
}
