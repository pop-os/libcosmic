use glib::Object;
use gtk4::glib;
use gtk4::subclass::prelude::*;
use serde::{Deserialize, Serialize};

mod imp;

glib::wrapper! {
    pub struct AppGroup(ObjectSubclass<imp::AppGroup>);
}

impl AppGroup {
    pub fn new(data: BoxedAppGroupType) -> Self {
        let self_: Self =
            Object::new(&[("inner", &data)]).expect("Failed to create `ApplicationObject`.");
        self_
    }

    pub fn popup(&self) {
        let imp = imp::AppGroup::from_instance(self);
        let inner = imp.inner.borrow().clone();
        match inner {
            BoxedAppGroupType::Group(d) => {
                // d.popup = true;
                imp.inner.replace(BoxedAppGroupType::Group(d));
            }
            BoxedAppGroupType::NewGroup(_) => {
                imp.inner.replace(BoxedAppGroupType::NewGroup(true));
            }
        };
    }

    pub fn popdown(&self) {
        let imp = imp::AppGroup::from_instance(self);
        let inner = imp.inner.borrow().clone();
        match inner {
            BoxedAppGroupType::Group(d) => {
                // d.popup = false;
                imp.inner.replace(BoxedAppGroupType::Group(d));
            }
            BoxedAppGroupType::NewGroup(_) => {
                imp.inner.replace(BoxedAppGroupType::NewGroup(false));
            }
        };
    }

    pub fn is_popup_active(&self) -> bool {
        let imp = imp::AppGroup::from_instance(self);
        match imp.inner.borrow().clone() {
            BoxedAppGroupType::Group(_d) => false,
            BoxedAppGroupType::NewGroup(is_active) => is_active,
        }
    }

    pub fn group_data(&self) -> Option<AppGroupData> {
        let imp = imp::AppGroup::from_instance(self);
        let inner = imp.inner.borrow().clone();
        match inner {
            BoxedAppGroupType::Group(d) => Some(d),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, glib::Boxed)]
#[boxed_type(name = "BoxedAppGroupType")]
pub enum BoxedAppGroupType {
    Group(AppGroupData),
    NewGroup(bool),
}

impl Default for BoxedAppGroupType {
    fn default() -> Self {
        Self::NewGroup(false)
    }
}

// Object holding the state
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct AppGroupData {
    pub id: u32,
    pub name: String,
    pub icon: String,
    pub mutable: bool,
    pub app_names: Vec<String>,
    pub category: String,
    // pub popup: bool,
}
