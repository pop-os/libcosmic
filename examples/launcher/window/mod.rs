use gdk4::Rectangle;
use gdk4_x11::X11Display;
use gdk4_x11::X11Surface;
use glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk::{Application, SignalListItemFactory};
use gtk4 as gtk;
use postage::prelude::Sink;
use x11rb::connection::Connection;
use x11rb::protocol::xproto;
use x11rb::protocol::xproto::ConnectionExt;

use libcosmic::x;

use crate::search_result_row::SearchResultRow;
use crate::SearchResultObject;
use crate::TX;
use crate::X11_CONN;

mod imp;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

const NUM_LAUNCHER_ITEMS: u8 = 9;

impl Window {
    pub fn new(app: &Application) -> Self {
        let self_: Self = Object::new(&[("application", app)]).expect("Failed to create `Window`.");
        self_
    }

    pub fn model(&self) -> &gio::ListStore {
        // Get state
        let imp = imp::Window::from_instance(self);
        imp.model.get().expect("Could not get model")
    }

    fn setup_model(&self) {
        // Get state and set model
        let imp = imp::Window::from_instance(self);
        let model = gio::ListStore::new(SearchResultObject::static_type());

        let slice_model = gtk::SliceListModel::new(Some(&model), 0, NUM_LAUNCHER_ITEMS.into());
        let selection_model = gtk::SingleSelection::builder()
            .model(&slice_model)
            .autoselect(false)
            .can_unselect(true)
            .selected(gtk4::INVALID_LIST_POSITION)
            .build();

        imp.model.set(model).expect("Could not set model");
        // Wrap model with selection and pass it to the list view
        imp.list_view.set_model(Some(&selection_model));
    }

