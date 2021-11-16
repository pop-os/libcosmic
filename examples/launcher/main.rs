use gtk4 as gtk;

use gtk::prelude::*;
use libcosmic::x;
use std::{
    cell::RefCell,
    rc::Rc,
};

use self::ipc::LauncherIpc;
mod ipc;

fn main() {
    let launcher = Rc::new(RefCell::new(
        LauncherIpc::new().expect("failed to connect to launcher service")
    ));

    let app = gtk::Application::builder()
        .application_id("com.system76.Launcher")
        .build();

    app.connect_activate(move |app| {
        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .default_width(480)
            .default_height(440)
            .title("Launcher")
            .build();

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
        window.set_child(Some(&vbox));

        let search = gtk::Entry::new();
        vbox.append(&search);

        {
            let launcher = launcher.clone();
            search.connect_changed(move |search| {
                let response = launcher.borrow_mut().request(
                    pop_launcher::Request::Search(search.text().to_string())
                ).expect("failed to access launcher service");

                println!("{:?}", response);
            });
        }

        // Setting the window to dialog type must happen between realize and show. Dialog windows
        // show up centered on the display with the cursor, so we do not have to set position
        window.connect_realize(move |window| {
            if let Some((display, surface)) = x::get_window_x11(window) {
                unsafe {
                    x::change_property(
                        &display,
                        &surface,
                        "_NET_WM_WINDOW_TYPE",
                        x::PropMode::Replace,
                        &[x::Atom::new(&display, "_NET_WM_WINDOW_TYPE_DIALOG").unwrap()],
                    );
                }
            } else {
                println!("failed to get X11 window");
            }
        });

        window.show();
    });

    app.run();
}
