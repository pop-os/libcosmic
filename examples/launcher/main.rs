use gdk4::Display;
use gio::DesktopAppInfo;
use gtk4::gio;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4::CssProvider;
use gtk4::StyleContext;
use once_cell::sync::OnceCell;
use pop_launcher_service::IpcClient;
use postage::mpsc::Sender;
use postage::prelude::*;
use x11rb::rust_connection::RustConnection;

use crate::utils::BoxedSearchResult;

use self::search_result_object::SearchResultObject;
use self::window::Window;

mod search_result_object;
mod search_result_row;
mod utils;
mod window;

const NUM_LAUNCHER_ITEMS: u8 = 10;
static TX: OnceCell<Sender<Event>> = OnceCell::new();
static X11_CONN: OnceCell<RustConnection> = OnceCell::new();

pub enum Event {
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
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn main() {
    let app = gtk4::Application::builder()
        .application_id("com.cosmic.Launcher")
        .build();

    app.connect_startup(|app| {
        setup_shortcuts(app);
        load_css()
    });
    app.connect_activate(move |app| {
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
                            let model = window.model();
                            let model_len = model.n_items();
                            dbg!(&results);
                            let new_results: Vec<glib::Object> = results
                                // [0..std::cmp::min(results.len(), NUM_LAUNCHER_ITEMS.into())]
                                .into_iter()
                                .map(|result| SearchResultObject::new(&BoxedSearchResult(Some(result))).upcast())
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
                                .launch(&[], Some(&window.display().app_launch_context())).expect("failed to launch the application.");
                        }
                    }
                }
            }
        })
    });

    app.run();
}
