use futures_util::stream::StreamExt;

use zbus::{dbus_proxy, Connection, Result};
use zvariant::ObjectPath;

#[dbus_proxy(
    default_service = "org.freedesktop.GeoClue2",
    interface = "org.freedesktop.GeoClue2.Manager",
    default_path = "/org/freedesktop/GeoClue2/Manager"
)]
trait Manager {
    #[dbus_proxy(object = "Client")]
    fn get_client(&self);
}

#[dbus_proxy(
    default_service = "org.freedesktop.GeoClue2",
    interface = "org.freedesktop.GeoClue2.Client"
)]
trait Client {
    fn start(&self) -> Result<()>;
    fn stop(&self) -> Result<()>;

    #[dbus_proxy(property)]
    fn set_desktop_id(&mut self, id: &str) -> Result<()>;

    #[dbus_proxy(signal)]
    fn location_updated(&self, old: ObjectPath<'_>, new: ObjectPath<'_>) -> Result<()>;
}

#[dbus_proxy(
    default_service = "org.freedesktop.GeoClue2",
    interface = "org.freedesktop.GeoClue2.Location"
)]
trait Location {
    #[dbus_proxy(property)]
    fn latitude(&self) -> Result<f64>;
    #[dbus_proxy(property)]
    fn longitude(&self) -> Result<f64>;
}

mod dock_item;
mod dock_object;
mod utils;
mod window;

use crate::utils::BoxedSearchResults;
use async_io::Timer;
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
    Loc(f64, f64),
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
        // loop {
        let _ = sender.send(Event::Search(String::new())).await;
        Timer::after(Duration::from_secs(1)).await;
        // }
    });

    let mut sender = tx.clone();
    glib::MainContext::default().spawn_local(async move {
        let conn = Connection::system().await.unwrap();
        let manager = ManagerProxy::new(&conn).await.unwrap();
        let mut client = manager.get_client().await.unwrap();

        client.set_desktop_id("org.freedesktop.zbus").await.unwrap();
        let mut location_updated_stream = client.receive_location_updated().await.unwrap();

        client.start().await.unwrap();
        while let Some(signal) = location_updated_stream.next().await {
            let args = signal.args().unwrap();

            let location = LocationProxy::builder(&conn)
                .path(args.new())
                .unwrap()
                .build()
                .await
                .unwrap();
            let lat = location.latitude().await.unwrap();
            let long = location.longitude().await.unwrap();
            println!("Latitude: {}\nLongitude: {}", lat, long,);
            let _ = sender.send(Event::Loc(lat, long)).await;
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

        glib::MainContext::default().spawn_local(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    Event::Loc(lat, long) => {
                        dbg!(lat);
                        dbg!(long);
                    }
                    Event::Search(search) => {
                        let _ = launcher.send(pop_launcher::Request::Search(search)).await;
                    }
                    Event::Activate(index) => {
                        let _ = launcher.send(pop_launcher::Request::Activate(index)).await;
                    }
                    Event::Response(event) => {
                        if let pop_launcher::Response::Update(results) = event {
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
