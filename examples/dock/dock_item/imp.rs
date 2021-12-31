use gtk4::glib;
use gtk4::subclass::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct DockItem {
    pub image: Rc<RefCell<gtk4::Image>>,
    pub dots: Rc<RefCell<gtk4::Label>>,
}

#[glib::object_subclass]
impl ObjectSubclass for DockItem {
    const NAME: &'static str = "DockItem";
    type Type = super::DockItem;
    type ParentType = gtk4::Button;
}

impl ObjectImpl for DockItem {}

impl WidgetImpl for DockItem {}

impl ButtonImpl for DockItem {}
