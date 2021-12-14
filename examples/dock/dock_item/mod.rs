use crate::icon_source;
use glib::FromVariant;
use glib::Variant;
use gtk4 as gtk;
mod imp;

use crate::ApplicationObject;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct ApplicationRow(ObjectSubclass<imp::ApplicationRow>)
        @extends gtk::Widget, gtk::Box;
}

impl Default for ApplicationRow {
    fn default() -> Self {
        Self::new()
    }
}

impl ApplicationRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create ApplicationRow")
    }

    pub fn set_app_info(&self, app_obj: ApplicationObject) {
        let self_ = imp::ApplicationRow::from_instance(self);

        if let Ok(name) = app_obj.property("name") {
            self_.name.set_text(
                &name
                    .get::<String>()
                    .expect("Property name needs to be a String."),
            );
        }
        if let Ok(desc) = app_obj.property("description") {
            self_.description.set_text(
                &desc
                    .get::<String>()
                    .expect("Property description needs to be a String."),
            );
        }
        if let Ok(icon) = app_obj.property("icon") {
            if let Ok(icon) = icon.get::<Variant>() {
                let icon = match <(i32, String)>::from_variant(&icon) {
                    Some((i_type, name)) if i_type == pop_launcher::IconSource::Name as i32 => {
                        Some(pop_launcher::IconSource::Name(name.into()))
                    }
                    Some((i_type, name)) if i_type == pop_launcher::IconSource::Mime as i32 => {
                        Some(pop_launcher::IconSource::Mime(name.into()))
                    }
                    _ => None,
                };
                icon_source(&self_.image, &icon);
            }
        }
        if let Ok(icon) = app_obj.property("categoryicon") {
            if let Ok(icon) = icon.get::<Variant>() {
                let icon = match <(i32, String)>::from_variant(&icon) {
                    Some((i_type, name)) if i_type == pop_launcher::IconSource::Name as i32 => {
                        Some(pop_launcher::IconSource::Name(name.into()))
                    }
                    Some((i_type, name)) if i_type == pop_launcher::IconSource::Mime as i32 => {
                        Some(pop_launcher::IconSource::Mime(name.into()))
                    }
                    _ => None,
                };
                icon_source(&self_.categoryimage, &icon);
            }
        }
    }

    pub fn set_shortcut(&self, indx: u32) {
        let self_ = imp::ApplicationRow::from_instance(self);
        self_.shortcut.set_text(&format!("Ctrl + {}", indx));
    }
}
