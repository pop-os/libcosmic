mod imp;

use crate::utils::BoxedSearchResults;
use gdk4::glib::Object;
use gio::DesktopAppInfo;
use gtk4::glib;
use gtk4::prelude::AppInfoExt;

glib::wrapper! {
    pub struct DockObject(ObjectSubclass<imp::DockObject>);
}

impl DockObject {
    pub fn new(appinfo: DesktopAppInfo) -> Self {
        Object::new(&[("appinfo", &Some(appinfo))]).expect("Failed to create `DockObject`.")
    }

    pub fn from_search_results(results: BoxedSearchResults) -> Self {
        let appinfo = if let Some(first) = results.0.iter().next() {
            xdg::BaseDirectories::new()
                .expect("could not access XDG Base directory")
                .get_data_dirs()
                .iter_mut()
                .filter_map(|xdg_data_path| {
                    xdg_data_path.push("applications");
                    dbg!(&xdg_data_path);
                    std::fs::read_dir(xdg_data_path).ok()
                })
                .flatten()
                .filter_map(|dir_entry| {
                    if let Ok(dir_entry) = dir_entry {
                        if let Some(path) = dir_entry.path().file_name() {
                            if let Some(path) = path.to_str() {
                                if let Some(app_info) = gio::DesktopAppInfo::new(path) {
                                    if app_info.should_show()
                                        && first.description.as_str() == app_info.name().as_str()
                                    {
                                        return Some(DockObject::new(app_info));
                                    }
                                }
                            }
                        }
                    }
                    None
                })
                .next()
        } else {
            None
        };
        Object::new(&[("appinfo", &appinfo), ("active", &results)])
            .expect("Failed to create `DockObject`.")
    }
}
