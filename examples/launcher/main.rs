use gdk4 as gdk;
use gtk4 as gtk;

use gdk::prelude::*;
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
        let display = gdk::Display::default().unwrap();
        let monitors = display.monitors().unwrap();

        for i in 0..monitors.n_items() {
            let monitor: gdk::Monitor = monitors.item(i).unwrap().downcast().unwrap();
            let rect = monitor.geometry();
            //TODO: get monitor with mouse cursor
            if i == 0 {
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

                window.show();

                if let Some((display, surface)) = x::get_window_x11(&window) {
                    unsafe {
                        x::change_property(
                            &display,
                            &surface,
                            "_NET_WM_WINDOW_TYPE",
                            x::PropMode::Replace,
                            &[x::Atom::new(&display, "_NET_WM_WINDOW_TYPE_DIALOG").unwrap()],
                        );
                        x::set_position(
                            &display,
                            &surface,
                            rect.x + (rect.width - 480) / 2,
                            rect.y + (rect.height - 440) / 2
                        );
                    }
                } else {
                    println!("Failed to get X11 window");
                }
            }
        }
    });

    app.run();
}
