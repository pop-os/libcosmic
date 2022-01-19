use cascade::cascade;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::{gio, glib, GridView, PolicyType, ScrolledWindow, SignalListItemFactory};

use crate::grid_item::GridItem;

mod imp;

glib::wrapper! {
    pub struct AppGrid(ObjectSubclass<imp::AppGrid>)
        @extends gtk4::Widget, gtk4::Box,
    @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl Default for AppGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl AppGrid {
    pub fn new() -> Self {
        let self_: Self = glib::Object::new(&[]).expect("Failed to create AppGrid");
        let imp = imp::AppGrid::from_instance(&self_);

        let library_window = cascade! {
            ScrolledWindow::new();
            ..set_hscrollbar_policy(PolicyType::Never);
            ..set_min_content_height(520);
            ..set_hexpand(true);
            ..set_margin_top(12);
        };
        self_.append(&library_window);

        let library_grid = cascade! {
            GridView::default();
            ..set_min_columns(7);
            ..set_max_columns(7);
            ..set_single_click_activate(true);
        };
        library_window.set_child(Some(&library_grid));

        imp.app_grid_view.set(library_grid).unwrap();

        // Setup
        self_.setup_model();
        self_.setup_callbacks();
        self_.setup_factory();

        self_
    }

    fn setup_model(&self) {
        // Create new model
        let app_model = gio::ListStore::new(gio::DesktopAppInfo::static_type());
        // Get state and set model
        let imp = imp::AppGrid::from_instance(self);

        // A sorter used to sort AppInfo in the model by their name
        xdg::BaseDirectories::new()
            .expect("could not access XDG Base directory")
            .get_data_dirs()
            .iter_mut()
            .for_each(|xdg_data_path| {
                xdg_data_path.push("applications");
                dbg!(&xdg_data_path);
                if let Ok(dir_iter) = std::fs::read_dir(xdg_data_path) {
                    dir_iter.for_each(|dir_entry| {
                        if let Ok(dir_entry) = dir_entry {
                            if let Some(path) = dir_entry.path().file_name() {
                                if let Some(path) = path.to_str() {
                                    if let Some(app_info) = gio::DesktopAppInfo::new(path) {
                                        if app_info.should_show() {
                                            app_model.append(&app_info)
                                        } else {
                                            println!("Ignoring {}", path);
                                        }
                                    } else {
                                        println!("error loading {}", path);
                                    }
                                }
                            }
                        }
                    })
                }
            });

        let sorter = gtk4::CustomSorter::new(move |obj1, obj2| {
            let app_info1 = obj1.downcast_ref::<gio::DesktopAppInfo>().unwrap();
            let app_info2 = obj2.downcast_ref::<gio::DesktopAppInfo>().unwrap();

            app_info1
                .name()
                .to_lowercase()
                .cmp(&app_info2.name().to_lowercase())
                .into()
        });
        let filter = gtk4::CustomFilter::new(|_obj| true);

        let search_filter_model =
            gtk4::FilterListModel::new(Some(&app_model), Some(filter).as_ref());
        let filter = gtk4::CustomFilter::new(|_obj| true);
        let group_filter_model =
            gtk4::FilterListModel::new(Some(&search_filter_model), Some(filter).as_ref());
        let sorted_model = gtk4::SortListModel::new(Some(&group_filter_model), Some(&sorter));

        let selection_model = gtk4::SingleSelection::builder()
            .model(&sorted_model)
            .autoselect(false)
            .can_unselect(true)
            .selected(gtk4::INVALID_LIST_POSITION)
            .build();

        // Wrap model with selection and pass it to the list view
        imp.app_model
            .set(app_model.clone())
            .expect("Could not set model");
        imp.app_sort_model.set(sorted_model).unwrap();
        imp.search_filter_model.set(search_filter_model).unwrap();
        imp.group_filter_model.set(group_filter_model).unwrap();
        imp.app_grid_view
            .get()
            .unwrap()
            .set_model(Some(&selection_model));
        selection_model.unselect_all();
    }

    fn setup_callbacks(&self) {
        let imp = imp::AppGrid::from_instance(self);
        let app_grid_view = &imp.app_grid_view.get().unwrap();

        app_grid_view.connect_activate(move |list_view, i| {
            // on activation change the group filter model to use the app names, and category
            println!("selected app {}", i);
            // Launch the application when an item of the list is activated
            let model = list_view.model().unwrap();
            if let Some(item) = model.item(i) {
                let app_info = item.downcast::<gio::DesktopAppInfo>().unwrap();
                let context = list_view.display().app_launch_context();
                if let Err(err) = app_info.launch(&[], Some(&context)) {
                    gtk4::MessageDialog::builder()
                        .text(&format!("Failed to start {}", app_info.name()))
                        .secondary_text(&err.to_string())
                        .message_type(gtk4::MessageType::Error)
                        .modal(true)
                        .build()
                        .show();
                }
            }
        });
    }

    fn setup_factory(&self) {
        let app_factory = SignalListItemFactory::new();
        app_factory.connect_setup(move |_factory, item| {
            let row = GridItem::new();
            item.set_child(Some(&row));
        });

        let imp = imp::AppGrid::from_instance(self);
        // the bind stage is used for "binding" the data to the created widgets on the "setup" stage
        let app_grid_view = &imp.app_grid_view.get().unwrap();
        app_factory.connect_bind(
            glib::clone!(@weak app_grid_view => move |_factory, grid_item| {
                let app_info = grid_item
                    .item()
                    .unwrap()
                    .downcast::<gio::DesktopAppInfo>()
                    .unwrap();
                let child = grid_item.child().unwrap().downcast::<GridItem>().unwrap();
                child.set_app_info(&app_info);
            }),
        );
        // Set the factory of the list view
        app_grid_view.set_factory(Some(&app_factory));
    }

    pub fn set_app_sorter(&self, sorter: &gtk4::CustomSorter) {
        let imp = imp::AppGrid::from_instance(&self);
        let sort_model = imp.app_sort_model.get().unwrap();
        sort_model.set_sorter(Some(sorter));
    }

    pub fn set_search_filter(&self, filter: &gtk4::CustomFilter) {
        let imp = imp::AppGrid::from_instance(&self);
        let filter_model = imp.search_filter_model.get().unwrap();
        filter_model.set_filter(Some(filter));
    }

    pub fn set_group_filter(&self, filter: &gtk4::CustomFilter) {
        let imp = imp::AppGrid::from_instance(&self);
        let filter_model = imp.group_filter_model.get().unwrap();
        filter_model.set_filter(Some(filter));
    }
}
