use crate::utils::BoxedSearchResults;
use gio::DesktopAppInfo;
use glib::{FromVariant, ParamFlags, ParamSpec, ToVariant, Value, Variant, VariantTy};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use once_cell::sync::Lazy;
use std::cell::RefCell;

// Object holding the state
#[derive(Default)]
pub struct DockObject {
    appinfo: RefCell<Option<DesktopAppInfo>>,
    active: RefCell<BoxedSearchResults>,
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
                ParamSpec::new_object(
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
                ParamSpec::new_boxed(
                    // Name
                    "active",
                    // Nickname
                    "active",
                    // Short description
                    "active",
                    BoxedSearchResults::static_type(),
                    // The property can be read and written to
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
                let active = value.get().expect("Value needs to be BoxedSearchResults");
                self.active.replace(active);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "appinfo" => self.appinfo.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}
