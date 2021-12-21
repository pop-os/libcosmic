#![feature(iter_zip)]
mod dock_item;
mod dock_object;
mod utils;
mod window;

use crate::utils::BoxedSearchResults;
use async_io::Timer;
use futures::StreamExt;
use gdk4::Display;
use gio::DesktopAppInfo;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use gtk::Application;
use gtk4 as gtk;
use gtk4::CssProvider;
use gtk4::StyleContext;
use once_cell::sync::OnceCell;
use pop_launcher::SearchResult;
use pop_launcher_service::IpcClient;
use postage::mpsc::Sender;
use postage::prelude::*;
use std::collections::BTreeMap;
use std::time::Duration;
use x11rb::rust_connection::RustConnection;

use self::dock_object::DockObject;
use self::window::Window;

const NUM_LAUNCHER_ITEMS: u8 = 10;
static TX: OnceCell<Sender<Event>> = OnceCell::new();
static X11_CONN: OnceCell<RustConnection> = OnceCell::new();

pub enum Event {
    Response(pop_launcher::Response),
    Search(String),
    Activate(u32),
}

fn spawn_launcher(tx: Sender<Event>) -> IpcClient {
    let (launcher, responses) =
        pop_launcher_service::IpcClient::new().expect("failed to connect to launcher service");

    let mut sender = tx.clone();
    glib::MainContext::default().spawn_local(async move {
        futures::pin_mut!(responses);
        while let Some(event) = responses.next().await {
            let _ = sender.send(Event::Response(event)).await;
        }
    });

    // TODO listen for signal indicating change from dock service...
    let mut sender = tx.clone();
    glib::MainContext::default().spawn_local(async move {
        loop {
            let _ = sender.send(Event::Search(String::new())).await;
            Timer::after(Duration::from_secs(1)).await;
        }
    });

    launcher
}

fn setup_shortcuts(app: &Application) {
    //quit shortcut
    app.set_accels_for_action("win.quit", &["<primary>W", "Escape"]);
    //launch shortcuts
    for i in 1..NUM_LAUNCHER_ITEMS {
        app.set_accels_for_action(&format!("win.launch{}", i), &[&format!("<primary>{}", i)]);
    }
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

fn main() {
    assert!(utils::BoxedSearchResults::static_type().is_valid());
    let app = gtk::Application::builder()
        .application_id("com.cosmic.Launcher")
        .build();

    app.connect_startup(|app| {
        setup_shortcuts(app);
        load_css()
    });

    // TODO investigate multiple signals to connect_activate
    // crashes when called twice bc of singleton
    app.connect_activate(move |app| {
        // Seems that over a long period of time, this might be called multiple times
        // The global variables should be initialized outside this closure
        let (tx, mut rx) = postage::mpsc::channel(1);
        let mut launcher = spawn_launcher(tx.clone());
        if TX.set(tx).is_err() {
            println!("failed to set global Sender. Exiting");
            std::process::exit(1);
        };

        let (conn, _screen_num) = x11rb::connect(None).expect("Failed to connect to X");
        if X11_CONN.set(conn).is_err() {
            println!("failed to set X11_CONN. Exiting");
            std::process::exit(1);
        };
        let window = Window::new(app);
        window.show();

        let cached_results: Vec<SearchResult> = vec![];
        glib::MainContext::default().spawn_local(async move {
            futures::pin_mut!(cached_results);
            while let Some(event) = rx.recv().await {
                match event {
                    Event::Search(search) => {
                        let _ = launcher.send(pop_launcher::Request::Search(search)).await;
                    }
                    Event::Activate(index) => {
                        let _ = launcher.send(pop_launcher::Request::Activate(index)).await;
                    }
                    Event::Response(event) => {
                        if let pop_launcher::Response::Update(mut results) = event {
                            // sort to make comparison with cache easier
                            let mut cached_results = cached_results.as_mut();
                            results.sort_by(|a, b| a.name.cmp(&b.name));
                            // check if cache equals the new polled results
                            // skip if equal
                            if cached_results.len() == results.len() && results.iter().zip(cached_results.iter()).fold(0, |acc, z: (&SearchResult, &SearchResult)| {
                                let (a, b) = z;
                                if a.name == b.name {acc + 1} else {acc}
                            }) == cached_results.len() {
                                continue // skip this update
                            }

                            println!("updating active apps");
                            let model = window.active_app_model();
                            let model_len = model.n_items();
                            let stack_active = results.iter().fold(BTreeMap::new(), |mut acc: BTreeMap<String, BoxedSearchResults>, elem| {
                                if let Some(v) = acc.get_mut(&elem.description) {
                                    v.0.push(elem.clone());
                                } else {
                                    acc.insert(elem.description.clone(), BoxedSearchResults(vec![elem.clone()]));
                                }
                                acc
                            });
                            let new_results: Vec<glib::Object> = stack_active
                                .into_values()
                                .map(|v| DockObject::from_search_results(v).upcast())
                                .collect();
                            model.splice(0, model_len, &new_results[..]);
                            cached_results.splice(.., results);
                        }
                        else if let pop_launcher::Response::DesktopEntry {
                            path,
                            gpu_preference: _gpu_preference, // TODO use GPU preference when launching app
                        } = event
                        {
                            let app_info =
                                DesktopAppInfo::new(&path.file_name().expect("desktop entry path needs to be a valid filename").to_string_lossy())
                                    .expect("failed to create a Desktop App info for launching the application.");
                            app_info
                                .launch(&[], Some(&window.display().app_launch_context())).expect("failed to launch the application.");
                        }
                    }
                }
            }
        })
    });

    app.run();
}
