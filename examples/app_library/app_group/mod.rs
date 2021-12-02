mod imp;

use gdk4::glib::Object;
use glib::ObjectExt;
use glib::ToVariant;
use gtk4::glib;
use serde::{Deserialize, Serialize};

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
