use gtk4 as gtk;
mod application_row;
use application_row::ApplicationRow;
use gtk::gdk::Display;
use gtk::prelude::*;
use gtk::{gio, glib};
use libcosmic::x;
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
        build_ui(app);
    });

    application.run();
}

fn build_ui(app: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .decorated(false)
        .default_width(600)
        .default_height(600)
        .application(app)
        .title("ListView: Applications Launcher")
        .build();

    let model = gio::ListStore::new(gio::AppInfo::static_type());
    gio::AppInfo::all().iter().for_each(|app_info| {
        model.append(app_info);
    });
    let window_model = gtk::Window::list_toplevels();
    dbg!(window_model.clone());
    for i in window_model {
        dbg!(i);
    }
    // TODO window ui file and custom class
    // TODO application search entry ui file and custom class
    // TODO list open windows in application search entries
    // TODO list aliases in application search entries
    let factory = gtk::SignalListItemFactory::new();
    // the "setup" stage is used for creating the widgets
    factory.connect_setup(move |_factory, item| {
        let row = ApplicationRow::new();
        item.set_child(Some(&row));
    });

    // the bind stage is used for "binding" the data to the created widgets on the "setup" stage
    factory.connect_bind(move |_factory, list_item| {
        let app_info = list_item
            .item()
            .unwrap()
            .downcast::<gio::AppInfo>()
            .unwrap();
        println!("position: {}", &list_item.position());
        println!("{}", app_info.name());
        // println!("{}", app_info.description());

        let child = list_item
            .child()
            .unwrap()
            .downcast::<ApplicationRow>()
            .unwrap();
        child.set_app_info(&app_info);
        if list_item.position() < 9 {
            child.set_shortcut(list_item.position() + 1);
        }
    });

    // A sorter used to sort AppInfo in the model by their name
    let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
        let app_info1 = obj1.downcast_ref::<gio::AppInfo>().unwrap();
        let app_info2 = obj2.downcast_ref::<gio::AppInfo>().unwrap();

        app_info1
            .name()
            .to_lowercase()
            .cmp(&app_info2.name().to_lowercase())
            .into()
    });
    let filter = gtk::CustomFilter::new(|_obj| true);
    let filter_model = gtk::FilterListModel::new(Some(&model), Some(filter).as_ref());
    let sorted_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));
    let slice_model = gtk::SliceListModel::new(Some(&sorted_model), 0, 9);
    let selection_model = gtk::SingleSelection::new(Some(&slice_model));

    let list_view = gtk::ListView::new(Some(&selection_model), Some(&factory));
    let action_launch = gio::SimpleAction::new("launch", Some(&i32::static_variant_type()));
    action_launch.connect_activate(glib::clone!(@weak list_view => move |_action, parameter| {
        // Get parameter
        let parameter = parameter
            .expect("Could not get parameter.")
            .get::<i32>()
            .expect("The variant needs to be of type `i32`.");
        println!("{}", parameter);
        let model = list_view.model().unwrap();
        let app_info = model
            .item(u32::try_from(parameter).unwrap());

        if app_info.is_none() {return}
        let app_info = app_info
            .unwrap()
            .downcast::<gio::AppInfo>()
            .unwrap();

        let context = list_view.display().app_launch_context();
        if let Err(err) = app_info.launch(&[], Some(&context)) {
            let parent_window = list_view.root().unwrap().downcast::<gtk::Window>().unwrap();

            gtk::MessageDialog::builder()
                .text(&format!("Failed to start {}", app_info.name()))
                .secondary_text(&err.to_string())
                .message_type(gtk::MessageType::Error)
                .modal(true)
                .transient_for(&parent_window)
                .build()
                .show();
        }

    }));
    window.add_action(&action_launch);
    for i in 1..10 {
        let action_launchi = gio::SimpleAction::new(&format!("launch{}", i), None);
        app.set_accels_for_action(&format!("win.launch{}", i), &[&format!("<primary>{}", i)]);
        window.add_action(&action_launchi);
        action_launchi.connect_activate(
            glib::clone!(@weak action_launch, @weak list_view => move |_action, _parameter| {
                let model = list_view.model().unwrap();
                let app_info = model
                    .item(i-1);

                println!("launching item {}", i);
                if app_info.is_none() {return}
                let app_info = app_info
                    .unwrap()
                    .downcast::<gio::AppInfo>()
                    .unwrap();
                println!("starting {}", app_info.name());
                let context = list_view.display().app_launch_context();
                if let Err(err) = app_info.launch(&[], Some(&context)) {
                    let parent_window = list_view.root().unwrap().downcast::<gtk::Window>().unwrap();

                    gtk::MessageDialog::builder()
                        .text(&format!("Failed to start {}", app_info.name()))
                        .secondary_text(&err.to_string())
                        .message_type(gtk::MessageType::Error)
                        .modal(true)
                        .transient_for(&parent_window)
                        .build()
                        .show();
                }
            }),
        );
    }

    // Launch the application when an item of the list is activated
    list_view.connect_activate(move |list_view, position| {
        let model = list_view.model().unwrap();
        let app_info = model
            .item(position)
            .unwrap()
            .downcast::<gio::AppInfo>()
            .unwrap();

        let context = list_view.display().app_launch_context();
        if let Err(err) = app_info.launch(&[], Some(&context)) {
            let parent_window = list_view.root().unwrap().downcast::<gtk::Window>().unwrap();

            gtk::MessageDialog::builder()
                .text(&format!("Failed to start {}", app_info.name()))
                .secondary_text(&err.to_string())
                .message_type(gtk::MessageType::Error)
                .modal(true)
                .transient_for(&parent_window)
                .build()
                .show();
        }
    });

    let scrolled_window = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never) // Disable horizontal scrolling
        .min_content_width(360)
        .vexpand(true)
        .child(&list_view)
        .build();

    let launcher_box = gtk::Box::builder()
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        //.valign(gtk::Align::Center)
        //.halign(gtk::Align::Center)
        //.spacing(12)
        .orientation(gtk::Orientation::Vertical)
        .build();
    let search_input = gtk::Entry::builder()
        //.margin_top(12)
        .margin_bottom(12)
        //.margin_start(12)
        //.margin_end(12)
        //.valign(gtk::Align::Center)
        //.halign(gtk::Align::Center)
        .build();
    // Filter model whenever the value of the key "filter" changes
    search_input.connect_changed(
        glib::clone!(@weak filter_model, @weak sorted_model => move |search: &gtk::Entry| {
            let search_text = search.text().to_string().to_lowercase();
            let new_filter: gtk::CustomFilter = gtk::CustomFilter::new(move |obj| {
                let search_res = obj.downcast_ref::<gio::AppInfo>()
                    .expect("The Object needs to be of type AppInfo");
                search_res.name().to_string().to_lowercase().contains(&search_text)
            });
            let search_text = search.text().to_string().to_lowercase();
            let new_sorter: gtk::CustomSorter = gtk::CustomSorter::new(move |obj1, obj2| {
                let app_info1 = obj1.downcast_ref::<gio::AppInfo>().unwrap();
                let app_info2 = obj2.downcast_ref::<gio::AppInfo>().unwrap();
                if search_text == "" {
                    return app_info1
                        .name()
                        .to_lowercase()
                        .cmp(&app_info2.name().to_lowercase())
                        .into();
                }

                let i_1 = app_info1.name().to_lowercase().find(&search_text);
                let i_2 = app_info2.name().to_lowercase().find(&search_text);
                match (i_1, i_2) {
                    (Some(i_1), Some(i_2)) => i_1.cmp(&i_2).into(),
                    (Some(_), None) => std::cmp::Ordering::Less.into(),
                    (None, Some(_)) => std::cmp::Ordering::Greater.into(),
                    _ => app_info1
                        .name()
                        .to_lowercase()
                        .cmp(&app_info2.name().to_lowercase())
                        .into()
                }
            });

            filter_model.set_filter(Some(new_filter).as_ref());
            sorted_model.set_sorter(Some(new_sorter).as_ref());
        }),
    );
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

    launcher_box.append(&search_input);
    launcher_box.append(&scrolled_window);
    window.set_child(Some(&launcher_box));
    // Add action "quit" to `window` taking no parameter
    app.set_accels_for_action("win.quit", &["<primary>W", "Escape"]);
    let action_quit = gio::SimpleAction::new("quit", None);
    action_quit.connect_activate(glib::clone!(@weak window => move |_, _| {
        window.close();
    }));
    window.add_action(&action_quit);

    window.connect_is_active_notify(|win| {
        if !win.is_active() {
            win.close();
        }
        println!("active or not lets find out...");
    });
    window.show();
}
