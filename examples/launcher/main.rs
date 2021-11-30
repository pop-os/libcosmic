mod application_object;
mod application_row;
use gio::DesktopAppInfo;
use gtk4 as gtk;
use gtk4::SliceListModel;

use cascade::cascade;
use gtk::gio;
use gtk::prelude::*;
use gtk::{glib, ListView, SignalListItemFactory, SingleSelection};
use libcosmic::x;
use pop_launcher_service::IpcClient;
use postage::mpsc::Sender;
use postage::prelude::*;

use self::application_object::ApplicationObject;
use self::application_row::ApplicationRow;

const NUM_LAUNCHER_ITEMS: u8 = 10;

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
    Activate(u32),
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

        //quit shortcut
        app.set_accels_for_action("win.quit", &["<primary>W", "Escape"]);
        //launch shortcuts
        for i in 1..10 {
            app.set_accels_for_action(&format!("win.launch{}", i), &[&format!("<primary>{}", i)]);
        }

        let window = gtk::ApplicationWindow::builder()
            .application(app)
            .decorated(false)
            .default_width(600)
            .default_height(440)
            .title("Launcher")
            .resizable(false)
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

        let model = gio::ListStore::new(ApplicationObject::static_type());
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let row = ApplicationRow::new();
            list_item.set_child(Some(&row))
        });
        factory.connect_bind(move |_, list_item| {
            let application_object = list_item
                .item()
                .expect("The item has to exist.")
                .downcast::<ApplicationObject>()
                .expect("The item has to be an `ApplicationObject`");
            let row = list_item
                .child()
                .expect("The list item child needs to exist.")
                .downcast::<ApplicationRow>()
                .expect("The list item type needs to be `ApplicationRow`");
            if list_item.position() < 9 {
                row.set_shortcut(list_item.position() + 1);
            }

            row.set_app_info(application_object);
        });
        let slice_model = SliceListModel::new(Some(&model), 0, NUM_LAUNCHER_ITEMS.into());
        let selection_model = SingleSelection::new(Some(&slice_model));
        let list_view = ListView::new(Some(&selection_model), Some(&factory));
        let scroll = cascade! {
            gtk::ScrolledWindow::new();
            ..set_min_content_height(400);
            ..set_max_content_height(700);
            ..set_propagate_natural_height(true);
            ..set_vexpand(true);
            ..set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
        };
        scroll.set_child(Some(&list_view));
        vbox.append(&scroll);

        for i in 1..10 {
            let action_launchi = gio::SimpleAction::new(&format!("launch{}", i), None);
            window.add_action(&action_launchi);
            action_launchi.connect_activate(
                glib::clone!(@weak list_view, @strong tx =>  move |_action, _parameter| {
                     println!("acitvating... {}", i);
                     let model = list_view.model().unwrap();
                     let app_info = model.item(i - 1);
                     if app_info.is_none() {
                         println!("oops no app for this row...");
                         return;
                     }
                    if let Ok(id)= app_info.unwrap().property("id") {
                         let id = id.get::<u32>().expect("App ID must be u32");
                         let mut tx = tx.clone();

                         glib::MainContext::default().spawn_local(async move {
                             let _ = tx.send(Event::Activate(id)).await;
                         });
                     }
                }),
            );
        }

        list_view.connect_activate(glib::clone!(@strong tx =>  move |list_view, i| {
            println!("acitvating... {}", i + 1);
            let model = list_view.model().unwrap();
            let app_info = model.item(i);
            if app_info.is_none() {
                println!("oops no app for this row...");
                return;
            }
            if let Ok(id)= app_info.unwrap().property("id") {
                let id = id.get::<u32>().expect("App ID must be u32");
                let mut tx = tx.clone();

                glib::MainContext::default().spawn_local(async move {
                    let _ = tx.send(Event::Activate(id)).await;
                });
            }
        }));

        {
            let search_changed = glib::clone!(@strong tx => move |search: &gtk::Entry| {
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
        let action_quit = gio::SimpleAction::new("quit", None);
        action_quit.connect_activate(glib::clone!(@weak window => move |_, _| {
            window.close();
        }));
        window.add_action(&action_quit);

        window.connect_is_active_notify(|win| {
            if !win.is_active() {
                win.close();
            }
        });

        window.show();
        glib::MainContext::default().spawn_local(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    Event::Search(search) => {
                        let _ = launcher.send(pop_launcher::Request::Search(search)).await;
                    }
                    Event::Activate(index) => {
                        let _ = launcher.send(pop_launcher::Request::Activate(index)).await;
                    }

                    Event::Response(event) => {
                        if let pop_launcher::Response::Update(results) = event {
                            let model_len = model.n_items();
                            dbg!(&results);
                            let new_results: Vec<glib::Object> = results
                                [0..std::cmp::min(results.len(), NUM_LAUNCHER_ITEMS.into())]
                                .iter()
                                .map(|result| ApplicationObject::new(result).upcast())
                                .collect();
                            model.splice(0, model_len, &new_results[..]);
                        } else if let pop_launcher::Response::DesktopEntry {
                            path,
                            gpu_preference: _gpu_preference, // TODO use GPU preference when launching app
                        } = event
                        {
                            let app_info =
                                DesktopAppInfo::new(&path.file_name().expect("desktop entry path needs to be a valid filename").to_string_lossy())
                                    .expect("failed to create a Desktop App info for launching the application.");
                            app_info
                                .launch(&[], Some(&window.display().app_launch_context().clone())).expect("failed to launch the application.");
                        }
                    }
                }
            }
        })
    });

    app.run();
}
