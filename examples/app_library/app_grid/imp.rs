use gtk4::subclass::prelude::*;
use gtk4::{gio, glib, GridView};
use once_cell::sync::OnceCell;

#[derive(Default)]
pub struct AppGrid {
    pub app_grid_view: OnceCell<GridView>,
    pub app_model: OnceCell<gio::ListStore>,
    pub app_sort_model: OnceCell<gtk4::SortListModel>,
    pub search_filter_model: OnceCell<gtk4::FilterListModel>,
    pub group_filter_model: OnceCell<gtk4::FilterListModel>,
}

#[glib::object_subclass]
impl ObjectSubclass for AppGrid {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "AppGrid";
    type Type = super::AppGrid;
    type ParentType = gtk4::Box;
}

impl ObjectImpl for AppGrid {}

impl WidgetImpl for AppGrid {}

impl BoxImpl for AppGrid {}
