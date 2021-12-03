use crate::app_group::AppGroup;
use gtk4 as gtk;
mod imp;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

glib::wrapper! {
pub struct GridItem(ObjectSubclass<imp::GridItem>)
    @extends gtk::Widget, gtk::Box,
    @implements gtk::Accessible, gtk::Actionable, gtk::Buildable, gtk::ConstraintTarget;
}

impl Default for GridItem {
    fn default() -> Self {
        Self::new()
    }
}

impl GridItem {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create GridItem")
    }

    pub fn set_app_info(&self, app_info: &gio::DesktopAppInfo) {
        let self_ = imp::GridItem::from_instance(self);
        self_.name.set_text(&app_info.name());
        if let Some(icon) = app_info.icon() {
            self_.image.set_from_gicon(&icon);
        }
    }

    pub fn set_group_info(&self, app_group: AppGroup) {
        let self_ = imp::GridItem::from_instance(self);
        if let Ok(name) = app_group.property("name") {
            self_.name.set_text(
                &name
                    .get::<String>()
                    .expect("property name needs to be a string."),
            );
        }
        if let Ok(icon) = app_group.property("icon") {
            self_.image.set_from_icon_name(Some(
                &icon
                    .get::<String>()
                    .expect("Property name needs to be a String."),
            ));
        }
    }

    pub fn set_index(&self, index: u32) {
        imp::GridItem::from_instance(self).index.set(index);
    }
}
