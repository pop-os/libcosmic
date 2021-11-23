mod application_object;
use gtk4 as gtk;

use gtk::gio;
use gtk::prelude::*;
use gtk::{
    glib, Application, ApplicationWindow, Label, ListView, PolicyType, ScrolledWindow,
    SignalListItemFactory, SingleSelection,
};
use libcosmic::x;
use std::{cell::RefCell, rc::Rc};

use self::application_object::ApplicationObject;
use self::ipc::LauncherIpc;
mod ipc;

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

fn main() {
    let launcher = Rc::new(RefCell::new(
        LauncherIpc::new().expect("failed to connect to launcher service"),
    ));

    let app = gtk::Application::builder()
        .application_id("com.system76.Launcher")
        .build();

    app.connect_activate(move |app| {
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

        let search = gtk::Entry::new();
        search.set_placeholder_text(Some(" Type to search apps, or type '?' for more options."));
        vbox.append(&search);

        let listbox = gtk::ListBox::new();
        //vbox.append(&listbox);

        let model = gio::ListStore::new(ApplicationObject::static_type());
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let label = Label::new(None);
            list_item.set_child(Some(&label))
        });
        factory.connect_bind(move |_, list_item| {
            let application_object = list_item
                .item()
                .expect("The item has to exist.")
                .downcast::<ApplicationObject>()
                .expect("The item has to be an `ApplicationObject`");
            let name = application_object
                .property("name")
                .expect("Property name of the wrong type or does not exist!")
                .get::<String>()
                .expect("Property name needs to be a String.");
            let label = list_item
                .child()
                .expect("The child has to exist.")
                .downcast::<Label>()
                .expect("The child has to be a `label`");
            label.set_label(&name);
        });
        let selection_model = SingleSelection::new(Some(&model));
        let list_view = ListView::new(Some(&selection_model), Some(&factory));
        vbox.append(&list_view);

        {
            let launcher = launcher.clone();
            let search_changed = move |search: &gtk::Entry| {
                let model_len = model.n_items();
                //TODO: is this the best way to clear a listbox?
                //while let Some(child) = listbox.last_child() {
                //    listbox.remove(&child);
                //}

                let response_res = launcher
                    .borrow_mut()
                    .request(pop_launcher::Request::Search(search.text().to_string()));

                println!("{:#?}", response_res);

                let num_items = 9;

                if let Ok(pop_launcher::Response::Update(results)) = response_res {
                    let new_results: Vec<glib::Object> = results
                        [0..std::cmp::min(results.len(), num_items)]
                        .iter()
                        .map(|result| ApplicationObject::new(result).upcast())
                        .collect();
                    model.splice(0, model_len, &new_results[..]);
                    // for (i, result) in results[0..std::cmp::min(results.len(), num_items)]
                    //     .iter()
                    //     .enumerate()
                    // {
                    //     dbg!(result);
                    //     // Limit to 9 results
                    //     // if i >= 9 {
                    //     //     break 'result_loop;
                    //     // }

                    //     let row = gtk::ListBoxRow::new();
                    //     listbox.append(&row);

                    //     // Select first row
                    //     if i == 0 {
                    //         listbox.select_row(Some(&row));
                    //     }

                    //     let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                    //     hbox.set_margin_start(8);
                    //     hbox.set_margin_end(8);
                    //     hbox.set_margin_top(8);
                    //     hbox.set_margin_bottom(8);
                    //     row.set_child(Some(&hbox));

                    //     let category_icon = gtk::Image::new();
                    //     category_icon.set_pixel_size(16);
                    //     icon_source(&category_icon, &result.category_icon);
                    //     hbox.append(&category_icon);

                    //     let icon = gtk::Image::new();
                    //     icon.set_pixel_size(32);
                    //     icon_source(&icon, &result.icon);
                    //     hbox.append(&icon);

                    //     let labels = gtk::Box::new(gtk::Orientation::Vertical, 4);
                    //     hbox.append(&labels);

                    //     let name = gtk::Label::new(Some(&result.name));
                    //     name.set_halign(gtk::Align::Start);
                    //     labels.append(&name);

                    //     let description = gtk::Label::new(Some(&result.description));
                    //     description.set_halign(gtk::Align::Start);
                    //     labels.append(&description);
                    // }
                }
            };

            search.connect_changed(search_changed.clone());

            search_changed(&search);
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
