mod imp;
use crate::ApplicationObject;
use crate::TX;
use gtk4 as gtk;
use postage::prelude::Sink;

use crate::application_row::ApplicationRow;
use glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk::{Application, SignalListItemFactory};

use libcosmic::x;

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
        let model = gio::ListStore::new(ApplicationObject::static_type());

        let slice_model = gtk::SliceListModel::new(Some(&model), 0, NUM_LAUNCHER_ITEMS.into());
        let selection_model = gtk::SingleSelection::new(Some(&slice_model));

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
        list_view.connect_activate(move |list_view, i| {
            println!("acitvating... {}", i + 1);
            let model = list_view.model().unwrap();
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
        });

        entry.connect_changed(move |search: &gtk::Entry| {
            let search = search.text().to_string();

            glib::MainContext::default().spawn_local(async move {
                if let Some(tx) = TX.get() {
                    let mut tx = tx.clone();
                    let _ = tx.send(crate::Event::Search(search)).await;
                }
            });
        });

        entry.connect_realize(move |search: &gtk::Entry| {
            let search = search.text().to_string();

            glib::MainContext::default().spawn_local(async move {
                if let Some(tx) = TX.get() {
                    let mut tx = tx.clone();
                    let _ = tx.send(crate::Event::Search(search)).await;
                }
            });
        });

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
        // Set the factory of the list view
        let imp = imp::Window::from_instance(self);
        imp.list_view.set_factory(Some(&factory));
    }
}
