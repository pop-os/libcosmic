mod imp;
use crate::utils::BoxedSearchResult;
use gdk4::glib::Object;

glib::wrapper! {
    pub struct ApplicationObject(ObjectSubclass<imp::ApplicationObject>);
}

impl ApplicationObject {
    pub fn new(search_result: &BoxedSearchResult) -> Self {
        Object::new(&[("data", search_result)]).expect("Failed to create Application Object")
    }
}
