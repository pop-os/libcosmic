use gtk4::subclass::prelude::*;
use gtk4::{gio, glib};
use gtk4::{Entry, ListView};
use once_cell::sync::OnceCell;

// Object holding the state
#[derive(Default)]
pub struct Window {
    pub entry: OnceCell<Entry>,
    pub list_view: OnceCell<ListView>,
    pub model: OnceCell<gio::ListStore>,
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
impl WindowImpl for Window {}

// Trait shared by all application
impl ApplicationWindowImpl for Window {}
