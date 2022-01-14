use crate::utils::BoxedSearchResult;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

mod imp;

glib::wrapper! {
    pub struct SearchResultObject(ObjectSubclass<imp::SearchResultObject>);
}

impl SearchResultObject {
    pub fn new(search_result: &BoxedSearchResult) -> Self {
        glib::Object::new(&[("data", search_result)]).expect("Failed to create Application Object")
    }

    pub fn data(&self) -> Option<pop_launcher::SearchResult> {
        if let Ok(data) = self.property("data") {
            if let Ok(search_result) = data.get::<BoxedSearchResult>() {
                return search_result.0;
            }
        }
        None
    }
}
