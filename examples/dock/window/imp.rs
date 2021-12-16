use gtk4 as gtk;
use gtk4::Box;
use gtk4::DropTarget;
use gtk4::EventControllerMotion;
use gtk4::Revealer;

use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk::{CompositeTemplate, ListView};
use once_cell::sync::OnceCell;

// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(file = "window.ui")]
pub struct Window {
    #[template_child]
    pub saved_app_list_view: TemplateChild<ListView>,
    #[template_child]
    pub unsaved_open_app_list_view: TemplateChild<ListView>,
    #[template_child]
    pub revealer: TemplateChild<Revealer>,
    #[template_child]
    pub cursor_enter_handle: TemplateChild<Box>,
    #[template_child]
    pub cursor_leave_handle: TemplateChild<Box>,
    pub saved_app_model: OnceCell<gio::ListStore>,
    pub unsaved_open_app_model: OnceCell<gio::ListStore>,
    pub enter_event_controller: OnceCell<EventControllerMotion>,
    pub leave_event_controller: OnceCell<EventControllerMotion>,
    pub drop_controller: OnceCell<DropTarget>,
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
        obj.setup_event_controller();
        obj.setup_drop_target();
        obj.setup_model();
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
