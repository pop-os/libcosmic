use gio::DesktopAppInfo;
use gio::Icon;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk4 as gtk;

use crate::dock_object::DockObject;
use crate::utils::BoxedWindowList;

mod imp;

glib::wrapper! {
    pub struct DockItem(ObjectSubclass<imp::DockItem>)
        @extends gtk::Widget, gtk::Box;
}

impl Default for DockItem {
    fn default() -> Self {
        Self::new()
    }
}

impl DockItem {
    pub fn new() -> Self {
        let self_: DockItem = glib::Object::new(&[]).expect("Failed to create DockItem");
        self_
    }

    // refactor to emit event for removing the item?
    pub fn set_app_info(&self, dock_object: &DockObject) {
        let self_ = imp::DockItem::from_instance(self);
        if let Ok(app_info_value) = dock_object.property("appinfo") {
            if let Ok(Some(app_info)) = app_info_value.get::<Option<DesktopAppInfo>>() {
                self_.image.set_tooltip_text(Some(&app_info.name()));

                let icon = app_info.icon().unwrap_or(
                    Icon::for_string("image-missing").expect("Failed to set default icon"),
                );

                self_.image.set_from_gicon(&icon);
            }
        } else {
            println!("initializing dock item failed...");
        }
        if let Ok(active_value) = dock_object.property("active") {
            if let Ok(active) = active_value.get::<BoxedWindowList>() {
                self_.dots.set_text("");
                for _ in active.0 {
                    self_
                        .dots
                        .set_text(format!("{}{}", self_.dots.text(), " · ").as_str());
                }
            }
        }
    }
}
