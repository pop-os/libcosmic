use cascade::cascade;
use gtk4::ffi::gtk_window_close;
use gtk4::subclass::prelude::*;
use gtk4::{gio, glib};
use gtk4::{prelude::*, Label};
use gtk4::{Box, Button, Image, ListBox, Orientation, Revealer};

use crate::dock_object::DockObject;
use crate::utils::BoxedWindowList;

mod imp;

glib::wrapper! {
    pub struct DockPopover(ObjectSubclass<imp::DockPopover>)
        @extends gtk4::Widget, gtk4::Box,
    @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl Default for DockPopover {
    fn default() -> Self {
        Self::new()
    }
}

impl DockPopover {
    pub fn new() -> Self {
        let self_: DockPopover = glib::Object::new(&[]).expect("Failed to create DockList");
        self_.layout();
        //dnd behavior is different for each type, as well as the data in the model
        self_
    }

    pub fn set_dock_object(&self, dock_object: &DockObject, update_layout: bool) {
        let imp = imp::DockPopover::from_instance(&self);
        imp.dock_object.replace(Some(dock_object.clone()));
        if update_layout {
            self.update_layout();
        }
    }

    pub fn update_layout(&self) {
        let imp = imp::DockPopover::from_instance(&self);
        let dock_object = imp.dock_object.borrow();

        // reset menu
        let menu_handle = cascade! {
            Box::new(Orientation::Vertical, 4);
        };
        self.append(&menu_handle);

        // build menu
        if let Some(dock_object) = dock_object.as_ref() {
            cascade! {
                &self;
                ..set_spacing(4);
                ..set_orientation(Orientation::Vertical);
                ..set_hexpand(true);
            };

            let all_windows_item_container = cascade! {
                Box::new(Orientation::Vertical, 4);
            };
            menu_handle.append(&all_windows_item_container);

            let all_windows_item_header = cascade! {
                Button::new();
                ..set_hexpand(true);
            };
            all_windows_item_container.append(&all_windows_item_header);

            let all_windows_item_header_box = cascade! {
                Box::new(Orientation::Horizontal, 4);
                ..set_hexpand(true);
            };
            all_windows_item_header.set_child(Some(&all_windows_item_header_box));

            let all_windows_item_header_title = cascade! {
                Label::new(Some("All Windows"));
                ..add_css_class("header-5");
                ..set_halign(gtk4::Align::Start);
            };
            all_windows_item_header_box.append(&all_windows_item_header_title);

            let all_windows_item_header_icon = cascade! {
                Image::from_icon_name(Some("go-down"));
                ..set_halign(gtk4::Align::End);
                ..set_pixel_size(16);
            };
            all_windows_item_header_box.append(&all_windows_item_header_icon);

            if let Ok(window_list) = dock_object
                .property("active")
                .unwrap()
                .get::<BoxedWindowList>()
            {
                if window_list.0.len() == 0 {
                    all_windows_item_container.hide();
                } else {
                    let window_list_revealer = cascade! {
                        Revealer::new();
                        ..set_reveal_child(true);
                        ..set_transition_type(gtk4::RevealerTransitionType::SlideDown);
                    };
                    all_windows_item_container.append(&window_list_revealer);
                    let window_listbox = cascade! {
                        ListBox::new();
                    };
                    window_list_revealer.set_child(Some(&window_listbox));
                    for w in window_list.0 {
                        let window_box = cascade! {
                            Box::new(Orientation::Vertical, 4);
                        };
                        window_listbox.append(&window_box);

                        let window_title = cascade! {
                            Label::new(Some(w.name.as_str()));
                        };

                        // TODO investigate Xembed
                        let window_image = cascade! {
                            //TODO fill with image of window
                            Image::from_pixbuf(None);
                        };
                        window_box.append(&window_image);
                        window_box.append(&window_title);
                    }
                }
            }

            let launch_item_container = cascade! {
                Box::new(Orientation::Vertical, 4);
                ..set_hexpand(true);
            };
            menu_handle.append(&launch_item_container);

            let launch_new_item = cascade! {
                Button::with_label("New Window");
            };
            launch_item_container.append(&launch_new_item);

            let favorite_item = cascade! {
                Button::with_label(if dock_object.property("saved").unwrap().get::<bool>().unwrap() {"Remove from Favorites"} else {"Add to Favorites"});
            };
            menu_handle.append(&favorite_item);

            if let Ok(window_list) = dock_object
                .property("active")
                .unwrap()
                .get::<BoxedWindowList>()
            {
                if window_list.0.len() > 1 {
                    let quit_all_item = cascade! {
                        Button::with_label(format!("Quit {} Windows", window_list.0.len()).as_str());
                    };
                    menu_handle.append(&quit_all_item);
                } else {
                    let quit_all_item = cascade! {
                        Button::with_label("Quit");
                    };
                    menu_handle.append(&quit_all_item);
                    if window_list.0.len() == 0 {
                        quit_all_item.hide();
                    }
                }
            }
            self.setup_handlers();
        }
        let old_menu_handle = imp.menu_handle.replace(menu_handle);
        self.remove(&old_menu_handle);
    }

    fn layout(&self) {
        let imp = imp::DockPopover::from_instance(&self);
        let menu_handle = cascade! {
            Box::default();
        };
        self.append(&menu_handle);
        imp.menu_handle.replace(menu_handle);
    }

    fn setup_handlers(&self) {
        // todo!();
    }
}
