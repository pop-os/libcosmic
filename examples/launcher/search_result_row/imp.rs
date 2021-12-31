use gtk4::glib;
use gtk4::subclass::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct SearchResultRow {
    pub name: Rc<RefCell<gtk4::Label>>,
    pub description: Rc<RefCell<gtk4::Label>>,
    pub shortcut: Rc<RefCell<gtk4::Label>>,
    pub image: Rc<RefCell<gtk4::Image>>,
    pub category_image: Rc<RefCell<gtk4::Image>>,
}

#[glib::object_subclass]
impl ObjectSubclass for SearchResultRow {
    const NAME: &'static str = "SearchResultRow";
    type Type = super::SearchResultRow;
    type ParentType = gtk4::Box;
}

impl ObjectImpl for SearchResultRow {}

impl WidgetImpl for SearchResultRow {}

impl BoxImpl for SearchResultRow {}
