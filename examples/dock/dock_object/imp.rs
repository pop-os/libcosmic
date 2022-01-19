use std::cell::Cell;
use std::cell::RefCell;

use gdk4::glib::ParamSpecBoolean;
use gdk4::glib::ParamSpecBoxed;
use gdk4::glib::ParamSpecObject;
use gio::DesktopAppInfo;
use glib::{ParamFlags, ParamSpec, Value};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use once_cell::sync::Lazy;

use crate::plugin::BoxedDockPlugin;
use crate::utils::BoxedWindowList;

// Object holding the state
#[derive(Default)]
pub struct DockObject {
    pub(super) appinfo: RefCell<Option<DesktopAppInfo>>,
    pub(super) active: RefCell<BoxedWindowList>,
    pub(super) plugin: RefCell<Option<BoxedDockPlugin>>,
    pub(super) saved: Cell<bool>,
    pub(super) popover: Cell<bool>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for DockObject {
    const NAME: &'static str = "DockObject";
    type Type = super::DockObject;
    type ParentType = glib::Object;
}

// Trait shared by all GObjects
impl ObjectImpl for DockObject {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                ParamSpecObject::new(
                    // Name
                    "appinfo",
                    // Nickname
                    "appinfo",
                    // Short description
                    "app info",
                    DesktopAppInfo::static_type(),
                    // The property can be read and written to
                    ParamFlags::READWRITE,
                ),
                ParamSpecBoxed::new(
                    // Name
                    "active",
                    // Nickname
                    "active",
                    // Short description
                    "active",
                    BoxedWindowList::static_type(),
                    // The property can be read and written to
                    ParamFlags::READWRITE,
                ),
                ParamSpecBoolean::new(
                    "saved",
                    "saved",
                    "Indicates whether app is saved to the dock",
                    false,
                    ParamFlags::READWRITE,
                ),
                ParamSpecBoolean::new(
                    "popover",
                    "popover",
                    "Indicates whether there is a popover menu displayed for this object",
                    false,
                    ParamFlags::READWRITE,
                ),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "appinfo" => {
                let appinfo = value
                    .get()
                    .expect("Value needs to be Option<DesktopAppInfo>");
                self.appinfo.replace(appinfo);
            }
            "active" => {
                let active = value.get().expect("Value needs to be BoxedWindowList");
                self.active.replace(active);
            }
            "saved" => {
                self.saved
                    .replace(value.get().expect("Value needs to be a boolean"));
            }
            "popover" => {
                self.popover
                    .replace(value.get().expect("Value needs to be a boolean"));
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "appinfo" => self.appinfo.borrow().to_value(),
            "active" => self.active.borrow().to_value(),
            "saved" => self.saved.get().to_value(),
            "popover" => self.popover.get().to_value(),
            _ => unimplemented!(),
        }
    }
}
