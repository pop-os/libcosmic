mod imp;
// use crate::ApplicationObject;
use crate::TX;
use crate::X11_CONN;
use gdk4::Rectangle;
use gdk4::Surface;
use gdk4_x11::X11Surface;
use gtk4 as gtk;
use gtk4::Allocation;
use gtk4::EventControllerMotion;
use postage::prelude::Sink;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt;

// use crate::application_row::ApplicationRow;
use glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk::{Application, SignalListItemFactory};
use libcosmic::x;
use x11rb::protocol::xproto;

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
        // let model = gio::ListStore::new(ApplicationObject::static_type());

        // let selection_model = gtk::SingleSelection::builder()
        //     .model(&slice_model)
        //     .autoselect(false)
        //     .can_unselect(true)
        //     .selected(gtk4::INVALID_LIST_POSITION)
        //     .build();

        // imp.model.set(model).expect("Could not set model");
        // // Wrap model with selection and pass it to the list view
        // imp.list_view.set_model(Some(&selection_model));
    }

    fn setup_callbacks(&self) {
        // Get state
        let imp = imp::Window::from_instance(self);
        let window = self.clone().upcast::<gtk::Window>();
        // let list_view = &imp.list_view;
        // let lv = list_view.get();

        // let revealer = Revealer::builder()
        //     .child(&window)
        //     .reveal_child(false)
        //     .transition_duration(200)
        //     .transition_type(gtk4::RevealerTransitionType::SlideUp)
        //     .build();

        // let app_selection_model = list_view
        //     .model()
        //     .expect("List view missing selection model")
        //     .downcast::<gtk::SingleSelection>()
        //     .expect("could not downcast listview model to single selection model");

        // app_selection_model.connect_selected_notify(glib::clone!(@weak window => move |model| {
        //     let i = model.selected();
        //     println!("acitvating... {}", i + 1);
        //     let app_info = model.item(i);
        //     if app_info.is_none() {
        //         println!("oops no app for this row...");
        //         return;
        //     }
        //     if let Ok(id) = app_info.unwrap().property("id") {
        //         let id = id.get::<u32>().expect("App ID must be u32");

        //         glib::MainContext::default().spawn_local(async move {
        //             if let Some(tx) = TX.get() {
        //                 let mut tx = tx.clone();
        //                 let _ = tx.send(crate::Event::Activate(id)).await;
        //             }
        //         });
        //     }
        // }));

        let event_controller = &imp.event_controller.get().unwrap();
        let revealer = &imp.revealer.get();
        window.connect_show(
            glib::clone!(@weak revealer, @weak event_controller => move |_| {
                dbg!(!event_controller.contains_pointer());
                if !event_controller.contains_pointer() {
                    revealer.set_reveal_child(false);
                }
            }),
        );
        window.connect_realize(move |window| {
            if let Some((display, surface)) = x::get_window_x11(window) {
                unsafe {
                    x::change_property(
                        &display,
                        &surface,
                        "_NET_WM_WINDOW_TYPE",
                        x::PropMode::Replace,
                        &[x::Atom::new(&display, "_NET_WM_WINDOW_TYPE_DOCK").unwrap()],
                    );
                }
                let s = window.surface().expect("Failed to get Surface for Window");
                let surface_resize_handler = glib::clone!(@weak window => move |s: &Surface| {
                    if let Some((display, _surface)) = x::get_window_x11(&window) {
                        let width = s.width() * s.scale_factor();
                        let height = s.height() * s.scale_factor();
                        let monitor = display
                            .primary_monitor()
                            .expect("Failed to get Monitor");
                        let Rectangle {
                            x: monitor_x,
                            y: monitor_y,
                            width: monitor_width,
                            height: monitor_height,
                        } = monitor.geometry();
                        dbg!(monitor_width);
                        dbg!(monitor_height);
                        dbg!(width);
                        dbg!(height);
                        let w_conf = xproto::ConfigureWindowAux::default()
                            .x(monitor_x + monitor_width / 2 - width / 2)
                            .y(monitor_y + monitor_height - height);
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

                   } else {
                        println!("failed to get X11 window");
                    }
                });
                s.connect_height_notify(surface_resize_handler.clone());
                s.connect_width_notify(surface_resize_handler.clone());
                s.connect_scale_factor_notify(surface_resize_handler);
                // s.connect_enter_monitor(glib::clone!(@weak rv => move |s, monitor| {
                //     monitor.connect_connector_notify
                // }));
            } else {
                println!("failed to get X11 window");
            }
        });
        event_controller.connect_enter(glib::clone!(@weak revealer => move |_evc, _x, _y| {
            dbg!("hello, mouse entered me :)");
            revealer.set_reveal_child(true);
        }));
        event_controller.connect_leave(glib::clone!(@weak revealer => move |_evc| {
            dbg!("hello, mouse left me :)");
            revealer.set_reveal_child(false);
        }));
    }

    fn setup_event_controller(&self) {
        let imp = imp::Window::from_instance(self);
        let window = &imp.revealer.get();
        let ev = EventControllerMotion::builder()
            .propagation_limit(gtk4::PropagationLimit::None)
            .propagation_phase(gtk4::PropagationPhase::Capture)
            .build();
        window.add_controller(&ev);
        imp.event_controller
            .set(ev)
            .expect("Could not set event controller");
    }

    fn setup_factory(&self) {
        let factory = SignalListItemFactory::new();
        // factory.connect_setup(move |_, list_item| {
        //     let row = ApplicationRow::new():q ;
        //     list_item.set_child(Some(&row))
        // });
        // factory.connect_bind(move |_, list_item| {
        //     let application_object = list_item
        //         .item()
        //         .expect("The item has to exist.")
        //         .downcast::<ApplicationObject>()
        //         .expect("The item has to be an `ApplicationObject`");
        //     let row = list_item
        //         .child()
        //         .expect("The list item child needs to exist.")
        //         .downcast::<ApplicationRow>()
        //         .expect("The list item type needs to be `ApplicationRow`");
        //     if list_item.position() < 9 {
        //         row.set_shortcut(list_item.position() + 1);
        //     }

        //     row.set_app_info(application_object);
        // });
        // Set the factory of the list view
        let imp = imp::Window::from_instance(self);
        // imp.list_view.set_factory(Some(&factory));
    }
}
