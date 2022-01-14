use glib::subclass::Signal;
use gtk4::subclass::prelude::*;
use gtk4::{gio, glib, GridView, ScrolledWindow};
use gtk4::{prelude::*, CustomFilter};
use once_cell::sync::{Lazy, OnceCell};

#[derive(Default)]
pub struct GroupGrid {
    pub group_grid_view: OnceCell<GridView>,
    pub group_scroll_window: OnceCell<ScrolledWindow>,
    pub group_model: OnceCell<gio::ListStore>,
}

#[glib::object_subclass]
impl ObjectSubclass for GroupGrid {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "GroupGrid";
    type Type = super::GroupGrid;
    type ParentType = gtk4::Box;
}

impl ObjectImpl for GroupGrid {
    fn signals() -> &'static [Signal] {
        static SIGNALS: Lazy<Vec<Signal>> = Lazy::new(|| {
            vec![Signal::builder(
                // Signal name
                "group-changed",
                // Types of the values which will be sent to the signal handler
                &[CustomFilter::static_type().into()],
                // Type of the value the signal handler sends back
                <()>::static_type().into(),
            )
            .build()]
        });
        SIGNALS.as_ref()
    }
}

impl WidgetImpl for GroupGrid {}

impl BoxImpl for GroupGrid {}
