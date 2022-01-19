use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::Duration;

use crate::dock_list::DockListType;
use crate::utils::{block_on, BoxedWindowList};
use gdk4::Display;
use gio::DesktopAppInfo;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4::CssProvider;
use gtk4::StyleContext;
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use zbus::Connection;
use zvariant_derive::Type;

use self::dock_object::DockObject;
use self::window::Window;

mod dock_item;
mod dock_list;
mod dock_object;
mod dock_popover;
mod plugin;
mod utils;
mod window;

const ID: &str = "com.cosmic.dock";
const DEST: &str = "com.System76.PopShell";
const PATH: &str = "/com/System76/PopShell";

static TX: OnceCell<mpsc::Sender<Event>> = OnceCell::new();
static PLUGINS: Lazy<Mutex<HashMap<String, libloading::Library>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

pub enum Event {
    WindowList(Vec<Item>),
    Activate((u32, u32)),
    Close((u32, u32)),
    Favorite((String, bool)),
    RefreshFromCache,
}

#[derive(Debug, Deserialize, Serialize, Type, Clone, PartialEq, Eq)]
pub struct Item {
    entity: (u32, u32),
    name: String,
    description: String,
    desktop_entry: String,
}

fn spawn_zbus(tx: mpsc::Sender<Event>) -> Connection {
    let connection = block_on(Connection::session()).unwrap();

    let sender = tx.clone();
    let conn = connection.clone();
    let _ = std::thread::spawn(move || {
        let cached_results: Vec<Item> = vec![];
        block_on(async move {
            futures::pin_mut!(cached_results);
            loop {
                let m = conn
                    .call_method(Some(DEST), PATH, Some(DEST), "WindowList", &())
                    .await;
                if let Ok(m) = m {
                    if let Ok(mut reply) = m.body::<Vec<Item>>() {
                        let mut cached_results = cached_results.as_mut();
                        reply.sort_by(|a, b| a.name.cmp(&b.name));

                        if cached_results.len() != reply.len()
                            || !reply.iter().zip(cached_results.iter()).fold(
                                0,
                                |acc, z: (&Item, &Item)| {
                                    let (a, b) = z;
                                    if a.name == b.name {
                                        acc + 1
                                    } else {
                                        acc
                                    }
                                },
                            ) == cached_results.len()
                        {
                            cached_results.splice(.., reply.clone());
                            let _ = sender.send(Event::WindowList(reply)).await;
                        }
                    }
                    glib::timeout_future(Duration::from_millis(100)).await;
                }
            }
        })
    });

    connection
}

