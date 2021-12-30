use glib::Object;
use glib::ObjectExt;
use glib::ToVariant;
use gtk4::glib;
use gtk4::subclass::prelude::*;
use serde::{Deserialize, Serialize};

mod imp;

glib::wrapper! {
    pub struct AppGroup(ObjectSubclass<imp::AppGroup>);
}

impl AppGroup {
    pub fn new(data: AppGroupData) -> Self {
        let self_: Self = Object::new(&[
            ("id", &data.id),
            ("name", &data.name),
            ("mutable", &data.mutable),
            ("icon", &data.icon),
            ("category", &data.category),
        ])
            .expect("Failed to create `ApplicationObject`.");
        if let Err(e) = self_.set_property("appnames", data.app_names.to_variant()) {
            println!("failed to set category icon property");
            dbg!(e);
        };
        self_
    }

    pub fn group_data(&self) -> AppGroupData {
        let imp = imp::AppGroup::from_instance(self);
        imp.data.borrow().clone()
    }
}

// Object holding the state
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct AppGroupData {
    pub id: u32,
    pub name: String,
    pub icon: String,
    pub mutable: bool,
    pub app_names: Vec<String>,
    pub category: String,
}
