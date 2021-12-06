use gtk4 as gtk;
use std::fs::File;

use glib::signal::Inhibit;
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk::{CompositeTemplate, GridView, SearchEntry};
use once_cell::sync::OnceCell;

use crate::app_group::AppGroup;
use crate::utils::data_path;

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(file = "window.ui")]
pub struct Window {
    #[template_child]
    pub entry: TemplateChild<SearchEntry>,
    #[template_child]
    pub app_grid_view: TemplateChild<GridView>,
    pub app_model: OnceCell<gio::ListStore>,
    #[template_child]
    pub group_grid_view: TemplateChild<GridView>,
    pub group_model: OnceCell<gio::ListStore>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "LauncherWindow";
    type Type = super::Window;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        Self::bind_template(klass);
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}
// Trait shared by all GObjects
impl ObjectImpl for Window {
    fn constructed(&self, obj: &Self::Type) {
        // Call "constructed" on parent
        self.parent_constructed(obj);

        // Setup
        obj.setup_model();
        obj.restore_data();
        obj.setup_callbacks();
        obj.setup_factory();
    }
}
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
