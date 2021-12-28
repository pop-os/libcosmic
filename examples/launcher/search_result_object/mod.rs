mod imp;
use crate::utils::BoxedSearchResult;
use gdk4::glib::Object;

glib::wrapper! {
    pub struct SearchResultObject(ObjectSubclass<imp::SearchResultObject>);
}

impl SearchResultObject {
    pub fn new(search_result: &BoxedSearchResult) -> Self {
        Object::new(&[("data", search_result)]).expect("Failed to create Application Object")
    }
}
