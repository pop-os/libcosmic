use std::cell::RefCell;
use std::rc::Rc;

use glib::SignalHandlerId;
use gtk4::subclass::prelude::*;
use gtk4::DragSource;
use gtk4::DropTarget;
use gtk4::EventControllerMotion;
use gtk4::ListView;
use gtk4::Revealer;
use gtk4::{gio, glib};
use gtk4::{Box, GestureClick};
use once_cell::sync::OnceCell;

// Object holding the state
#[derive(Default)]
pub struct Window {
    pub saved_app_list_view: OnceCell<ListView>,
    pub active_app_list_view: OnceCell<ListView>,
    pub revealer: OnceCell<Revealer>,
    pub cursor_handle: OnceCell<Box>,
    pub saved_app_model: OnceCell<gio::ListStore>,
    pub active_app_model: OnceCell<gio::ListStore>,
    pub cursor_motion_controller: OnceCell<EventControllerMotion>,
    pub saved_click_controller: Rc<OnceCell<GestureClick>>,
    pub active_click_controller: Rc<OnceCell<GestureClick>>,
    pub drop_controller: OnceCell<DropTarget>,
    pub saved_drag_source: Rc<OnceCell<DragSource>>,
    pub active_drag_source: Rc<OnceCell<DragSource>>,
    pub saved_drag_end_signal: Rc<RefCell<Option<SignalHandlerId>>>,
    pub active_drag_end_signal: Rc<RefCell<Option<SignalHandlerId>>>,
    pub saved_drag_cancel_signal: Rc<RefCell<Option<SignalHandlerId>>>,
    pub active_drag_cancel_signal: Rc<RefCell<Option<SignalHandlerId>>>,
    pub window_drop_controller: Rc<OnceCell<DropTarget>>,
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
