use gtk4::glib;
use gtk4::subclass::prelude::*;
use gtk4::SearchEntry;
use once_cell::sync::OnceCell;

use crate::app_grid::AppGrid;
use crate::group_grid::GroupGrid;

#[derive(Default)]
pub struct AppLibraryWindowInner {
    pub entry: OnceCell<SearchEntry>,
    pub app_grid: OnceCell<AppGrid>,
    pub group_grid: OnceCell<GroupGrid>,
}

#[glib::object_subclass]
impl ObjectSubclass for AppLibraryWindowInner {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "AppLibraryWindowInner";
    type Type = super::AppLibraryWindowInner;
    type ParentType = gtk4::Box;
}

impl ObjectImpl for AppLibraryWindowInner {}

impl WidgetImpl for AppLibraryWindowInner {}

impl BoxImpl for AppLibraryWindowInner {}
