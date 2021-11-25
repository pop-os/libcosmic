use gtk4 as gtk;

use gtk::prelude::*;
use libcosmic::x;
use pop_launcher_service::IpcClient;
use postage::mpsc::Sender;
use postage::prelude::*;

fn icon_source(icon: &gtk::Image, source: &Option<pop_launcher::IconSource>) {
    match source {
        Some(pop_launcher::IconSource::Name(name)) => {
            icon.set_from_icon_name(Some(name));
        }
        Some(pop_launcher::IconSource::Mime(content_type)) => {
            icon.set_from_gicon(&gio::content_type_get_icon(content_type));
        }
        _ => {
            icon.set_from_icon_name(None);
        }
    }
}

enum Event {
    Response(pop_launcher::Response),
    Search(String),
}

fn spawn_launcher(mut tx: Sender<Event>) -> IpcClient {
    let (launcher, responses) =
        pop_launcher_service::IpcClient::new().expect("failed to connect to launcher service");

    glib::MainContext::default().spawn_local(async move {
        use futures::StreamExt;
        futures::pin_mut!(responses);
        while let Some(event) = responses.next().await {
            let _ = tx.send(Event::Response(event)).await;
        }
    });

    launcher
}

fn main() {
    let app = gtk::Application::builder()
        .application_id("com.system76.Launcher")
        .build();

    app.connect_activate(move |app| {
        let (tx, mut rx) = postage::mpsc::channel(1);
        let mut launcher = spawn_launcher(tx.clone());

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .decorated(false)
            .default_width(480)
            .default_height(440)
            .title("Launcher")
            .build();

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 16);
        vbox.set_margin_start(16);
        vbox.set_margin_end(16);
        vbox.set_margin_top(16);
        vbox.set_margin_bottom(16);
        window.set_child(Some(&vbox));

        let search = gtk::SearchEntry::new();
        search.set_placeholder_text(Some(" Type to search apps, or type '?' for more options."));
        vbox.append(&search);

        let listbox = gtk::ListBox::new();
        vbox.append(&listbox);

        {
            let search_changed = glib::clone!(@strong tx => move |search: &gtk::SearchEntry| {
                let search = search.text().to_string();

                let mut tx = tx.clone();
                glib::MainContext::default().spawn_local(async move {
                    let _ = tx.send(Event::Search(search)).await;
                });
            });

            search_changed(&search);
            search.connect_changed(search_changed);
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

        glib::MainContext::default().spawn_local(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    Event::Search(search) => {
                        //TODO: is this the best way to clear a listbox?
                        while let Some(child) = listbox.last_child() {
                            listbox.remove(&child);
                        }

                        let _ = launcher.send(pop_launcher::Request::Search(search)).await;
                    }

                    Event::Response(event) => {
                        if let pop_launcher::Response::Update(results) = event {
                            for (i, result) in results.iter().enumerate() {
                                // Limit to 9 results
                                if i >= 9 {
                                    continue;
                                }

                                let row = gtk::ListBoxRow::new();
                                listbox.append(&row);

                                // Select first row
                                if i == 0 {
                                    listbox.select_row(Some(&row));
                                }

                                let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                                hbox.set_margin_start(8);
                                hbox.set_margin_end(8);
                                hbox.set_margin_top(8);
                                hbox.set_margin_bottom(8);
                                row.set_child(Some(&hbox));

                                let category_icon = gtk::Image::new();
                                category_icon.set_pixel_size(16);
                                icon_source(&category_icon, &result.category_icon);
                                hbox.append(&category_icon);

                                let icon = gtk::Image::new();
                                icon.set_pixel_size(32);
                                icon_source(&icon, &result.icon);
                                hbox.append(&icon);

                                let labels = gtk::Box::new(gtk::Orientation::Vertical, 4);
                                hbox.append(&labels);

                                let name = gtk::Label::new(Some(&result.name));
                                name.set_halign(gtk::Align::Start);
                                labels.append(&name);

                                let description = gtk::Label::new(Some(&result.description));
                                description.set_halign(gtk::Align::Start);
                                labels.append(&description);
                            }
                        }
                    }
                }
            }
        });
    });

    app.run();
}
