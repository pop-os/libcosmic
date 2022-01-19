use crate::window_inner::AppLibraryWindowInner;
use gtk4::glib;
use gtk4::subclass::prelude::*;
use once_cell::sync::OnceCell;

// Object holding the state
#[derive(Default)]

pub struct AppLibraryWindow {
    pub(super) inner: OnceCell<AppLibraryWindowInner>,
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
impl WindowImpl for AppLibraryWindow {}

// Trait shared by all application
impl ApplicationWindowImpl for AppLibraryWindow {}
