use glib::{FromVariant, ParamFlags, ParamSpec, ToVariant, Value, Variant, VariantTy};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use once_cell::sync::Lazy;

use std::cell::RefCell;
use std::rc::Rc;

use super::AppGroupData;

// Object holding the state
#[derive(Default)]
pub struct AppGroup {
    data: Rc<RefCell<AppGroupData>>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for AppGroup {
    const NAME: &'static str = "AppGroup";
    type Type = super::AppGroup;
    type ParentType = glib::Object;
}

// Trait shared by all GObjects
impl ObjectImpl for AppGroup {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                ParamSpec::new_uint(
                    // Name
                    "id",
                    // Nickname
                    "id",
                    // Short description
                    "Name of the application group",
                    0,
                    u32::MAX,
                    // Default value
                    0,
                    // The property can be read and written to
                    ParamFlags::READWRITE,
                ),
                ParamSpec::new_string(
                    // Name
                    "name",
                    // Nickname
                    "name",
                    // Short description
                    "Name of the application group",
                    // Default value
                    Some(""),
                    // The property can be read and written to
                    ParamFlags::READWRITE,
                ),
                ParamSpec::new_boolean(
                    // Name
                    "mutable",
                    // Nickname
                    "mutable",
                    // Short description
                    "Mutability of the application group",
                    // Default value
                    false,
                    // The property can be read and written to
                    ParamFlags::READWRITE,
                ),
                ParamSpec::new_string(
                    // Name
                    "icon",
                    // Nickname
                    "icon",
                    // Short description
                    "Icon of application Group",
                    // Default value
                    Some("folder"),
                    // The property can be read and written to
                    ParamFlags::READWRITE,
                ),
                ParamSpec::new_string(
                    // Name
                    "category",
                    // Nickname
                    "category",
                    // Short description
                    "Category of application Group",
                    // Default value
                    None,
                    // The property can be read and written to
                    ParamFlags::READWRITE,
                ),
                ParamSpec::new_variant(
                    // Name
                    "appnames",
                    // Nickname
                    "appnames",
                    // Short description
                    "Names of applications in the App Group",
                    VariantTy::new("as").expect("Oops invalid string for VariantTy tuple."),
                    // Default value
                    None,
                    // The property can be read and written to
                    ParamFlags::READWRITE,
                ),
            ]
        });
        PROPERTIES.as_ref()
    }

    fn set_property(&self, _obj: &Self::Type, _id: usize, value: &Value, pspec: &ParamSpec) {
        match pspec.name() {
            "id" => {
                let id = value.get().expect("The value needs to be of type `u32`.");
                self.data.borrow_mut().id = id;
            }
            "name" => {
                let name = value
                    .get()
                    .expect("The value needs to be of type `String`.");
                self.data.borrow_mut().name = name;
            }
            "icon" => {
                let icon = value.get().expect("The icon needs to be of type `String`");
                self.data.borrow_mut().icon = icon;
            }
            "category" => {
                let category = value
                    .get()
                    .expect("The category needs to be of type `String`");
                self.data.borrow_mut().category = category;
            }
            "mutable" => {
                let mutable = value
                    .get()
                    .expect("The mutable property needs to be of type `bool`");
                self.data.borrow_mut().mutable = mutable;
            }
            "appnames" => {
                let appnames = <Vec<String>>::from_variant(
                    &value
                        .get::<Variant>()
                        .expect("The icon needs to be a Variant"),
                )
                .expect("The icon variant needs to be a Vec<String>");
                self.data.borrow_mut().app_names = appnames;
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "id" => self.data.borrow().id.to_value(),
            "name" => self.data.borrow().name.to_value(),
            "icon" => self.data.borrow().icon.to_value(),
            "mutable" => self.data.borrow().mutable.to_value(),
            "category" => self.data.borrow().category.to_value(),
            "appnames" => self.data.borrow().app_names.to_variant().to_value(),
            _ => unimplemented!(),
        }
    }
}
