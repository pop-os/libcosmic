use glib::signal::Inhibit;
use gtk4::glib;
use gtk4::subclass::prelude::*;
use gtk4::SearchEntry;
use once_cell::sync::OnceCell;

use crate::app_grid::AppGrid;
use crate::group_grid::GroupGrid;

// Object holding the state
#[derive(Default)]
pub struct AppLibraryWindow {
    pub entry: OnceCell<SearchEntry>,
    pub app_grid: OnceCell<AppGrid>,
    pub group_grid: OnceCell<GroupGrid>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for AppLibraryWindow {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "AppLibraryWindow";
    type Type = super::AppLibraryWindow;
    type ParentType = gtk4::ApplicationWindow;
}

// Trait shared by all GObjects
impl ObjectImpl for AppLibraryWindow {}

// Trait shared by all widgets
impl WidgetImpl for AppLibraryWindow {}

// Trait shared by all windows
impl WindowImpl for AppLibraryWindow {
    fn close_request(&self, window: &Self::Type) -> Inhibit {
        let imp = AppLibraryWindow::from_instance(window);
        imp.group_grid.get().unwrap().store_data();
        // Pass close request on to the parent
        self.parent_close_request(window)
    }
}

// Trait shared by all application
impl ApplicationWindowImpl for AppLibraryWindow {}
