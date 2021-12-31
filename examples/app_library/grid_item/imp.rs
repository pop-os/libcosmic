use std::cell::Cell;
use std::cell::RefCell;
use std::rc::Rc;

use glib;
use gtk4::subclass::prelude::*;

#[derive(Debug, Default)]
pub struct GridItem {
    pub name: Rc<RefCell<gtk4::Label>>,
    pub image: Rc<RefCell<gtk4::Image>>,
    pub index: Cell<u32>,
}

#[glib::object_subclass]
impl ObjectSubclass for GridItem {
    const NAME: &'static str = "GridItem";
    type Type = super::GridItem;
    type ParentType = gtk4::Box;
}

impl ObjectImpl for GridItem {}

impl WidgetImpl for GridItem {}

impl BoxImpl for GridItem {}
