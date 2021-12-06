use gtk4::ScrolledWindow;
use std::path::PathBuf;

use gtk4::glib;

pub fn data_path() -> PathBuf {
    let mut path = glib::user_data_dir();
    path.push("com.cosmic.app_library");
    std::fs::create_dir_all(&path).expect("Could not create directory.");
    path.push("data.json");
    path
}

pub fn set_group_scroll_policy(scroll_window: &ScrolledWindow, group_cnt: u32) {
    if scroll_window.policy().1 == gtk4::PolicyType::Never && group_cnt > 16 {
        scroll_window.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    } else if scroll_window.policy().1 == gtk4::PolicyType::Automatic && group_cnt <= 16 {
        scroll_window.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Never);
    }
}
