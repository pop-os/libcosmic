mod imp;
// use crate::ApplicationObject;
use crate::dock_item::DockItem;
use crate::dock_object::DockObject;
use crate::utils::data_path;
use crate::BoxedWindowList;
use crate::Event;
use crate::TX;
use crate::X11_CONN;
use gdk4::ContentProvider;
use gdk4::Display;
use gdk4::Rectangle;
use gdk4_x11::X11Display;
use gdk4_x11::X11Surface;
use gio::DesktopAppInfo;
use gio::Icon;
use glib::Type;
use gtk4 as gtk;
use gtk4::prelude::ListModelExt;
use gtk4::DragSource;
use gtk4::DropTarget;
use gtk4::EventControllerMotion;
use gtk4::IconTheme;
use postage::prelude::Sink;
use std::fs::File;
use std::path::Path;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::ConnectionExt as OtherConnectionExt;
use x11rb::protocol::xproto::*;
use x11rb::wrapper::ConnectionExt;
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

impl Window {
    pub fn new(app: &Application) -> Self {
        let self_: Self = Object::new(&[("application", app)]).expect("Failed to create `Window`.");
        self_
    }

    pub fn saved_app_model(&self) -> &gio::ListStore {
        // Get state
        let imp = imp::Window::from_instance(self);
        imp.saved_app_model
            .get()
            .expect("Could not get saved_app_model")
    }

    pub fn active_app_model(&self) -> &gio::ListStore {
        // Get state
        let imp = imp::Window::from_instance(self);
        imp.active_app_model
            .get()
            .expect("Could not get active_app_model")
    }

    fn setup_model(&self) {
        // Get state and set model

        let imp = imp::Window::from_instance(self);
        let saved_app_model = gio::ListStore::new(DockObject::static_type());

        let saved_selection_model = gtk::SingleSelection::builder()
            .autoselect(false)
            .can_unselect(true)
            .selected(gtk4::INVALID_LIST_POSITION)
            .model(&saved_app_model)
            .build();

        imp.saved_app_model
            .set(saved_app_model)
            .expect("Could not set model");
        // Wrap model with selection and pass it to the list view
        imp.saved_app_list_view
            .set_model(Some(&saved_selection_model));

        let active_app_model = gio::ListStore::new(DockObject::static_type());
        let active_selection_model = gtk::SingleSelection::builder()
            .autoselect(false)
            .can_unselect(true)
            .selected(gtk4::INVALID_LIST_POSITION)
            .model(&active_app_model)
            .build();

        imp.active_app_model
            .set(active_app_model)
            .expect("Could not set model");
        // Wrap model with selection and pass it to the list view
        imp.active_app_list_view
            .set_model(Some(&active_selection_model));
    }

