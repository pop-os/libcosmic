use glib::subclass::Signal;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::rc::Rc;

use crate::dock_popover::DockPopover;

#[derive(Debug, Default)]
pub struct DockItem {
    pub image: Rc<RefCell<Option<gtk4::Image>>>,
    pub dots: Rc<RefCell<gtk4::Label>>,
    pub item_box: Rc<RefCell<gtk4::Box>>,
    pub popover: Rc<RefCell<gtk4::Popover>>,
    pub popover_menu: Rc<RefCell<DockPopover>>,
}

#[glib::object_subclass]
impl ObjectSubclass for DockItem {
    const NAME: &'static str = "DockItem";
    type Type = super::DockItem;
    type ParentType = gtk4::Button;
}

impl ObjectImpl for DockItem {
    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder(
                // Signal name
                "popover-closed",
                // Types of the values which will be sent to the signal handler
                &[],
                // Type of the value the signal handler sends back
                <()>::static_type().into(),
            )
            .build()]
        });
        SIGNALS.as_ref()
    }
}

impl WidgetImpl for DockItem {}

impl ButtonImpl for DockItem {}