    fn setup_callbacks(&self) {
        // Get state
        let imp = imp::Window::from_instance(self);
        let window = self.clone().upcast::<gtk::Window>();
        let list_view = &imp.list_view;
        let entry = &imp.entry;
        let lv = list_view.get();
        for i in 1..10 {
            let action_launchi = gio::SimpleAction::new(&format!("launch{}", i), None);
            self.add_action(&action_launchi);
            action_launchi.connect_activate(glib::clone!(@weak lv =>  move |_action, _parameter| {
                println!("acitvating... {}", i);
                let model = lv.model().unwrap();
                let app_info = model.item(i - 1);
                if app_info.is_none() {
                    println!("oops no app for this row...");
                    return;
                }
                if let Ok(id)= app_info.unwrap().property("id") {
                    let id = id.get::<u32>().expect("App ID must be u32");

                    glib::MainContext::default().spawn_local(async move {
                        if let Some(tx) = TX.get() {
                            let mut tx = tx.clone();
                            let _ = tx.send(crate::Event::Activate(id)).await;
                        }
                    });
                }
            }));
        }

        let app_selection_model = list_view
            .model()
            .expect("List view missing selection model")
            .downcast::<gtk::SingleSelection>()
            .expect("could not downcast listview model to single selection model");

        app_selection_model.connect_selected_notify(glib::clone!(@weak window => move |model| {
            let i = model.selected();
            println!("acitvating... {}", i + 1);
            let app_info = model.item(i);
            if app_info.is_none() {
                println!("oops no app for this row...");
                return;
            }
            if let Ok(id) = app_info.unwrap().property("id") {
                let id = id.get::<u32>().expect("App ID must be u32");

                glib::MainContext::default().spawn_local(async move {
                    if let Some(tx) = TX.get() {
                        let mut tx = tx.clone();
                        let _ = tx.send(crate::Event::Activate(id)).await;
                    }
                });
            }
        }));

        entry.connect_changed(glib::clone!(@weak lv => move |search: &gtk::Entry| {
            let search = search.text().to_string();

            glib::MainContext::default().spawn_local(async move {
                if let Some(tx) = TX.get() {
                    let mut tx = tx.clone();
                    let _ = tx.send(crate::Event::Search(search)).await;
                }
            });
        }));

        entry.connect_realize(glib::clone!(@weak lv => move |search: &gtk::Entry| {
            let search = search.text().to_string();

            glib::MainContext::default().spawn_local(async move {
                if let Some(tx) = TX.get() {
                    let mut tx = tx.clone();
                    let _ = tx.send(crate::Event::Search(search)).await;
                }
            });
        }));

        window.connect_realize(move |window| {
            if let Some((display, surface)) = x::get_window_x11(window) {
                // ignore all x11 errors...
                let xdisplay = display.clone().downcast::<X11Display>().expect("Failed to downgrade X11 Display.");
                xdisplay.error_trap_push();
                unsafe {
                    x::change_property(
                        &display,
                        &surface,
                        "_NET_WM_WINDOW_TYPE",
                        x::PropMode::Replace,
                        &[x::Atom::new(&display, "_NET_WM_WINDOW_TYPE_DIALOG").unwrap()],
                    );
                }
                let resize = glib::clone!(@weak window => move || {
                    let s = window.surface().expect("Failed to get Surface for Window");
                    let height = window.height();
                    let width = window.width();

                    if let Some((display, _surface)) = x::get_window_x11(&window) {
                        let monitor = display
                            .primary_monitor()
                            .expect("Failed to get Monitor");
                        let Rectangle {
                            x: monitor_x,
                            y: monitor_y,
                            width: monitor_width,
                            height: monitor_height,
                        } = monitor.geometry();
                        // dbg!(monitor_width);
                        // dbg!(monitor_height);
                        // dbg!(width);
                        // dbg!(height);
                        let w_conf = xproto::ConfigureWindowAux::default()
                            .x((monitor_x + monitor_width / 2 - width / 2).clamp(0, monitor_x + monitor_width - 1))
                            .y((monitor_y + monitor_height / 2 - height / 2).clamp(0, monitor_y + monitor_height - 1));
                        let conn = X11_CONN.get().expect("Failed to get X11_CONN");

                        let x11surface = gdk4_x11::X11Surface::xid(
                            &s.clone().downcast::<X11Surface>()
                                .expect("Failed to downcast Surface to X11Surface"),
                        );
                        conn.configure_window(
                            x11surface.try_into().expect("Failed to convert XID"),
                            &w_conf,
                        )
                            .expect("failed to configure window...");
                        conn.flush().expect("failed to flush");
                    }
                });
                let s = window.surface().expect("Failed to get Surface for Window");
                let resize_height = resize.clone();
                s.connect_height_notify(move |_s| {
                    glib::source::idle_add_local_once(resize_height.clone());
                });
                let resize_width = resize.clone();
                s.connect_width_notify(move |_s| {
                    glib::source::idle_add_local_once(resize_width.clone());
                });
                s.connect_scale_factor_notify(move |_s| {
                    glib::source::idle_add_local_once(resize.clone());
                });
            } else {
                println!("failed to get X11 window");
            }
        });

        let action_quit = gio::SimpleAction::new("quit", None);
        action_quit.connect_activate(glib::clone!(@weak window => move |_, _| {
            window.close();
        }));
        self.add_action(&action_quit);

        window.connect_is_active_notify(|win| {
            if !win.is_active() {
                win.close();
            }
        });
    }

    fn setup_factory(&self) {
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_, list_item| {
            let row = SearchResultRow::new();
            list_item.set_child(Some(&row))
        });
        factory.connect_bind(move |_, list_item| {
            let application_object = list_item
                .item()
                .expect("The item has to exist.")
                .downcast::<SearchResultObject>()
                .expect("The item has to be an `SearchResultObject`");
            let row = list_item
                .child()
                .expect("The list item child needs to exist.")
                .downcast::<SearchResultRow>()
                .expect("The list item type needs to be `SearchResultRow`");
            if list_item.position() < 9 {
                row.set_shortcut(list_item.position() + 1);
            }

            row.set_search_result(application_object);
        });
        // Set the factory of the list view
        let imp = imp::Window::from_instance(self);
        imp.list_view.set_factory(Some(&factory));
    }
}
