mod app_group;
mod grid_item;
mod window;

use gtk::gdk::Display;
use gtk::prelude::*;
use gtk4 as gtk;

use window::Window;

fn main() {
    let application = gtk::Application::new(
        Some("com.github.gtk-rs.examples.apps_launcher"),
        Default::default(),
    );

    application.connect_activate(|app| {
        let provider = gtk::CssProvider::new();
        provider.load_from_data(include_bytes!("style.css"));
        gtk::StyleContext::add_provider_for_display(
            &Display::default().expect("Error initializing gtk css provider."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        new_build_ui(app);
    });

    application.run();
}

fn new_build_ui(app: &gtk::Application) {
    // Create a new custom window and show it
    let window = Window::new(app);
    window.show();
}
