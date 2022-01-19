use crate::utils::BoxedSearchResult;
use gtk4::glib;
use gtk4::prelude::*;

mod imp;

glib::wrapper! {
    pub struct SearchResultObject(ObjectSubclass<imp::SearchResultObject>);
}

impl SearchResultObject {
    pub fn new(search_result: &BoxedSearchResult) -> Self {
        glib::Object::new(&[("data", search_result)]).expect("Failed to create Application Object")
    }

    pub fn data(&self) -> Option<pop_launcher::SearchResult> {
        let search_result = self.property::<BoxedSearchResult>("data");
        return search_result.0;
    }
}
