mod imp;
use gtk4 as gtk;

use crate::application_row::ApplicationRow;
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
        imp.model.get().expect("Could not get model")
    }

    fn setup_model(&self) {
        // Create new model
        let model = gio::ListStore::new(gio::AppInfo::static_type());
        gio::AppInfo::all().iter().for_each(|app_info| {
            model.append(app_info);
        });

        // Get state and set model
        let imp = imp::Window::from_instance(self);
        imp.model.set(model.clone()).expect("Could not set model");

        // A sorter used to sort AppInfo in the model by their name
        let sorter = gtk::CustomSorter::new(move |obj1, obj2| {
            let app_info1 = obj1.downcast_ref::<gio::AppInfo>().unwrap();
            let app_info2 = obj2.downcast_ref::<gio::AppInfo>().unwrap();

            app_info1
                .name()
                .to_lowercase()
                .cmp(&app_info2.name().to_lowercase())
                .into()
        });
        let filter = gtk::CustomFilter::new(|_obj| true);
        let filter_model = gtk::FilterListModel::new(Some(&model), Some(filter).as_ref());
        let sorted_model = gtk::SortListModel::new(Some(&filter_model), Some(&sorter));
        let slice_model = gtk::SliceListModel::new(Some(&sorted_model), 0, 9);
        let selection_model = gtk::SingleSelection::new(Some(&slice_model));

        // Wrap model with selection and pass it to the list view
        imp.list_view.set_model(Some(&selection_model));
    }

    fn setup_callbacks(&self) {
        // Get state
        let imp = imp::Window::from_instance(self);
        let window = self.clone().upcast::<gtk::Window>();
        let list_view = &imp.list_view;
        let sorted_model = list_view
            .model()
            .expect("List view missing selection model")
            .downcast::<gtk::SingleSelection>()
            .expect("could not downcast listview model to single selection model")
            .model()
            .downcast::<gtk::SliceListModel>()
            .expect("could not downcast single selection model to slice list model.")
            .model()
            .expect("sorted list model is missing from slice list model")
            .downcast::<gtk::SortListModel>()
            .expect("sorted list model could not be downcast");
        let filter_model = sorted_model
            .model()
            .expect("missing model for sort list model.")
            .downcast::<gtk::FilterListModel>()
            .expect("could not downcast sort list model to filter list model");

        let entry = &imp.entry;
        let lv = list_view.get();
        for i in 1..10 {
            let action_launchi = gio::SimpleAction::new(&format!("launch{}", i), None);
            self.add_action(&action_launchi);
            let context = list_view.display().app_launch_context().clone();
            let parent_window = list_view.root().unwrap().downcast::<gtk::Window>().unwrap();
            action_launchi.connect_activate(glib::clone!(@weak lv =>  move |_action, _parameter| {
                let model = lv.model().unwrap();
                let app_info = model.item(i - 1);
                if app_info.is_none() {
                    println!("oops no app for this row...");
                    return;
                }
                let app_info = app_info.unwrap().downcast::<gio::AppInfo>().unwrap();
                if let Err(err) = app_info.launch(&[], Some(&context)) {

                    gtk::MessageDialog::builder()
                        .text(&format!("Failed to start {}", app_info.name()))
                        .secondary_text(&err.to_string())
                        .message_type(gtk::MessageType::Error)
                        .modal(true)
                        .transient_for(&parent_window)
                        .build()
                        .show();

                    println!("oops launch failed")
                }
                println!("{}", i-1);
            }));
        }

        // Launch the application when an item of the list is activated
        list_view.connect_activate(move |list_view, position| {
            let model = list_view.model().unwrap();
            let app_info = model
                .item(position)
                .unwrap()
                .downcast::<gio::AppInfo>()
                .unwrap();

            let context = list_view.display().app_launch_context();
            if let Err(err) = app_info.launch(&[], Some(&context)) {
                let parent_window = list_view.root().unwrap().downcast::<gtk::Window>().unwrap();

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

        entry.connect_changed(
            glib::clone!(@weak filter_model, @weak sorted_model => move |search: &gtk::Entry| {
                let search_text = search.text().to_string().to_lowercase();
                let new_filter: gtk::CustomFilter = gtk::CustomFilter::new(move |obj| {
                    let search_res = obj.downcast_ref::<gio::AppInfo>()
                        .expect("The Object needs to be of type AppInfo");
                    search_res.name().to_string().to_lowercase().contains(&search_text)
                });
                let search_text = search.text().to_string().to_lowercase();
                let new_sorter: gtk::CustomSorter = gtk::CustomSorter::new(move |obj1, obj2| {
                    let app_info1 = obj1.downcast_ref::<gio::AppInfo>().unwrap();
                    let app_info2 = obj2.downcast_ref::<gio::AppInfo>().unwrap();
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

                filter_model.set_filter(Some(new_filter).as_ref());
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
        let factory = SignalListItemFactory::new();
        factory.connect_setup(move |_factory, item| {
            let row = ApplicationRow::new();
            item.set_child(Some(&row));
        });

        // the bind stage is used for "binding" the data to the created widgets on the "setup" stage
        factory.connect_bind(move |_factory, list_item| {
            let app_info = list_item
                .item()
                .unwrap()
                .downcast::<gio::AppInfo>()
                .unwrap();

            let child = list_item
                .child()
                .unwrap()
                .downcast::<ApplicationRow>()
                .unwrap();
            child.set_app_info(&app_info);
            if list_item.position() < 9 {
                child.set_shortcut(list_item.position() + 1);
            }
        });
        // Set the factory of the list view
        let imp = imp::Window::from_instance(self);
        imp.list_view.set_factory(Some(&factory));
    }
}
