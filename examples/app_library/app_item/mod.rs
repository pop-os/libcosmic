use gtk4 as gtk;
mod imp;

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};

glib::wrapper! {
    pub struct AppItem(ObjectSubclass<imp::AppItem>)
        @extends gtk::Widget, gtk::Box;
}

impl Default for AppItem {
    fn default() -> Self {
        Self::new()
    }
}

impl AppItem {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create AppItem")
    }

    pub fn set_app_info(&self, app_info: &gio::AppInfo) {
        let self_ = imp::AppItem::from_instance(self);
        self_.name.set_text(&app_info.name());
        if let Some(icon) = app_info.icon() {
            self_.image.set_from_gicon(&icon);
        }
    }
}
