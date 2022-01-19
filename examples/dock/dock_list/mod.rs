use crate::dock_item::DockItem;
use crate::dock_object::DockObject;
use crate::plugin;
use crate::utils::data_path;
use crate::BoxedWindowList;
use crate::Event;
use crate::Item;
use crate::PLUGINS;
use crate::TX;
use cascade::cascade;
use gdk4::ContentProvider;
use gdk4::Display;
use gdk4::ModifierType;
use gio::DesktopAppInfo;
use gio::Icon;
use glib::Object;
use glib::Type;
use gtk4::glib;
use gtk4::prelude::ListModelExt;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::DropTarget;
use gtk4::IconTheme;
use gtk4::ListView;
use gtk4::Orientation;
use gtk4::SignalListItemFactory;
use gtk4::Window;
use gtk4::{DragSource, GestureClick};
use std::ffi::CStr;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::path::Path;

mod imp;

glib::wrapper! {
    pub struct DockList(ObjectSubclass<imp::DockList>)
        @extends gtk4::Widget, gtk4::Box,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DockListType {
    Saved,
    Active,
}

impl Default for DockListType {
    fn default() -> Self {
        DockListType::Active
    }
}

impl DockList {
    pub fn new(type_: DockListType) -> Self {
        let self_: DockList = glib::Object::new(&[]).expect("Failed to create DockList");
        let imp = imp::DockList::from_instance(&self_);
        imp.type_.set(type_).unwrap();
        self_.layout();
        //dnd behavior is different for each type, as well as the data in the model
        self_.setup_model();
        self_.setup_click_controller();
        self_.setup_drag();
        self_.setup_drop_target();
        self_.setup_factory();

        self_
    }

    pub fn model(&self) -> &gio::ListStore {
        // Get state
        let imp = imp::DockList::from_instance(self);
        imp.model.get().expect("Could not get model")
    }

    pub fn drop_controller(&self) -> &DropTarget {
        // Get state
        let imp = imp::DockList::from_instance(self);
        imp.drop_controller.get().expect("Could not get model")
    }

    pub fn popover_index(&self) -> Option<u32> {
        // Get state
        let imp = imp::DockList::from_instance(self);
        imp.popover_menu_index.get()
    }

    fn restore_data(&self) {
        if let Ok(file) = File::open(data_path()) {
            if let Ok(data) = serde_json::from_reader::<_, Vec<String>>(file) {
                // dbg!(&data);
                let dock_objects: Vec<Object> = data
                    .into_iter()
                    .filter_map(|d| {
                        DockObject::from_app_info_path(&d)
                            .map(|dockobject| dockobject.upcast::<Object>())
                    })
                    .collect();
                // dbg!(&dock_objects);

                let model = self.model();
                model.splice(model.n_items(), 0, &dock_objects);
            }
        } else {
            eprintln!("Error loading saved apps!");
            let model = &self.model();
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
                                                model.append(&DockObject::new(app_info));
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
        // TODO load saved plugins here... for now, load the hardcoded example.
        // TODO unload plugin library before the dynamic library is changed, otherwise, it will crash after segfault
        // TODO unload plugin on removal from model
        // TODO dnd for plugin? I think they should either be at the start or end of the dock and not draggable
        // TODO call plugin click handler on click or if it is not provided by the library, open the popover menu instead
        let mut path_dir = glib::user_data_dir();
        path_dir.push(crate::ID);
        std::fs::create_dir_all(&path_dir).expect("Could not create directory.");
        path_dir.push("plugins");
        std::fs::create_dir_all(&path_dir).expect("Could not create directory.");
        let mut path = path_dir.clone();
        path.push("dock_plugin_uwu.so");
        let mut path_css = path_dir.clone();
        path_css.push("dock_plugin_uwu.css");
        let provider = gtk4::CssProvider::new();
        if path.exists() {
            let path = path
                .as_os_str()
                .to_str()
                .expect("plugin path needs to be a valid string");

            if let Ok(f) = File::open(path_css) {
                let mut reader = BufReader::new(f);
                let mut buffer = Vec::new();

                if reader.read_to_end(&mut buffer).is_ok() {
                    provider.load_from_data(&buffer);
                    // Add the provider to the default screen
                    gtk4::StyleContext::add_provider_for_display(
                        &gdk4::Display::default().expect("Error initializing GTK CSS provider."),
                        &provider,
                        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
                    );
                } else {
                    eprintln!("loading plugin css failed");
                }
            } else {
                eprintln!("loading plugin css failed");
            }

            let (popover_menu, image, name, lib) = unsafe {
                let lib = libloading::Library::new(path).unwrap();
                // store library until unloading the plugin
                let image_func: libloading::Symbol<
                    unsafe extern "C" fn() -> *mut gtk4_sys::GtkWidget,
                > = lib.get(b"dock_plugin_image").unwrap();
                let popover_func: libloading::Symbol<
                    unsafe extern "C" fn() -> *mut gtk4_sys::GtkWidget,
                > = lib.get(b"dock_plugin_popover_menu").unwrap();
                let name_func: libloading::Symbol<
                    unsafe extern "C" fn() -> *const std::os::raw::c_char,
                > = lib.get(b"dock_plugin_name").unwrap();
                // click handler is optional

                (popover_func(), image_func(), name_func(), lib)
            };
            if let Ok(ref mut mutex) = PLUGINS.try_lock() {
                mutex.insert(String::from(path), lib);
            }
            let name = if !name.is_null() {
                unsafe { String::from(CStr::from_ptr(name).to_str().unwrap_or_default()) }
            } else {
                String::new()
            };
            let image = if !image.is_null() {
                unsafe {
                    gtk4::glib::translate::from_glib_none::<_, gtk4::Widget>(image).unsafe_cast()
                }
            } else {
                gtk4::Image::new()
            };
            let popover_menu = if !popover_menu.is_null() {
                unsafe {
                    gtk4::glib::translate::from_glib_none::<_, gtk4::Widget>(popover_menu)
                        .unsafe_cast()
                }
            } else {
                gtk4::Box::new(Orientation::Vertical, 4)
            };
            let boxed_plugin = plugin::BoxedDockPlugin {
                path: String::from(path),
                name,
                image,
                popover_menu,
            };
            let model = self.model();
            model.append(&DockObject::from_plugin(boxed_plugin).upcast::<Object>());
        }
    }

    fn store_data(model: &gio::ListStore) {
        // Store todo data in vector
        let mut backup_data = Vec::new();
        let mut i = 0;
        while let Some(item) = model.item(i) {
            // Get `AppGroup` from `glib::Object`
            let dock_object = item
                .downcast_ref::<DockObject>()
                .expect("The object needs to be of type `AppGroupData`.");
            // Add todo data to vector and increase position
            if let Some(app_info) = dock_object.property::<Option<DesktopAppInfo>>("appinfo") {
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
        // TODO save plugins here for now examples are hardcoded and don't need to be saved
    }

    fn layout(&self) {
        let imp = imp::DockList::from_instance(&self);
        let list_view = cascade! {
            ListView::default();
            ..set_orientation(Orientation::Horizontal);
            ..add_css_class("docklist");
        };
        if imp.type_.get().unwrap() == &DockListType::Saved {
            list_view.set_width_request(64);
        }
        self.append(&list_view);
        imp.list_view.set(list_view).unwrap();
    }

    fn setup_model(&self) {
        let imp = imp::DockList::from_instance(self);
        let model = gio::ListStore::new(DockObject::static_type());

        let selection_model = gtk4::NoSelection::new(Some(&model));

        // Wrap model with selection and pass it to the list view
        let list_view = imp.list_view.get().unwrap();
        list_view.set_model(Some(&selection_model));
        imp.model.set(model).expect("Could not set model");

        if imp.type_.get().unwrap() == &DockListType::Saved {
            let model = self.model();
            self.restore_data();
            model.connect_items_changed(|model, _, _removed, _added| {
                Self::store_data(&model);
            });
        }
    }

    fn setup_click_controller(&self) {
        let imp = imp::DockList::from_instance(self);
        let controller = GestureClick::builder()
            .button(0)
            .propagation_limit(gtk4::PropagationLimit::None)
            .propagation_phase(gtk4::PropagationPhase::Capture)
            .build();
        self.add_controller(&controller);

        let model = self.model();
        let list_view = &imp.list_view.get().unwrap();
        let popover_menu_index = &imp.popover_menu_index;
        controller.connect_released(glib::clone!(@weak model, @weak list_view, @weak popover_menu_index => move |self_, _, x, y| {
            let window = list_view.root().unwrap().downcast::<Window>().unwrap();
            let max_x = list_view.allocated_width();
            let max_y = list_view.allocated_height();
            // dbg!(max_y);
            // dbg!(y);
            let n_buckets = model.n_items();
            let index = (x * n_buckets as f64 / (max_x as f64 + 0.1)) as u32;
            // dbg!(self_.current_button());
            // dbg!(self_.last_event(self_.current_sequence().as_ref()));
            let click_modifier = if let Some(event) =  self_.last_event(self_.current_sequence().as_ref()) {
                    // dbg!(&event);
                    Some(event.modifier_state())
                }
                else {
                    None
                };
            // dbg!(click_modifier);
            // Launch the application when an item of the list is activated
            let focus_window = move |first_focused_item: &Item| {
                let entity = first_focused_item.entity.clone();
                glib::MainContext::default().spawn_local(async move {
                    if let Some(tx) = TX.get() {
                        let _ = tx.send(Event::Activate(entity)).await;
                    }
                });
            };
            let old_index = popover_menu_index.get();
            if let Some(old_index) = old_index  {
                if let Some(old_item) = model.item(old_index) {
                    if let Ok(old_dock_object) = old_item.downcast::<DockObject>() {
                        old_dock_object.set_popover(false);
                        popover_menu_index.replace(None);
                        model.items_changed(old_index, 0, 0);
                        //TODO signal dock to check if it should hide
                    }
                }
                return;
            }
            if y > f64::from(max_y) || y < 0.0 || x > f64::from(max_x) || x < 0.0 {
                // println!("out of bounds click...");
                return;
            }

            if let Some(item) = model.item(index) {
                if let Ok(dock_object) = item.downcast::<DockObject>() {
                    let active = dock_object.property::<BoxedWindowList>("active");
                    let app_info = dock_object.property::<Option<DesktopAppInfo>>("appinfo");
                    match (self_.current_button(), click_modifier, active.0.iter().next(), app_info) {
                        (click, Some(click_modifier), Some(first_focused_item), _) if click == 1 && !click_modifier.contains(ModifierType::CONTROL_MASK) => focus_window(first_focused_item),
                        (click, None, Some(first_focused_item), _) if click == 1 => focus_window(first_focused_item),
                        (click, _, _, Some(app_info)) | (click, _, None, Some(app_info)) if click != 3  => {
                            let context = window.display().app_launch_context();
                            if let Err(err) = app_info.launch(&[], Some(&context)) {
                                gtk4::MessageDialog::builder()
                                    .text(&format!("Failed to start {}", app_info.name()))
                                    .secondary_text(&err.to_string())
                                    .message_type(gtk4::MessageType::Error)
                                    .modal(true)
                                    .transient_for(&window)
                                    .build()
                                    .show();
                            }

                        }
                        (click, _, _, _) if click == 3 => {
                            // println!("handling right click");
                            if let Some(old_index) = popover_menu_index.get().clone() {
                                if let Some(item) = model.item(old_index) {
                                    if let Ok(dock_object) = item.downcast::<DockObject>() {
                                        dock_object.set_popover(false);
                                        popover_menu_index.replace(Some(index));
                                        model.items_changed(old_index, 0, 0);
                                    }
                                }
                            }
                            dock_object.set_popover(true);
                            popover_menu_index.replace(Some(index));
                            model.items_changed(index, 0, 0);
                        }
                        _ => eprintln!("Failed to process click.")
                    }
                }
            }
        }));
        imp.click_controller.set(controller).unwrap();
    }

    fn setup_drop_target(&self) {
        let imp = imp::DockList::from_instance(self);
        if imp.type_.get().unwrap() != &DockListType::Saved {
            return;
        }

        let drop_target_widget = &imp.list_view.get().unwrap();
        let mut drop_actions = gdk4::DragAction::COPY;
        drop_actions.insert(gdk4::DragAction::MOVE);
        let drop_format = gdk4::ContentFormats::for_type(Type::STRING);
        let drop_format = drop_format.union(&gdk4::ContentFormats::for_type(Type::U32));
        let drop_controller = DropTarget::builder()
            .preload(true)
            .actions(drop_actions)
            .formats(&drop_format)
            .build();
        drop_target_widget.add_controller(&drop_controller);

        let model = self.model();
        let list_view = &imp.list_view.get().unwrap();
        let drag_end = &imp.drag_end_signal;
        let drag_source = &imp.drag_source.get().unwrap();
        drop_controller.connect_drop(
            glib::clone!(@weak model, @weak list_view, @weak drag_end, @weak drag_source => @default-return true, move |_self, drop_value, x, _y| {
                //calculate insertion location
                let max_x = list_view.allocated_width();
                let n_buckets = model.n_items() * 2;

                let drop_bucket = (x * n_buckets as f64 / (max_x as f64 + 0.1)) as u32;
                let index = if drop_bucket == 0 {
                    0
                } else if drop_bucket == n_buckets - 1 {
                    model.n_items()
                } else {
                    (drop_bucket + 1) / 2
                };

                if let Ok(Some(path_str)) = drop_value.get::<Option<String>>() {
                    let desktop_path = &Path::new(&path_str);
                    if let Some(pathbase) = desktop_path.file_name() {
                        if let Some(app_info) = gio::DesktopAppInfo::new(&pathbase.to_string_lossy()) {
                            // remove item if already exists
                            let mut i: u32 = 0;
                            let mut index_of_existing_app: Option<u32> = None;
                            while let Some(item) = model.item(i) {
                                if let Ok(cur_app_info) = item.downcast::<DockObject>() {
                                    if let Some(cur_app_info) = cur_app_info.property::<Option<DesktopAppInfo>>("appinfo") {
                                        if cur_app_info.filename() == Some(Path::new(&path_str).to_path_buf()) {
                                            index_of_existing_app = Some(i);
                                        }
                                    }
                                }
                                i += 1;
                            }
                            if let Some(index_of_existing_app) = index_of_existing_app {
                                // remove existing entry
                                model.remove(index_of_existing_app);
                                if let Some(old_handle) = drag_end.replace(None) {
                                    glib::signal_handler_disconnect(&drag_source, old_handle);
                                }
                            }
                            model.insert(index, &DockObject::new(app_info));
                        }
                    }
                }
                else if let Ok(old_index) = drop_value.get::<u32>() {
                    if let Some(item) = model.item(old_index) {
                        if let Ok(dock_object) = item.downcast::<DockObject>() {
                            model.remove(old_index);
                            model.insert(index, &dock_object);
                            if let Some(old_handle) = drag_end.replace(None) {
                                glib::signal_handler_disconnect(&drag_source, old_handle);
                            }
                        }
                    }
                }
                else {
                    // dbg!("rejecting drop");
                    _self.reject();
                }
                glib::MainContext::default().spawn_local(async move {
                   let _ = TX.get().unwrap().send(Event::RefreshFromCache).await;
                });
                true
            }),
        );

        imp.drop_controller
            .set(drop_controller)
            .expect("Could not set dock dnd drop controller");
    }

    fn setup_drag(&self) {
        let imp = imp::DockList::from_instance(self);
        let type_ = imp.type_.get().unwrap();

        let actions = match type_ {
            &DockListType::Saved => gdk4::DragAction::MOVE,
            &DockListType::Active => gdk4::DragAction::COPY,
        };
        let drag_source = DragSource::builder()
            .name("dock drag source")
            .actions(actions)
            .build();

        let model = self.model();
        let list_view = imp.list_view.get().unwrap();
        let drag_end = &imp.drag_end_signal;
        let drag_cancel = &imp.drag_cancel_signal;
        let type_ = type_.clone();
        list_view.add_controller(&drag_source);
        drag_source.connect_prepare(glib::clone!(@weak model, @weak list_view, @weak drag_end, @weak drag_cancel => @default-return None, move |self_, x, _y| {
            let max_x = list_view.allocated_width();
            // dbg!(max_x);
            // dbg!(max_y);
            let n_buckets = model.n_items();

            let index = (x * n_buckets as f64 / (max_x as f64 + 0.1)) as u32;
            if let Some(item) = model.item(index) {
                if type_ == DockListType::Saved {
                    if let Some(old_handle) = drag_end.replace(Some(self_.connect_drag_end(
                        glib::clone!(@weak model => move |_self, _drag, _delete_data| {
                            if _delete_data {
                                model.remove(index);
                                glib::MainContext::default().spawn_local(async move {
                                    if let Some(tx) = TX.get() {
                                        let _ = tx.send(Event::RefreshFromCache).await;
                                    }
                                });
                            };
                        }),
                    ))) {
                        glib::signal_handler_disconnect(self_, old_handle);
                    }
                    if let Some(old_handle) = drag_cancel.replace(Some(self_.connect_drag_cancel(
                        glib::clone!(@weak model => @default-return false, move |_self, _drag, cancel_reason| {
                            if cancel_reason != gdk4::DragCancelReason::UserCancelled {
                                model.remove(index);
                                glib::MainContext::default().spawn_local(async move {
                                    if let Some(tx) = TX.get() {
                                        let _ = tx.send(Event::RefreshFromCache).await;
                                    }
                                });
                                true
                            } else  {
                                false
                            }
                        }),
                    ))) {
                        glib::signal_handler_disconnect(self_, old_handle);
                    }
                }
                if let Ok(dock_object) = item.downcast::<DockObject>() {
                    if let Some(app_info) = dock_object.property::<Option<DesktopAppInfo>>("appinfo") {
                        let icon = app_info
                            .icon()
                            .unwrap_or(Icon::for_string("image-missing").expect("Failed to set default icon"));

                        if let Some(default_display) = &Display::default() {
                            let icon_theme = IconTheme::for_display(default_display);
                            let paintable_icon = icon_theme.lookup_by_gicon(
                                &icon,
                                64,
                                1,
                                gtk4::TextDirection::None,
                                gtk4::IconLookupFlags::empty(),
                            );
                            self_.set_icon(Some(&paintable_icon), 32, 32);
                        }

                        // saved app list provides index
                        return match type_ {
                            DockListType::Saved => Some(ContentProvider::for_value(&index.to_value())),
                            DockListType::Active => app_info.filename().map(|file| ContentProvider::for_value(&file.to_string_lossy().to_value()))
                        }
                    }
                }
            }
            None
        }));

        // TODO investigate why drop does not finish when dropping on some surfaces
        // for now this is a fix that will cancel the drop after 100 ms and not completing.
        drag_source.connect_drag_begin(|_self, drag| {
            drag.connect_drop_performed(|_self| {
                glib::timeout_add_local_once(
                    std::time::Duration::from_millis(100),
                    glib::clone!(@weak _self => move || {
                        _self.drop_done(false);
                    }),
                );
            });
        });

        imp.drag_source
            .set(drag_source)
            .expect("Could not set saved drag source");
    }

    fn setup_factory(&self) {
        let imp = imp::DockList::from_instance(self);
        let popover_menu_index = &imp.popover_menu_index;
        let factory = SignalListItemFactory::new();
        let model = imp.model.get().expect("Failed to get saved app model.");
        factory.connect_setup(
            glib::clone!(@weak popover_menu_index, @weak model => move |_, list_item| {
                let dock_item = DockItem::new();
                dock_item
                    .connect_local("popover-closed", false, move |_| {
                        if let Some(old_index) = popover_menu_index.replace(None) {
                            if let Some(item) = model.item(old_index) {
                                if let Ok(dock_object) = item.downcast::<DockObject>() {
                                    dock_object.set_popover(false);
                                    model.items_changed(old_index, 0, 0);
                                }
                            }
                        }

                        None
                    });
                list_item.set_child(Some(&dock_item));
            }),
        );
        factory.connect_bind(move |_, list_item| {
            let dock_object = list_item
                .item()
                .expect("The item has to exist.")
                .downcast::<DockObject>()
                .expect("The item has to be a `DockObject`");
            let dock_item = list_item
                .child()
                .expect("The list item child needs to exist.")
                .downcast::<DockItem>()
                .expect("The list item type needs to be `DockItem`");
            dock_item.set_dock_object(&dock_object);
        });
        // Set the factory of the list view
        imp.list_view.get().unwrap().set_factory(Some(&factory));
    }
}