fn _setup_shortcuts(_app: &Application) {}

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
    assert!(utils::BoxedWindowList::static_type().is_valid());
    assert!(plugin::BoxedDockPlugin::static_type().is_valid());
    let app = gtk4::Application::builder().application_id(ID).build();

    app.connect_startup(|_app| {
        // setup_shortcuts(app);
        load_css()
    });

    app.connect_activate(move |app| {
        let (tx, mut rx) = mpsc::channel(100);

        let zbus_conn = spawn_zbus(tx.clone());
        if TX.set(tx).is_err() {
            eprintln!("failed to set global Sender. Exiting");
            std::process::exit(1);
        };

        let window = Window::new(app);
        window.show();

        let cached_results: Vec<Item> = vec![];
        glib::MainContext::default().spawn_local(async move {
            futures::pin_mut!(cached_results);
            // let rx = RX.get().unwrap().clone();
            while let Some(event) = rx.recv().await {
                match event {
                    Event::Activate(e) => {
                        let _activate_window = zbus_conn
                            .call_method(Some(DEST), PATH, Some(DEST), "WindowFocus", &((e,)))
                            .await
                            .expect("Failed to focus selected window");
                    }
                    Event::Close(e) => {
                        let _activate_window = zbus_conn
                            .call_method(Some(DEST), PATH, Some(DEST), "WindowQuit", &((e,)))
                            .await
                            .expect("Failed to close selected window");
                    }
                    Event::Favorite((name, should_favorite)) => {
                        dbg!(&name);
                        dbg!(should_favorite);
                        let saved_app_model = window.model(DockListType::Saved);
                        let active_app_model = window.model(DockListType::Active);
                        if should_favorite {
                            let mut cur: u32 = 0;
                            let mut index: Option<u32> = None;
                            while let Some(item) = active_app_model.item(cur) {
                                if let Ok(cur_dock_object) = item.downcast::<DockObject>() {
                                    if cur_dock_object.get_path() == Some(name.clone()) {
                                        cur_dock_object.set_saved(true);
                                        index = Some(cur);
                                    }
                                }
                                cur += 1;
                            }
                            if let Some(index) = index {
                                let object = active_app_model.item(index).unwrap();
                                active_app_model.remove(index);
                                saved_app_model.append(&object);
                            }
                        } else {
                            let mut cur: u32 = 0;
                            let mut index: Option<u32> = None;
                            while let Some(item) = saved_app_model.item(cur) {
                                if let Ok(cur_dock_object) = item.downcast::<DockObject>() {
                                    if cur_dock_object.get_path() == Some(name.clone()) {
                                        cur_dock_object.set_saved(false);
                                        index = Some(cur);
                                    }
                                }
                                cur += 1;
                            }
                            if let Some(index) = index {
                                let object = saved_app_model.item(index).unwrap();
                                saved_app_model.remove(index);
                                active_app_model.append(&object);
                            }
                        }
                        let _ = TX.get().unwrap().send(Event::RefreshFromCache).await;
                    }
                    Event::RefreshFromCache => {
                        // println!("refreshing model from cache");
                        let cached_results = cached_results.as_ref();
                        let stack_active = cached_results.iter().fold(
                            BTreeMap::new(),
                            |mut acc: BTreeMap<String, BoxedWindowList>, elem| {
                                if let Some(v) = acc.get_mut(&elem.description) {
                                    v.0.push(elem.clone());
                                } else {
                                    acc.insert(
                                        elem.description.clone(),
                                        BoxedWindowList(vec![elem.clone()]),
                                    );
                                }
                                acc
                            },
                        );
                        let mut stack_active: Vec<BoxedWindowList> =
                            stack_active.into_values().collect();

                        // update active app stacks for saved apps into the saved app model
                        // then put the rest in the active app model (which doesn't include saved apps)
                        let saved_app_model = window.model(DockListType::Saved);

                        let mut saved_i: u32 = 0;
                        while let Some(item) = saved_app_model.item(saved_i) {
                            if let Ok(dock_obj) = item.downcast::<DockObject>() {
                                if let Some(cur_app_info) =
                                    dock_obj.property::<Option<DesktopAppInfo>>("appinfo")
                                {
                                    if let Some((i, _s)) = stack_active
                                        .iter()
                                        .enumerate()
                                        .find(|(_i, s)| s.0[0].description == cur_app_info.name())
                                    {
                                        // println!(
                                        //     "found active saved app {} at {}",
                                        //     _s.0[0].name, i
                                        // );
                                        let active = stack_active.remove(i);
                                        dock_obj.set_property("active", active.to_value());
                                        saved_app_model.items_changed(
                                            saved_i.try_into().unwrap(),
                                            0,
                                            0,
                                        );
                                    } else if let Some(_) = cached_results
                                        .iter()
                                        .find(|s| s.description == cur_app_info.name())
                                    {
                                        dock_obj.set_property(
                                            "active",
                                            BoxedWindowList(Vec::new()).to_value(),
                                        );
                                        saved_app_model.items_changed(
                                            saved_i.try_into().unwrap(),
                                            0,
                                            0,
                                        );
                                    }
                                }
                            }
                            saved_i += 1;
                        }

                        let active_app_model = window.model(DockListType::Active);
                        let model_len = active_app_model.n_items();
                        let new_results: Vec<glib::Object> = stack_active
                            .into_iter()
                            .map(|v| DockObject::from_search_results(v).upcast())
                            .collect();
                        active_app_model.splice(0, model_len, &new_results[..]);
                    }
                    Event::WindowList(results) => {
                        // sort to make comparison with cache easier
                        let mut cached_results = cached_results.as_mut();

                        // build active app stacks for each app
                        let stack_active = results.iter().fold(
                            BTreeMap::new(),
                            |mut acc: BTreeMap<String, BoxedWindowList>, elem| {
                                if let Some(v) = acc.get_mut(&elem.description) {
                                    v.0.push(elem.clone());
                                } else {
                                    acc.insert(
                                        elem.description.clone(),
                                        BoxedWindowList(vec![elem.clone()]),
                                    );
                                }
                                acc
                            },
                        );
                        let mut stack_active: Vec<BoxedWindowList> =
                            stack_active.into_values().collect();

                        // update active app stacks for saved apps into the saved app model
                        // then put the rest in the active app model (which doesn't include saved apps)
                        let saved_app_model = window.model(DockListType::Saved);

                        let mut saved_i: u32 = 0;
                        while let Some(item) = saved_app_model.item(saved_i) {
                            if let Ok(dock_obj) = item.downcast::<DockObject>() {
                                if let Some(cur_app_info) =
                                    dock_obj.property::<Option<DesktopAppInfo>>("appinfo")
                                {
                                    if let Some((i, _s)) = stack_active
                                        .iter()
                                        .enumerate()
                                        .find(|(_i, s)| s.0[0].description == cur_app_info.name())
                                    {
                                        // println!("found active saved app {} at {}", s.0[0].name, i);
                                        let active = stack_active.remove(i);
                                        dock_obj.set_property("active", active.to_value());
                                        saved_app_model.items_changed(
                                            saved_i.try_into().unwrap(),
                                            0,
                                            0,
                                        );
                                    } else if let Some(_) = cached_results
                                        .iter()
                                        .find(|s| s.description == cur_app_info.name())
                                    {
                                        dock_obj.set_property(
                                            "active",
                                            BoxedWindowList(Vec::new()).to_value(),
                                        );
                                        saved_app_model.items_changed(
                                            saved_i.try_into().unwrap(),
                                            0,
                                            0,
                                        );
                                    }
                                }
                            }
                            saved_i += 1;
                        }

                        let active_app_model = window.model(DockListType::Active);
                        let model_len = active_app_model.n_items();
                        let new_results: Vec<glib::Object> = stack_active
                            .into_iter()
                            .map(|v| DockObject::from_search_results(v).upcast())
                            .collect();
                        active_app_model.splice(0, model_len, &new_results[..]);
                        cached_results.splice(.., results);
                    }
                }
            }
        });
    });

    app.run();
}
