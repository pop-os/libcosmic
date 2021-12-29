use gdk4::glib::Object;

use crate::utils::BoxedSearchResult;

mod imp;

glib::wrapper! {
    pub struct SearchResultObject(ObjectSubclass<imp::SearchResultObject>);
}

impl SearchResultObject {
    pub fn new(search_result: &BoxedSearchResult) -> Self {
        Object::new(&[("data", search_result)]).expect("Failed to create Application Object")
    }
}
