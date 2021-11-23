use glib::{
    types::Type, FromVariant, ParamFlags, ParamSpec, ParamSpecObject, ToVariant, Value, Variant,
    VariantTy,
};
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use once_cell::sync::Lazy;

use std::cell::RefCell;
use std::rc::Rc;

use super::ApplicationData;

// Object holding the state
#[derive(Default)]
pub struct ApplicationObject {
    data: Rc<RefCell<ApplicationData>>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ApplicationObject {
    const NAME: &'static str = "LibCosmicLauncherApplicationObject";
    type Type = super::ApplicationObject;
    type ParentType = glib::Object;
}

// Trait shared by all GObjects
impl ObjectImpl for ApplicationObject {
    fn properties() -> &'static [ParamSpec] {
        static PROPERTIES: Lazy<Vec<ParamSpec>> = Lazy::new(|| {
            vec![
                ParamSpec::new_string(
                    // Name
                    "name",
                    // Nickname
                    "name",
                    // Short description
                    "Name of application in launcher search result",
                    // Default value
                    Some(""),
                    // The property can be read and written to
                    ParamFlags::READWRITE,
                ),
                ParamSpec::new_string(
                    // Name
                    "description",
                    // Nickname
                    "description",
                    // Short description
                    "Description of application in launcher search result",
                    // Default value
                    Some(""),
                    // The property can be read and written to
                    ParamFlags::READWRITE,
                ),
                ParamSpec::new_variant(
                    // Name
                    "icon",
                    // Nickname
                    "icon",
                    // Short description
                    "Icon of application in launcher search result",
                    VariantTy::new("(is)").expect("Oops invalid string for VariantTy tuple."),
                    // Default value
                    None,
                    // The property can be read and written to
                    ParamFlags::READWRITE,
                ),
                ParamSpec::new_variant(
                    // Name
                    "categoryicon",
                    // Nickname
                    "categoryicon",
                    // Short description
                    "Category icon of application in launcher search result",
                    VariantTy::new("(is)").expect("Oops invalid string for VariantTy tuple."),
                    // Default value
                    None,
                    // The property can be read and written to
                    ParamFlags::READWRITE,
                ),
                ParamSpec::new_variant(
                    // Name
                    "window",
                    // Nickname
                    "window",
                    // Short description
                    "Window of application in launcher search result",
                    // type (tuple of two uint32)
                    VariantTy::new("(uu)").expect("Oops invalid string for VariantTy tuple."),
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
            "name" => {
                let name = value
                    .get()
                    .expect("The value needs to be of type `String`.");
                self.data.borrow_mut().0.name = name;
            }
            "description" => {
                let description = value
                    .get()
                    .expect("The description needs to be of type `String`");
                self.data.borrow_mut().0.description = description;
            }
            "icon" => {
                let icon = <Option<(i32, String)>>::from_variant(
                    &value
                        .get::<Variant>()
                        .expect("The icon needs to be a Variant"),
                )
                .expect("The icon variant needs to be an Option<(i32, String)>");
                if let Some(icon) = icon {
                    self.data.borrow_mut().0.icon = match icon {
                        (i_type, name) if i_type == pop_launcher::IconSource::Name as i32 => {
                            Some(pop_launcher::IconSource::Name(name.into()))
                        }
                        (i_type, name) if i_type == pop_launcher::IconSource::Mime as i32 => {
                            Some(pop_launcher::IconSource::Mime(name.into()))
                        }
                        _ => None,
                    };
                } else {
                    self.data.borrow_mut().0.icon = None;
                }
            }
            "categoryicon" => {
                let icon = <Option<(i32, String)>>::from_variant(
                    &value
                        .get::<Variant>()
                        .expect("The icon needs to be a Variant"),
                )
                .expect("The icon variant needs to be an Option<(i32, String)>");
                if let Some(icon) = icon {
                    self.data.borrow_mut().0.icon = match icon {
                        (i_type, name) if i_type == pop_launcher::IconSource::Name as i32 => {
                            Some(pop_launcher::IconSource::Name(name.into()))
                        }
                        (i_type, name) if i_type == pop_launcher::IconSource::Mime as i32 => {
                            Some(pop_launcher::IconSource::Mime(name.into()))
                        }
                        _ => None,
                    };
                } else {
                    self.data.borrow_mut().0.icon = None;
                }
            }
            "window" => {
                unimplemented!()
            }
            _ => unimplemented!(),
        }
    }

    fn property(&self, _obj: &Self::Type, _id: usize, pspec: &ParamSpec) -> Value {
        match pspec.name() {
            "name" => self.data.borrow().0.name.to_value(),
            "description" => self.data.borrow().0.name.to_value(),
            "icon" => match &self.data.borrow().0.icon {
                Some(pop_launcher::IconSource::Name(icon_name)) => {
                    (pop_launcher::IconSource::Name as i32, icon_name.to_string())
                        .to_variant()
                        .to_value()
                }
                Some(pop_launcher::IconSource::Mime(icon_name)) => {
                    (pop_launcher::IconSource::Mime as i32, icon_name.to_string())
                        .to_variant()
                        .to_value()
                }
                _ => None::<Variant>.to_value(),
            },
            "categoryicon" => match &self.data.borrow().0.category_icon {
                Some(pop_launcher::IconSource::Name(icon_name)) => {
                    (pop_launcher::IconSource::Name as i32, icon_name.to_string())
                        .to_variant()
                        .to_value()
                }
                Some(pop_launcher::IconSource::Mime(icon_name)) => {
                    (pop_launcher::IconSource::Mime as i32, icon_name.to_string())
                        .to_variant()
                        .to_value()
                }
                _ => None::<Variant>.to_value(),
            },
            _ => unimplemented!(),
        }
    }
}
