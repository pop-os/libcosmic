use cascade::cascade;
use glib::Object;
use glib::{FromVariant, Variant};
use gtk4::subclass::prelude::*;
use gtk4::{
    gio, glib, Dialog, Entry, GridView, Label, PolicyType, ScrolledWindow, SignalListItemFactory,
    Window,
};
use gtk4::{prelude::*, CustomFilter};
use std::fs::File;

use crate::app_group::AppGroup;
use crate::app_group::AppGroupData;
use crate::grid_item::GridItem;
use crate::utils::data_path;
use crate::utils::set_group_scroll_policy;

mod imp;

glib::wrapper! {
    pub struct GroupGrid(ObjectSubclass<imp::GroupGrid>)
        @extends gtk4::Widget, gtk4::Box,
    @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl Default for GroupGrid {
    fn default() -> Self {
        Self::new()
    }
}

impl GroupGrid {
    pub fn new() -> Self {
        let self_: Self = glib::Object::new(&[]).expect("Failed to create GroupGrid");
        let imp = imp::GroupGrid::from_instance(&self_);

        let group_window = cascade! {
            ScrolledWindow::new();
            ..set_hscrollbar_policy(PolicyType::Never);
            ..set_vscrollbar_policy(PolicyType::Never);
            ..set_propagate_natural_height(true);
            ..set_min_content_height(150);
            ..set_max_content_height(300);
            ..set_hexpand(true);
        };
        self_.append(&group_window);

        let group_grid_view = cascade! {
            GridView::default();
            ..set_min_columns(8);
            ..set_max_columns(8);
        };
        group_window.set_child(Some(&group_grid_view));

        imp.group_grid_view.set(group_grid_view).unwrap();
        imp.group_scroll_window.set(group_window).unwrap();

        // Setup
        // Setup
        self_.setup_model();
        self_.restore_data();
        self_.setup_callbacks();
        self_.setup_factory();

        self_
    }

    fn setup_model(&self) {
        let imp = imp::GroupGrid::from_instance(&self);
        let group_model = gio::ListStore::new(AppGroup::static_type());
        imp.group_model
            .set(group_model.clone())
            .expect("Could not set group model");
        vec![
            AppGroup::new(AppGroupData {
                id: 0,
                name: "Library Home".to_string(),
                icon: "user-home".to_string(),
                mutable: false,
                app_names: Vec::new(),
                category: "".to_string(),
            }),
            AppGroup::new(AppGroupData {
                id: 0,
                name: "System".to_string(),
                icon: "folder".to_string(),
                mutable: false,
                app_names: Vec::new(),
                category: "System".to_string(),
            }),
            AppGroup::new(AppGroupData {
                id: 0,
                name: "Utilities".to_string(),
                icon: "folder".to_string(),
                mutable: false,
                app_names: Vec::new(),
                category: "Utility".to_string(),
            }),
            // Example of group with app name
            // AppGroup::new(AppGroupData {
            //     id: 0,
            //     name: "Custom Web".to_string(),
            //     icon: "folder".to_string(),
            //     mutable: true,
            //     app_names: vec!["Firefox Web Browser".to_string()],
            //     category: "".to_string(),
            // }),
            AppGroup::new(AppGroupData {
                id: 0,
                name: "New Group".to_string(),
                icon: "folder-new".to_string(),
                mutable: true,
                app_names: vec![],
                category: "".to_string(),
            }),
        ]
        .iter()
        .for_each(|group| {
            group_model.append(group);
        });
        let group_selection = gtk4::SingleSelection::new(Some(&group_model));
        imp.group_grid_view
            .get()
            .unwrap()
            .set_model(Some(&group_selection));
    }

    fn group_model(&self) -> &gio::ListStore {
        // Get state
        let imp = imp::GroupGrid::from_instance(self);
        imp.group_model.get().expect("Could not get model")
    }

    fn setup_callbacks(&self) {
        let imp = imp::GroupGrid::from_instance(self);
        let group_grid_view = &imp.group_grid_view.get().unwrap();
        let group_selection_model = group_grid_view
            .model()
            .expect("List view missing selection model")
            .downcast::<gtk4::SingleSelection>()
            .expect("could not downcast listview model to single selection model");
        let scroll_window = &imp.group_scroll_window.get().unwrap();
        // dynamically set scroll method
        self.group_model().connect_items_changed(
            glib::clone!(@weak scroll_window => move |scroll_list_model, _i, _rmv_cnt, _add_cnt| {
                set_group_scroll_policy(&scroll_window, scroll_list_model.n_items());
            }),
        );

        let self_clone = self.clone();
        group_grid_view.connect_activate(move |group_grid_view, i| {
            // on activation change the group filter model to use the app names, and category
            let window = group_grid_view.root().unwrap().downcast::<Window>().unwrap();
            println!("grid view activated. {}", i);
            let group_model = group_grid_view.model().unwrap().downcast::<gtk4::SingleSelection>().unwrap()
                .model()
                .downcast::<gio::ListStore>()
                .expect("could not downcast app group view selection model to list store model");

           // if last item in the model, don't change filter, instead show dialog for adding new group!
            if i == group_model.n_items() - 1 {
                let dialog_entry = Entry::new();
                let label = Label::new(Some("Name"));
                label.set_justify(gtk4::Justification::Left);
                label.set_xalign(0.0);
                let vbox = gtk4::Box::builder()
                    .spacing(12)
                    .hexpand(true)
                    .orientation(gtk4::Orientation::Vertical)
                    .margin_top(12)
                    .margin_bottom(12)
                    .margin_end(12)
                    .margin_start(12)
                    .build();
                vbox.append(&label);
                vbox.append(&dialog_entry);

                let dialog = Dialog::builder()
                    .modal(true)
                    .resizable(false)
                    .use_header_bar(true.into())
                    .destroy_with_parent(true)
                    .transient_for(&window)
                    .title("New App Group")
                    .child(&vbox)
                    .build();
                let app = window
                    .application()
                    .expect("could not get application from window");

                dialog.set_application(Some(&app));
                dialog.add_buttons(&[
                    ("Apply", gtk4::ResponseType::Apply),
                    ("Cancel", gtk4::ResponseType::Cancel),
                ]);

               dialog.connect_response(
                    glib::clone!(@weak dialog_entry, @weak group_selection_model, @weak group_model => move |dialog, response_type| {
                        println!("dialog should be closing...");
                        let name = dialog_entry.text().to_string();
                        if response_type == gtk4::ResponseType::Apply && name != "" {
                            let new_app_group = AppGroup::new(AppGroupData {
                                id: 0,
                                name: name,
                                icon: "folder".to_string(),
                                mutable: true,
                                app_names: vec![],
                                category: "".to_string(),
                            });
                            group_model.insert(group_model.n_items() - 1, &new_app_group);
                            group_selection_model.set_selected(i - 1);
                        } else {
                            group_selection_model.set_selected(0);
                        }
                        dialog.emit_close();
                    }),
                );
                dialog.connect_is_active_notify(move |win| {
                    let app = win
                        .application()
                        .expect("could not get application from window");
                    let active_window = app
                        .active_window()
                        .expect("no active window available, closing app library.");
                    dbg!(&active_window);
                    if win == &active_window && !win.is_active() {
                        println!("no focus");
                        // close top level window
                        window.close();
                    }
                });
                dialog.show();
                return;
            };
            // update the application filter
            let app_info = group_model
                .item(i)
                .unwrap()
                .downcast::<AppGroup>()
                .unwrap();
            let category =
                app_info.property::<String>("category").to_lowercase();

            let app_names =
                    <Vec<String>>::from_variant(&app_info.property::<Variant>("appnames")).unwrap_or_default();
            dbg!(&app_names);
            let new_filter: gtk4::CustomFilter = gtk4::CustomFilter::new(move |obj| {
                let app = obj
                    .downcast_ref::<gio::DesktopAppInfo>()
                    .expect("The Object needs to be of type AppInfo");
                if app_names.len() > 0 {
                    return app_names.contains(&String::from(app.name().as_str()));
                }
                match app.categories() {
                    Some(categories) => categories.to_string().to_lowercase().contains(&category),
                    None => false,
                }
            });
            self_clone
                .emit_by_name::<CustomFilter>("group-changed", &[&new_filter]);
        });
    }

    fn setup_factory(&self) {
        let imp = imp::GroupGrid::from_instance(&self);
        let group_factory = SignalListItemFactory::new();
        group_factory.connect_setup(move |_factory, item| {
            let row = GridItem::new();
            item.set_child(Some(&row));
        });

        // the bind stage is used for "binding" the data to the created widgets on the "setup" stage
        group_factory.connect_bind(move |_factory, grid_item| {
            let group_info = grid_item.item().unwrap().downcast::<AppGroup>().unwrap();

            let child = grid_item.child().unwrap().downcast::<GridItem>().unwrap();
            child.set_group_info(group_info);
        });
        // Set the factory of the list view
        imp.group_grid_view
            .get()
            .unwrap()
            .set_factory(Some(&group_factory));
    }

    fn restore_data(&self) {
        if let Ok(file) = File::open(data_path()) {
            // Deserialize data from file to vector
            let backup_data: Vec<AppGroupData> =
                serde_json::from_reader(file).expect("Could not get backup data from json file.");

            let app_group_objects: Vec<Object> = backup_data
                .into_iter()
                .map(|data| AppGroup::new(data).upcast::<Object>())
                .collect();
            let scroll_window = &imp::GroupGrid::from_instance(self)
                .group_scroll_window
                .get()
                .unwrap();

            // Insert restored objects into model
            self.group_model().splice(3, 0, &app_group_objects);
            set_group_scroll_policy(&scroll_window, self.group_model().n_items());
        } else {
            println!("Backup file does not exist yet {:?}", data_path());
        }
    }
    pub fn store_data(&self) {
        let mut backup_data = Vec::new();
        let mut position = 3;
        while let Some(item) = self.group_model().item(position) {
            if position == self.group_model().n_items() - 1 {
                break;
            }
            // Get `AppGroup` from `glib::Object`
            let group_data = item
                .downcast_ref::<AppGroup>()
                .expect("The object needs to be of type `AppGroupData`.")
                .group_data();
            // Add data to vector and increase position
            backup_data.push(group_data);
            position += 1;
        }

        // Save state in file
        let file = File::create(data_path()).expect("Could not create json file.");
        serde_json::to_writer_pretty(file, &backup_data)
            .expect("Could not write data to json file");
    }
}
