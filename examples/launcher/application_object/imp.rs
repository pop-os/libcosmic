use crate::utils::BoxedSearchResult;
use glib::{ParamFlags, ParamSpec, Value};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use once_cell::sync::Lazy;

use std::cell::RefCell;
use std::rc::Rc;

// Object holding the state
#[derive(Default)]
pub struct ApplicationObject {
    data: Rc<RefCell<BoxedSearchResult>>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ApplicationObject {
    const NAME: &'static str = "ApplicationObject";
    type Type = super::ApplicationObject;
    type ParentType = glib::Object;
}

// Trait shared by all GObjects
impl ObjectImpl for ApplicationObject {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![ParamSpec::new_boxed(
                // Name
                "data",
                // Nickname
                "data",
                // Short description
                "data",
                BoxedSearchResult::static_type(),
                // The property can be read and written to
                ParamFlags::READWRITE,
            )]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "data" => {
                let data = value.get().expect("Value needs to be BoxedSearchResult");
                self.data.replace(data);
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "data" => self.data.borrow().to_value(),
            _ => unimplemented!(),
        }
    }
}
