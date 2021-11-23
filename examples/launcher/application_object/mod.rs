mod imp;

use gdk4::glib::Object;
use gtk4::glib;
use std::cell::RefCell;
use std::rc::Rc;

glib::wrapper! {
    pub struct ApplicationObject(ObjectSubclass<imp::ApplicationObject>);
}

impl ApplicationObject {
    pub fn new(application_search_result: &pop_launcher::SearchResult) -> Self {
        Object::new(&[("name", &application_search_result.name)])
            .expect("Failed to create `ApplicationObject`.")
    }
}

// Object holding the state
pub struct ApplicationData(pop_launcher::SearchResult);

impl Default for ApplicationData {
    fn default() -> Self {
        let default_application = pop_launcher::SearchResult {
            id: 0,
            name: String::default(),
            description: String::default(),
            icon: None,
            category_icon: None,
            window: None,
        };
        Self(default_application)
    }
}
