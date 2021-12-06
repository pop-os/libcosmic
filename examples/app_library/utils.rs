use std::path::PathBuf;

use gtk4::glib;

pub fn data_path() -> PathBuf {
    let mut path = glib::user_data_dir();
    path.push("com.cosmic.app_library");
    std::fs::create_dir_all(&path).expect("Could not create directory.");
    path.push("data.json");
    path
}
