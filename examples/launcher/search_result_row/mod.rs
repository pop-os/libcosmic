use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;

use crate::BoxedSearchResult;
use crate::icon_source;
use crate::SearchResultObject;

mod imp;

glib::wrapper! {
    pub struct SearchResultRow(ObjectSubclass<imp::SearchResultRow>)
        @extends gtk4::Widget, gtk4::Box;
}

impl Default for SearchResultRow {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchResultRow {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create SearchResultRow")
    }

    pub fn set_search_result(&self, search_obj: SearchResultObject) {
        let self_ = imp::SearchResultRow::from_instance(self);
        if let Ok(search_result) = search_obj.property("data") {
            if let Ok(search_result) = search_result.get::<BoxedSearchResult>() {
                if let Some(search_result) = search_result.0 {
                    self_.name.set_text(&search_result.name);
                    self_.description.set_text(&search_result.description);
                    icon_source(&self_.image, &search_result.icon);
                    icon_source(&self_.categoryimage, &search_result.category_icon);
                }
            }
        }
    }

    pub fn set_shortcut(&self, indx: u32) {
        let self_ = imp::SearchResultRow::from_instance(self);
        self_.shortcut.set_text(&format!("Ctrl + {}", indx));
    }
}
