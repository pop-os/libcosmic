use gtk4::glib;
use gtk4::subclass::prelude::*;
use gtk4::Box;
use gtk4::DropTarget;
use gtk4::EventControllerMotion;
use gtk4::Revealer;
use once_cell::sync::OnceCell;

use crate::dock_list::DockList;

// Object holding the state
#[derive(Default)]
pub struct Window {
    pub revealer: OnceCell<Revealer>,
    pub cursor_handle: OnceCell<Box>,
    pub cursor_motion_controller: OnceCell<EventControllerMotion>,
    pub window_drop_controller: OnceCell<DropTarget>,
    pub saved_list: OnceCell<DockList>,
    pub active_list: OnceCell<DockList>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "DockWindow";
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
