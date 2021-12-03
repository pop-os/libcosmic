mod imp;
use crate::app_group::AppGroup;
use crate::app_group::AppGroupData;
use glib::FromVariant;
use glib::Variant;
use gtk4 as gtk;
use gtk4::Button;
use std::path::Path;

use crate::grid_item::GridItem;
use glib::Object;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk::{Application, SignalListItemFactory};

use libcosmic::x;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        //quit shortcut
        app.set_accels_for_action("win.quit", &["<primary>W", "Escape"]);
        //launch shortcuts
        for i in 1..10 {
            app.set_accels_for_action(&format!("win.launch{}", i), &[&format!("<primary>{}", i)]);
        }
        Object::new(&[("application", app)]).expect("Failed to create `Window`.")
    }

    fn model(&self) -> &gio::ListStore {
        // Get state
        let imp = imp::Window::from_instance(self);
        imp.app_model.get().expect("Could not get model")
    }

    fn setup_model(&self) {
        // Create new model
        let app_model = gio::ListStore::new(gio::DesktopAppInfo::static_type());
        // Get state and set model
        let imp = imp::Window::from_instance(self);
        imp.app_model
            .set(app_model.clone())
            .expect("Could not set model");

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

        let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
            let app_info1 = obj1.downcast_ref::<gio::DesktopAppInfo>().unwrap();
            let app_info2 = obj2.downcast_ref::<gio::DesktopAppInfo>().unwrap();

            app_info1
                .name()
                .to_lowercase()
                .cmp(&app_info2.name().to_lowercase())
                .into()
        });
        let filter = gtk::CustomFilter::new(|_obj| true);
        let search_filter_model =
            gtk::FilterListModel::new(Some(&app_model), Some(filter).as_ref());
        let filter = gtk::CustomFilter::new(|_obj| true);
        let group_filter_model =
            gtk::FilterListModel::new(Some(&search_filter_model), Some(filter).as_ref());
        // TODO add app group filter model
        let sorted_model = gtk::SortListModel::new(Some(&group_filter_model), Some(&sorter));
        let selection_model = gtk::SingleSelection::new(Some(&sorted_model));

        // Wrap model with selection and pass it to the list view
        imp.app_grid_view.set_model(Some(&selection_model));

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
            AppGroup::new(AppGroupData {
                id: 0,
                name: "Web".to_string(),
                icon: "folder".to_string(),
                mutable: true,
                app_names: vec!["Firefox Web Browser".to_string()],
                category: "".to_string(),
            }),
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
        imp.group_grid_view.set_model(Some(&group_selection));
    }

    fn setup_callbacks(&self) {
        // Get state
        let imp = imp::Window::from_instance(self);
        let window = self.clone().upcast::<gtk::Window>();
        let app_grid_view = &imp.app_grid_view;
        let group_grid_view = &imp.group_grid_view;
        let sorted_model = app_grid_view
            .model()
            .expect("List view missing selection model")
            .downcast::<gtk::SingleSelection>()
            .expect("could not downcast listview model to single selection model")
            .model()
            .downcast::<gtk::SortListModel>()
            .expect("sorted list model could not be downcast");
        let group_filter_model = sorted_model
            .model()
            .expect("missing model for sort list model.")
            .downcast::<gtk::FilterListModel>()
            .expect("could not downcast sort list model to filter list model");
        let app_filter_model = group_filter_model
            .model()
            .expect("missing model for sort list model.")
            .downcast::<gtk::FilterListModel>()
            .expect("could not downcast sort list model to filter list model");

        let entry = &imp.entry;

        // Launch the application when an item of the list is activated
        app_grid_view.connect_activate(move |grid_view, position| {
            let model = grid_view.model().unwrap();
            let app_info = model
                .item(position)
                .unwrap()
                .downcast::<gio::DesktopAppInfo>()
                .unwrap();

            let context = grid_view.display().app_launch_context();
            if let Err(err) = app_info.launch(&[], Some(&context)) {
                let parent_window = grid_view.root().unwrap().downcast::<gtk::Window>().unwrap();

                gtk::MessageDialog::builder()
                    .text(&format!("Failed to start {}", app_info.name()))
                    .secondary_text(&err.to_string())
                    .message_type(gtk::MessageType::Error)
                    .modal(true)
                    .transient_for(&parent_window)
                    .build()
                    .show();
            }
        });

        // on activation change the group filter model to use the app names, and category
        group_grid_view.connect_activate(glib::clone!(@weak app_filter_model => move |grid_view, position| {
            // if last item in the model, don't change filter, instead show dialog for adding new group!
            if position == grid_view.model().unwrap().n_items() - 1 {
                println!("TODO: launch action to show the Add/Edit Group Window");
                return;
            };
            // update the application filter
            let model = grid_view.model().unwrap();
            let app_info = model
                .item(position)
                .unwrap()
                .downcast::<AppGroup>()
                .unwrap();
            let category =
            if let Ok(category_prop) = app_info.property("category") {
                category_prop.get::<String>().unwrap_or("".to_string()).to_lowercase()
            } else {
                "".to_string()
            };

            let app_names =
                if let Ok(app_names_prop) = app_info.property("appnames") {
                    <Vec<String>>::from_variant(&app_names_prop.get::<Variant>().expect("appnames nneds to be a variant.")).unwrap_or_default()
            } else {
                vec![]
            };
            dbg!(&app_names);
            let new_filter: gtk::CustomFilter = gtk::CustomFilter::new(move |obj| {
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
            group_filter_model.set_filter(Some(new_filter).as_ref());
        }));
        // can't listen to select signal on grid view, but this a good to know example...
        // if group_grid_view.connect_local("select", true, glib::clone!(@weak group_filter_model => @default-return None, move |args| {
        //     let grid_view = args[0].get::<gtk::GridView>().unwrap();
        //     let position = args[1].get::<u32>().unwrap();
        // // on activation change the group filter model to use the app names, and category
        //     let model = grid_view.model().unwrap();
        //     let app_info = model
        //         .item(position)
        //         .unwrap()
        //         .downcast::<AppGroup>()
        //         .unwrap();
        //     let category: String;
        //     if let Ok(category_prop) = app_info.property("category") {
        //         category = category_prop.get::<String>().unwrap_or("".to_string()).to_lowercase();
        //     } else {
        //         category = "".to_string();
        //     }
        //     let new_filter: gtk::CustomFilter = gtk::CustomFilter::new(move |obj| {
        //         let app = obj
        //             .downcast_ref::<gio::DesktopAppInfo>()
        //             .expect("The Object needs to be of type AppInfo");
        //         match app.categories() {
        //             Some(categories) => categories.to_string().to_lowercase().contains(&category),
        //             None => false,
        //         }
        //     });
        //     group_filter_model.set_filter(Some(new_filter).as_ref());
        //     None
        // })).is_err() { println!("Failed to connect to grid view select...") };

        entry.connect_changed(
            glib::clone!(@weak app_filter_model, @weak sorted_model => move |search: &gtk::SearchEntry| {
                let search_text = search.text().to_string().to_lowercase();
                let new_filter: gtk::CustomFilter = gtk::CustomFilter::new(move |obj| {
                    let search_res = obj.downcast_ref::<gio::DesktopAppInfo>()
                        .expect("The Object needs to be of type AppInfo");
                    search_res.name().to_string().to_lowercase().contains(&search_text)
                });
                let search_text = search.text().to_string().to_lowercase();
                let new_sorter: gtk::CustomSorter = gtk::CustomSorter::new(move |obj1, obj2| {
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

                app_filter_model.set_filter(Some(new_filter).as_ref());
                sorted_model.set_sorter(Some(new_sorter).as_ref());
            }),
        );

        window.connect_realize(move |window| {
            if let Some((display, surface)) = x::get_window_x11(window) {
                unsafe {
                    x::change_property(
                        &display,
                        &surface,
                        "_NET_WM_WINDOW_TYPE",
                        x::PropMode::Replace,
                        &[x::Atom::new(&display, "_NET_WM_WINDOW_TYPE_DIALOG").unwrap()],
                    );
                }
            } else {
                println!("failed to get X11 window");
            }
        });

        let action_quit = gio::SimpleAction::new("quit", None);
        action_quit.connect_activate(glib::clone!(@weak window => move |_, _| {
            window.close();
        }));
        self.add_action(&action_quit);

        window.connect_is_active_notify(|win| {
            if !win.is_active() {
                win.close();
            }
        });
    }

    fn setup_factory(&self) {
        let app_factory = SignalListItemFactory::new();
        app_factory.connect_setup(move |_factory, item| {
            let row = GridItem::new();
            item.set_child(Some(&row));
        });

        let imp = imp::Window::from_instance(self);
        // the bind stage is used for "binding" the data to the created widgets on the "setup" stage
        let app_grid_view = &imp.app_grid_view.get();
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
        imp.group_grid_view.set_factory(Some(&group_factory));
    }
}