    fn setup_callbacks(&self) {
        // Get state
        let imp = imp::Window::from_instance(self);
        let window = self.clone().upcast::<gtk::Window>();
        let saved_app_list_view = &imp.saved_app_list_view;
        let saved_app_model = &imp
            .saved_app_model
            .get()
            .expect("Failed to get saved app model");

        let saved_app_selection_model = saved_app_list_view
            .model()
            .expect("List view missing selection model")
            .downcast::<gtk::SingleSelection>()
            .expect("could not downcast listview model to single selection model");
        let active_app_selection_model = imp
            .active_app_list_view
            .model()
            .expect("List view missing selection model")
            .downcast::<gtk::SingleSelection>()
            .expect("could not downcast listview model to single selection model");

        let selected_handler = glib::clone!(@weak window => move |model: &gtk::SingleSelection| {
            let position = model.selected();
            println!("selected app {}", position);
            // Launch the application when an item of the list is activated
            if let Some(item) = model.item(position) {
                let dockobject = item.downcast::<DockObject>().expect("App model must only contain DockObject");
                if let Ok(active) = dockobject.property("active").expect("DockObject must have active property").get::<BoxedWindowList>() {
                    if let Some(focused_item) = active.0.iter().next() {
                        let entity = focused_item.entity.clone();
                        glib::MainContext::default().spawn_local(async move {
                            if let Some(tx) = TX.get() {
                                let mut tx = tx.clone();
                                let _ = tx.send(Event::Activate(entity)).await;
                            }
                        });
                    }
                    else if let Ok(Some(app_info)) = dockobject.property("appinfo").expect("DockObject must have appinfo property").get::<Option<DesktopAppInfo>>() {
                        let context = window.display().app_launch_context();
                        if let Err(err) = app_info.launch(&[], Some(&context)) {
                            gtk::MessageDialog::builder()
                                .text(&format!("Failed to start {}", app_info.name()))
                                .secondary_text(&err.to_string())
                                .message_type(gtk::MessageType::Error)
                                .modal(true)
                                .transient_for(&window)
                                .build()
                                .show();
                        }
                    }
                }
            }
            model.set_selected(gtk4::INVALID_LIST_POSITION);
        });
        saved_app_selection_model.connect_selected_notify(selected_handler.clone());

        active_app_selection_model.connect_selected_notify(selected_handler);

        let cursor_event_controller = &imp.cursor_event_controller.get().unwrap();
        let drop_controller = &imp.drop_controller.get().unwrap();
        let window_drop_controller = &imp.window_drop_controller.get().unwrap();
        let revealer = &imp.revealer.get();
        window.connect_show(
            glib::clone!(@weak revealer, @weak cursor_event_controller => move |_| {
                // dbg!(!cursor_event_controller.contains_pointer());
                if !cursor_event_controller.contains_pointer() {
                    revealer.set_reveal_child(false);
                }
            }),
        );
        window.connect_realize(glib::clone!(@weak revealer, @weak window_drop_controller, @weak cursor_event_controller => move |window| {
            if let Some((display, surface)) = x::get_window_x11(window) {
                // ignore all x11 errors...
                let xdisplay = display.clone().downcast::<X11Display>().expect("Failed to downgrade X11 Display.");
                xdisplay.error_trap_push();
                let conn = X11_CONN.get().expect("Failed to get X11 connection");
                let window_type_atom = conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE").unwrap().reply().unwrap().atom;
                let dock_type_atom = conn.intern_atom(false, b"_NET_WM_WINDOW_TYPE_DOCK").unwrap().reply().unwrap().atom;
                conn.change_property32(
                    PropMode::REPLACE,
                    surface.xid().try_into().unwrap(),
                    window_type_atom,
                    AtomEnum::ATOM,
                    &[dock_type_atom]
                ).unwrap();
                let resize = glib::clone!(@weak window, @weak revealer => move || {
                    let s = window.surface().expect("Failed to get Surface for Window");
                    let height = if revealer.reveals_child() { window.height() } else { 4 };
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
                            .y((monitor_y + monitor_height - height).clamp(0, monitor_y + monitor_height - 1));
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

                let resize_drop = resize.clone();
                window_drop_controller.connect_enter(glib::clone!(@weak revealer, @weak window => @default-return gdk4::DragAction::COPY, move |_self, _x, _y| {
                    glib::source::idle_add_local_once(resize_drop.clone());
                    revealer.set_reveal_child(true);
                    gdk4::DragAction::COPY
                }));

                let resize_cursor = resize.clone();
                cursor_event_controller.connect_enter(glib::clone!(@weak revealer, @weak window => move |_evc, _x, _y| {
                    // dbg!("hello, mouse entered me :)");
                    revealer.set_reveal_child(true);
                    glib::source::idle_add_local_once(resize_cursor.clone());
                }));

                let resize_revealed = resize.clone();
                revealer.connect_child_revealed_notify(glib::clone!(@weak window => move |r| {
                    if !r.is_child_revealed() {
                        glib::source::idle_add_local_once(resize_revealed.clone());
                    }
                    }));

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
        }));

        cursor_event_controller.connect_leave(
            glib::clone!(@weak revealer, @weak drop_controller => move |_evc| {
                // only hide if DnD is not happening
                if drop_controller.current_drop().is_none() {
                    // dbg!("hello, mouse left me :)");
                    revealer.set_reveal_child(false);
                }
            }),
        );

        drop_controller.connect_enter(glib::clone!(@weak revealer => @default-return gdk4::DragAction::COPY, move |_self, _x, _y| {

            revealer.set_reveal_child(true);
            gdk4::DragAction::COPY
        }));
        window_drop_controller.connect_drop(|_, _, _, _| {
            println!("dropping into window");
            false
        });
        let saved_app_list_view = saved_app_list_view.get();

        // drag end handler
        // must be modified in case of reorder...
        let drag_end = &imp.drag_end_signal;
        let saved_drag_source = &imp.saved_drag_source;
        drop_controller.connect_drop(
            glib::clone!(@weak saved_app_model, @weak saved_app_list_view, @weak drag_end, @weak saved_drag_source => @default-return true, move |_self, drop_value, x, _y| {
                if let Ok(Some(path_str)) = drop_value.get::<Option<String>>() {
                    let desktop_path = &Path::new(&path_str);
                    if let Some(pathbase) = desktop_path.file_name() {
                        if let Some(app_info) = gio::DesktopAppInfo::new(&pathbase.to_string_lossy()) {
                            // remove item if already exists
                            let mut i: u32 = 0;
                            let mut index_of_existing_app: Option<u32> = None;
                            while let Some(item) = saved_app_model.item(i) {
                                if let Ok(cur_app_info) = item.downcast::<DockObject>() {
                                    if let Ok(Some(cur_app_info)) = cur_app_info.property("appinfo").expect("property appinfo missing from DockObject").get::<Option<DesktopAppInfo>>() {
                                        // dbg!(cur_app_info.filename());
                                        if cur_app_info.filename() == Some(Path::new(&path_str).to_path_buf()) {
                                            index_of_existing_app = Some(i);
                                        }
                                    }
                                }
                                i += 1;
                            }
                            // dbg!(app_info.name());
                            // dbg!(index_of_existing_app);
                            if let Some(index_of_existing_app) = index_of_existing_app {
                                // remove existing entry
                                saved_app_model.remove(index_of_existing_app);
                                if let Some(old_handle) = drag_end.replace(None) {
                                    glib::signal_handler_disconnect(saved_drag_source.get().expect("Failed to get drag handler"), old_handle);
                                }
                            }

                            //calculate insertion location
                            // dbg!(x);
                            // dbg!(y);
                            let max_x = saved_app_list_view.allocated_width();
                            // dbg!(max_x);
                            // dbg!(max_y);
                            let n_buckets = saved_app_model.n_items() * 2;

                            let drop_bucket = (x * n_buckets as f64 / (max_x as f64 + 0.1)) as u32;
                            let index = if drop_bucket == 0 {
                                0
                            } else if drop_bucket == n_buckets - 1 {
                                saved_app_model.n_items()
                            } else {
                                (drop_bucket + 1) / 2
                            };
                            // dbg!(index);
                            // dbg!("dropped it!");
                            // dbg!(drop_value.type_());
                            saved_app_model.insert(index, &DockObject::new(app_info));
                        }
                    }
                }
                else {
                    // dbg!("rejecting drop");
                    _self.reject();
                }
                true
            }),
        );

        saved_app_model.connect_items_changed(|saved_app_model, _, _removed, _added| {
            Self::store_saved_apps(saved_app_model);
        });
    }

    fn setup_event_controller(&self) {
        let imp = imp::Window::from_instance(self);
        let handle = &imp.cursor_handle.get();
        let ev = EventControllerMotion::builder()
            .propagation_limit(gtk4::PropagationLimit::None)
            .propagation_phase(gtk4::PropagationPhase::Capture)
            .build();
        handle.add_controller(&ev);

        imp.cursor_event_controller
            .set(ev)
            .expect("Could not set event controller");
    }

    fn setup_drop_target(&self) {
        let imp = imp::Window::from_instance(self);
        let drop_target_widget = &imp.saved_app_list_view;
        let mut drop_actions = gdk4::DragAction::COPY;
        drop_actions.insert(gdk4::DragAction::MOVE);
        let drop_format = gdk4::ContentFormats::for_type(Type::STRING);
        // causes error for some reason...
        drop_format.union(&gdk4::ContentFormats::for_type(DockObject::static_type()));
        let drop_target_controller = DropTarget::builder()
            .preload(true)
            .actions(drop_actions)
            .formats(&drop_format)
            .build();
        drop_target_widget.add_controller(&drop_target_controller);
        imp.drop_controller
            .set(drop_target_controller)
            .expect("Could not set dock dnd drop controller");

        // hack for revealing hidden dock when drag enters dock window
        let window_drop_target_controller = DropTarget::builder()
            .actions(drop_actions)
            .formats(&gdk4::ContentFormats::for_type(Type::STRING))
            .build();
        let enter_handle = &imp.cursor_handle.get();

        enter_handle.add_controller(&window_drop_target_controller);
        imp.window_drop_controller
            .set(window_drop_target_controller)
            .expect("Could not set dock dnd drop controller");
    }

    fn setup_drag_source(&self) {
        let imp = imp::Window::from_instance(self);
        let saved_app_list_view = &imp.saved_app_list_view.get();
        let saved_app_model = imp
            .saved_app_model
            .get()
            .expect("Failed to get saved app model.");

        let actions = gdk4::DragAction::MOVE;
        let saved_drag_source = DragSource::builder()
            .name("dock drag source")
            .actions(actions)
            .build();

        let drag_end = &imp.drag_end_signal;
        let drag_cancel = &imp.drag_end_signal;
        saved_app_list_view.add_controller(&saved_drag_source);
        saved_drag_source.connect_prepare(glib::clone!(@weak saved_app_model, @weak saved_app_list_view, @weak drag_end, @weak drag_cancel => @default-return None, move |self_, x, _y| {
            // set drag source icon if possible...
            // gio Icon is not easily converted to a Paintable, but this seems to be the correct method
            let max_x = saved_app_list_view.allocated_width();
            // dbg!(max_x);
            // dbg!(max_y);
            let n_buckets = saved_app_model.n_items();

            let index = (x * n_buckets as f64 / (max_x as f64 + 0.1)) as u32;
            if let Some(item) = saved_app_model.item(index) {
                if let Some(old_handle) = drag_end.replace(Some(self_.connect_drag_end(
                    glib::clone!(@weak saved_app_model => move |_self, _drag, _delete_data| {
                        dbg!(_delete_data);
                        if _delete_data {saved_app_model.remove(index)};
                    }),
                ))) {
                    glib::signal_handler_disconnect(self_, old_handle);
                }
                if let Some(old_handle) = drag_cancel.replace(Some(self_.connect_drag_cancel(
                    glib::clone!(@weak saved_app_model => @default-return false, move |_self, _drag, cancel_reason| {
                        if cancel_reason != gdk4::DragCancelReason::UserCancelled {
                            saved_app_model.remove(index);
                            true
                        } else  {
                            false
                        }
                    }),
                ))) {
                    glib::signal_handler_disconnect(self_, old_handle);
                }
                if let Ok(dock_object) = item.downcast::<DockObject>() {
                    if let Ok(Some(app_info)) = dock_object.property("appinfo").expect("property appinfo missing from DockObject").get::<Option<DesktopAppInfo>>() {
                        let icon = app_info
                            .icon()
                            .unwrap_or(Icon::for_string("image-missing").expect("Failed to set default icon"));

                        if let Some(default_display) = &Display::default() {
                            if let Some(icon_theme) = IconTheme::for_display(default_display) {
                                if let Some(paintable_icon) = icon_theme.lookup_by_gicon(
                                    &icon,
                                    64,
                                    1,
                                    gtk4::TextDirection::None,
                                    gtk4::IconLookupFlags::empty(),
                                ) {
                                    self_.set_icon(Some(&paintable_icon), 32, 32);
                                }
                            }
                        }
                        if let Some(file) = app_info.filename() {
                            return Some(ContentProvider::for_value(&file.to_string_lossy().to_value()));
                        }
                    }
                }
            }
            None
        }));
        imp.saved_drag_source
            .set(saved_drag_source)
            .expect("Could not set saved drag source");

        let active_app_list_view = &imp.active_app_list_view.get();
        let active_app_model = imp
            .active_app_model
            .get()
            .expect("Failed to get saved app model.");

        let actions = gdk4::DragAction::MOVE;
        let active_drag_source = DragSource::builder()
            .name("dock drag source")
            .actions(actions)
            .build();

        active_drag_source.connect_drag_begin(|_self, drag| {
            drag.set_selected_action(gdk4::DragAction::MOVE);
        });

        active_app_list_view.add_controller(&active_drag_source);
        active_drag_source.connect_prepare(glib::clone!(@weak active_app_model, @weak active_app_list_view, @weak drag_end, @weak drag_cancel => @default-return None, move |self_, x, _y| {
            let max_x = active_app_list_view.allocated_width();
            // dbg!(max_x);
            // dbg!(max_y);
            let n_buckets = active_app_model.n_items();
            let index = (x * n_buckets as f64 / (max_x as f64 + 0.1)) as u32;
            if let Some(item) = active_app_model.item(index) {
                if let Some(old_handle) = drag_end.replace(Some(self_.connect_drag_end(
                    glib::clone!(@weak active_app_model => move |_self, _drag, _delete_data| {
                        dbg!(_delete_data);
                        if _delete_data {active_app_model.remove(index)};
                    }),
                ))) {
                    glib::signal_handler_disconnect(self_, old_handle);
                }
                if let Some(old_handle) = drag_cancel.replace(Some(self_.connect_drag_cancel(
                    glib::clone!(@weak active_app_model => @default-return false, move |_self, _drag, cancel_reason| {
                        if cancel_reason != gdk4::DragCancelReason::UserCancelled {
                            active_app_model.remove(index);
                            true
                        } else  {
                            false
                        }
                    }),
                ))) {
                    glib::signal_handler_disconnect(self_, old_handle);
                }


                if let Ok(dock_object) = item.downcast::<DockObject>() {
                    if let Ok(Some(app_info)) = dock_object.property("appinfo").expect("property appinfo missing from DockObject").get::<Option<DesktopAppInfo>>() {
                        let icon = app_info
                            .icon()
                            .unwrap_or(Icon::for_string("image-missing").expect("Failed to set default icon"));

                        if let Some(default_display) = &Display::default() {
                            if let Some(icon_theme) = IconTheme::for_display(default_display) {
                                if let Some(paintable_icon) = icon_theme.lookup_by_gicon(
                                    &icon,
                                    64,
                                    1,
                                    gtk4::TextDirection::None,
                                    gtk4::IconLookupFlags::empty(),
                                ) {
                                    self_.set_icon(Some(&paintable_icon), 32, 32);
                                }
                            }
                        }
                        if let Some(file) = app_info.filename() {
                            return Some(ContentProvider::for_value(&file.to_string_lossy().to_value()));
                        }
                    }
                }
            }
            None
        }));
        imp.active_drag_source
            .set(active_drag_source)
            .expect("Could not set saved drag source");
    }

    fn setup_factory(&self) {
        let saved_app_factory = SignalListItemFactory::new();
        saved_app_factory.connect_setup(move |_, list_item| {
            let dock_item = DockItem::new();
            list_item.set_child(Some(&dock_item));
        });
        let imp = imp::Window::from_instance(self);
        let saved_app_model = imp
            .saved_app_model
            .get()
            .expect("Failed to get saved app model.");
        saved_app_factory.connect_bind(glib::clone!(@weak saved_app_model => move |_, list_item| {
            let application_object = list_item
                .item()
                .expect("The item has to exist.")
                .downcast::<DockObject>()
                .expect("The item has to be a `DockObject`");
            let dock_item = list_item
                .child()
                .expect("The list item child needs to exist.")
                .downcast::<DockItem>()
                .expect("The list item type needs to be `DockItem`");

            dock_item.set_app_info(&application_object);
        }));
        // Set the factory of the list view
        imp.saved_app_list_view
            .set_factory(Some(&saved_app_factory));

        let active_app_model = imp
            .active_app_model
            .get()
            .expect("Failed to get saved app model.");
        let active_factory = SignalListItemFactory::new();
        active_factory.connect_setup(move |_, list_item| {
            let dock_item = DockItem::new();
            list_item.set_child(Some(&dock_item));
        });
        active_factory.connect_bind(glib::clone!(@weak active_app_model => move |_, list_item| {
            let application_object = list_item
                .item()
                .expect("The item has to exist.")
                .downcast::<DockObject>()
                .expect("The item has to be a `DockObject`");
            let dock_item = list_item
                .child()
                .expect("The list item child needs to exist.")
                .downcast::<DockItem>()
                .expect("The list item type needs to be `DockItem`");

            dock_item.set_app_info(&application_object);
        }));
        // Set the factory of the list view
        imp.active_app_list_view.set_factory(Some(&active_factory));
    }

    fn restore_saved_apps(&self) {
        if let Ok(file) = File::open(data_path()) {
            if let Ok(saved_data) = serde_json::from_reader::<_, Vec<String>>(file) {
                // dbg!(&saved_data);
                let dock_objects: Vec<Object> = saved_data
                    .into_iter()
                    .filter_map(|d| {
                        DockObject::from_app_info_path(&d)
                            .map(|dockobject| dockobject.upcast::<Object>())
                    })
                    .collect();
                // dbg!(&dock_objects);
                let saved_app_model = self.saved_app_model();
                saved_app_model.splice(saved_app_model.n_items(), 0, &dock_objects);
                return;
            }
        }
        println!("Error loading saved apps!");
        let saved_app_model = &self.saved_app_model();
        xdg::BaseDirectories::new()
            .expect("could not access XDG Base directory")
            .get_data_dirs()
            .iter_mut()
            .for_each(|xdg_data_path| {
                let defaults = ["Firefox Web Browser", "Files", "Terminal", "Pop!_Shop"];
                xdg_data_path.push("applications");
                // dbg!(&xdg_data_path);
                if let Ok(dir_iter) = std::fs::read_dir(xdg_data_path) {
                    dir_iter.for_each(|dir_entry| {
                        if let Ok(dir_entry) = dir_entry {
                            if let Some(path) = dir_entry.path().file_name() {
                                if let Some(path) = path.to_str() {
                                    if let Some(app_info) = gio::DesktopAppInfo::new(path) {
                                        if app_info.should_show()
                                            && defaults.contains(&app_info.name().as_str())
                                        {
                                            saved_app_model.append(&DockObject::new(app_info));
                                        } else {
                                            // println!("Ignoring {}", path);
                                        }
                                    } else {
                                        // println!("error loading {}", path);
                                    }
                                }
                            }
                        }
                    })
                }
            });
    }

    fn store_saved_apps(saved_app_model: &gio::ListStore) {
        // Store todo data in vector
        let mut backup_data = Vec::new();
        let mut i = 0;
        while let Some(item) = saved_app_model.item(i) {
            // Get `AppGroup` from `glib::Object`
            let dock_object = item
                .downcast_ref::<DockObject>()
                .expect("The object needs to be of type `AppGroupData`.");
            // Add todo data to vector and increase position
            if let Ok(Some(app_info)) = dock_object
                .property("appinfo")
                .expect("DockObject must have appinfo property")
                .get::<Option<DesktopAppInfo>>()
            {
                if let Some(f) = app_info.filename() {
                    backup_data.push(f);
                }
            }
            i += 1;
        }
        // dbg!(&backup_data);
        // Save state in file
        let file = File::create(data_path()).expect("Could not create json file.");
        serde_json::to_writer_pretty(file, &backup_data)
            .expect("Could not write data to json file");
    }
}
