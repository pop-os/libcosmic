use cascade::cascade;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gio, glib, Align, CustomFilter, Orientation, SearchEntry, Separator};

use crate::app_grid::AppGrid;
use crate::group_grid::GroupGrid;

mod imp;

glib::wrapper! {
    pub struct AppLibraryWindowInner(ObjectSubclass<imp::AppLibraryWindowInner>)
        @extends gtk4::Widget, gtk4::Box,
    @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl Default for AppLibraryWindowInner {
    fn default() -> Self {
        Self::new()
    }
}

impl AppLibraryWindowInner {
    pub fn new() -> Self {
        let self_: Self = glib::Object::new(&[]).expect("Failed to create AppLibraryWindowInner");
        let imp = imp::AppLibraryWindowInner::from_instance(&self_);

        cascade! {
            &self_;
            ..set_orientation(Orientation::Vertical);
            ..add_css_class("app_library_container");
        };

        let entry = cascade! {
            SearchEntry::new();
            ..set_width_request(300);
            ..set_halign(Align::Center);
            ..set_margin_top(12);
            ..set_margin_bottom(12);
            ..set_placeholder_text(Some(" Type to search"));
        };
        self_.append(&entry);

        let app_grid = AppGrid::new();
        self_.append(&app_grid);

        let separator = cascade! {
            Separator::new(Orientation::Horizontal);
            ..set_hexpand(true);
            ..set_margin_bottom(12);
            ..set_margin_top(12);
        };
        self_.append(&separator);

        let group_grid = GroupGrid::new();
        self_.append(&group_grid);

        imp.entry.set(entry).unwrap();
        imp.app_grid.set(app_grid).unwrap();
        imp.group_grid.set(group_grid).unwrap();

        Self::setup_callbacks(&self_);

        self_
    }

    pub fn group_grid(&self) -> Option<&GroupGrid> {
        let imp = imp::AppLibraryWindowInner::from_instance(self);
        imp.group_grid.get()
    }

    fn setup_callbacks(&self) {
        // Get state
        let imp = imp::AppLibraryWindowInner::from_instance(self);
        let app_grid = &imp.app_grid.get().unwrap();
        let group_grid = &imp.group_grid.get().unwrap();

        let entry = &imp.entry.get().unwrap();

        group_grid.connect_local(
            "group-changed",
            false,
            glib::clone!(@weak app_grid => @default-return None, move |args| {
                let new_filter = args[1].get::<CustomFilter>().unwrap();
                app_grid.set_group_filter(&new_filter);
                None
            }),
        );

        entry.connect_changed(
            glib::clone!(@weak app_grid => move |search: &gtk4::SearchEntry| {
                let search_text = search.text().to_string().to_lowercase();
                let new_filter: gtk4::CustomFilter = gtk4::CustomFilter::new(move |obj| {
                    let search_res = obj.downcast_ref::<gio::DesktopAppInfo>()
                        .expect("The Object needs to be of type AppInfo");
                    search_res.name().to_string().to_lowercase().contains(&search_text)
                });
                let search_text = search.text().to_string().to_lowercase();
                let new_sorter: gtk4::CustomSorter = gtk4::CustomSorter::new(move |obj1, obj2| {
                    let app_info1 = obj1.downcast_ref::<gio::DesktopAppInfo>().unwrap();
                    let app_info2 = obj2.downcast_ref::<gio::DesktopAppInfo>().unwrap();
                    if search_text == "" {
                        return app_info1
                            .name()
                            .to_lowercase()
                            .cmp(&app_info2.name().to_lowercase())
                            .into();
                    }

                    let i_1 = app_info1.name().to_lowercase().find(&search_text);
                    let i_2 = app_info2.name().to_lowercase().find(&search_text);
                    match (i_1, i_2) {
                        (Some(i_1), Some(i_2)) => i_1.cmp(&i_2).into(),
                        (Some(_), None) => std::cmp::Ordering::Less.into(),
                        (None, Some(_)) => std::cmp::Ordering::Greater.into(),
                        _ => app_info1
                            .name()
                            .to_lowercase()
                            .cmp(&app_info2.name().to_lowercase())
                            .into()
                    }
                });
                app_grid.set_search_filter(&new_filter);
                app_grid.set_app_sorter(&new_sorter);
            }),
        );
    }
}
