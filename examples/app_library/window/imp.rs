use std::fs::File;

use glib::signal::Inhibit;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::ScrolledWindow;
use gtk4::{gio, glib};
use gtk4::{GridView, SearchEntry};
use once_cell::sync::OnceCell;

use crate::app_group::AppGroup;
use crate::utils::data_path;

// Object holding the state
#[derive(Default)]
pub struct Window {
    pub entry: OnceCell<SearchEntry>,
    pub app_grid_view: OnceCell<GridView>,
    pub app_model: OnceCell<gio::ListStore>,
    pub group_grid_view: OnceCell<GridView>,
    pub group_scroll_window: OnceCell<ScrolledWindow>,
    pub group_model: OnceCell<gio::ListStore>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "LauncherWindow";
    type Type = super::Window;
    type ParentType = gtk4::ApplicationWindow;
}

// Trait shared by all GObjects
impl ObjectImpl for Window {}

// Trait shared by all widgets
impl WidgetImpl for Window {}

// Trait shared by all windows
impl WindowImpl for Window {
    fn close_request(&self, window: &Self::Type) -> Inhibit {
        // Store todo data in vector
        let mut backup_data = Vec::new();
        let mut position = 3;
        while let Some(item) = window.group_model().item(position) {
            if position == window.group_model().n_items() - 1 {
                break;
            }
            // Get `AppGroup` from `glib::Object`
            let group_data = item
                .downcast_ref::<AppGroup>()
                .expect("The object needs to be of type `AppGroupData`.")
                .group_data();
            // Add todo data to vector and increase position
            backup_data.push(group_data);
            position += 1;
        }

        // Save state in file
        let file = File::create(data_path()).expect("Could not create json file.");
        serde_json::to_writer_pretty(file, &backup_data)
            .expect("Could not write data to json file");

        // Pass close request on to the parent
        self.parent_close_request(window)
    }
}

// Trait shared by all application
impl ApplicationWindowImpl for Window {}
