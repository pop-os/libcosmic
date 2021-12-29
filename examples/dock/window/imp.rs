use std::cell::RefCell;
use std::rc::Rc;

use glib::SignalHandlerId;
use glib::subclass::InitializingObject;
use gtk::{gio, glib};
use gtk::{CompositeTemplate, ListView};
use gtk4 as gtk;
use gtk4::Box;
use gtk4::DragSource;
use gtk4::DropTarget;
use gtk4::EventControllerMotion;
use gtk4::Revealer;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use once_cell::sync::OnceCell;

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(file = "window.ui")]
pub struct Window {
    #[template_child]
    pub saved_app_list_view: TemplateChild<ListView>,
    #[template_child]
    pub active_app_list_view: TemplateChild<ListView>,
    #[template_child]
    pub revealer: TemplateChild<Revealer>,
    #[template_child]
    pub cursor_handle: TemplateChild<Box>,
    pub saved_app_model: OnceCell<gio::ListStore>,
    pub active_app_model: OnceCell<gio::ListStore>,
    pub cursor_event_controller: OnceCell<EventControllerMotion>,
    pub drop_controller: OnceCell<DropTarget>,
    pub saved_drag_source: Rc<OnceCell<DragSource>>,
    pub active_drag_source: OnceCell<DragSource>,
    pub saved_drag_end_signal: Rc<RefCell<Option<SignalHandlerId>>>,
    pub active_drag_end_signal: Rc<RefCell<Option<SignalHandlerId>>>,
    pub saved_drag_cancel_signal: Rc<RefCell<Option<SignalHandlerId>>>,
    pub active_drag_cancel_signal: Rc<RefCell<Option<SignalHandlerId>>>,
    pub window_drop_controller: OnceCell<DropTarget>,
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
        obj.setup_event_controller();
        obj.setup_drop_target();
        obj.setup_drag_source();
        obj.restore_saved_apps();
        obj.setup_callbacks();
        obj.setup_factory();
    }
}

// Trait shared by all widgets
impl WidgetImpl for Window {}

// Trait shared by all windows
impl WindowImpl for Window {}

// Trait shared by all application
impl ApplicationWindowImpl for Window {}
