use std::path::PathBuf;

use gtk4::glib;

use crate::DockObject;
use crate::Item;

#[derive(Clone, Debug, Default, glib::GBoxed)]
#[gboxed(type_name = "BoxedWindowList")]
pub struct BoxedWindowList(pub Vec<Item>);

#[derive(Clone, Debug, Default, glib::GBoxed)]
#[gboxed(type_name = "BoxedDockObject")]
pub struct BoxedDockObject(pub Option<DockObject>);

pub fn data_path() -> PathBuf {
    let mut path = glib::user_data_dir();
    path.push("com.cosmic.dock");
    std::fs::create_dir_all(&path).expect("Could not create directory.");
    path.push("data.json");
    path
}
