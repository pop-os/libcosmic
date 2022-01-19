use std::rc::Rc;

use crate::app_grid::AppGrid;
use crate::group_grid::GroupGrid;
use cascade::cascade;
use gdk4_x11::X11Display;
use gdk4_x11::X11Surface;
use glib::Object;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::Align;
use gtk4::Application;
use gtk4::Box;
use gtk4::CustomFilter;
use gtk4::Orientation;
use gtk4::SearchEntry;
use gtk4::Separator;
use gtk4::{gio, glib};
use libcosmic::x;
use once_cell::sync::OnceCell;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;

mod imp;

glib::wrapper! {
    pub struct AppLibraryWindow(ObjectSubclass<imp::AppLibraryWindow>)
        @extends gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk4::Accessible, gtk4::Buildable,
                    gtk4::ConstraintTarget, gtk4::Native, gtk4::Root, gtk4::ShortcutManager;
}

impl AppLibraryWindow {
    pub fn new(app: &Application) -> Self {
        //quit shortcut
        app.set_accels_for_action("win.quit", &["<primary>W", "Escape"]);
        //launch shortcuts
        for i in 1..10 {
            app.set_accels_for_action(&format!("win.launch{}", i), &[&format!("<primary>{}", i)]);
        }
        let self_: Self =
            Object::new(&[("application", app)]).expect("Failed to create `AppLibraryWindow`.");
        let imp = imp::AppLibraryWindow::from_instance(&self_);

        cascade! {
            &self_;
            ..set_width_request(1200);
            ..set_title(Some("Cosmic App Library"));
            ..set_decorated(false);
            ..add_css_class("root_window");
        };

        let app_library = cascade! {
            Box::new(Orientation::Vertical, 0);
            ..add_css_class("app_library_container");
        };
        self_.set_child(Some(&app_library));

        let entry = cascade! {
            SearchEntry::new();
            ..set_width_request(300);
            ..set_halign(Align::Center);
            ..set_margin_top(12);
            ..set_margin_bottom(12);
            ..set_placeholder_text(Some(" Type to search"));
        };
        app_library.append(&entry);

        let app_grid = AppGrid::new();
        app_library.append(&app_grid);

        let separator = cascade! {
            Separator::new(Orientation::Horizontal);
            ..set_hexpand(true);
            ..set_margin_bottom(12);
            ..set_margin_top(12);
        };
        app_library.append(&separator);

        let group_grid = GroupGrid::new();
        app_library.append(&group_grid);

        imp.entry.set(entry).unwrap();
        imp.app_grid.set(app_grid).unwrap();
        imp.group_grid.set(group_grid).unwrap();

        Self::setup_callbacks(&self_);
        self_
    }

    fn setup_callbacks(&self) {
        // Get state
        let imp = imp::AppLibraryWindow::from_instance(self);
        let window = self.clone().upcast::<gtk4::Window>();
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

        window.connect_realize(move |window| {
            if let Some((display, surface)) = x::get_window_x11(window) {
                let (conn, _screen_num) = x11rb::connect(None).expect("Failed to connect to X");
                let x11rb_conn = Rc::new(OnceCell::new());
                x11rb_conn.set(conn).unwrap();

                // ignore all x11 errors...
                let xdisplay = display
                    .clone()
                    .downcast::<X11Display>()
                    .expect("Failed to downgrade X11 Display.");
                xdisplay.error_trap_push();
                unsafe {
                    x::change_property(
                        &display,
                        &surface,
                        "_NET_WM_WINDOW_TYPE",
                        x::PropMode::Replace,
                        &[x::Atom::new(&display, "_NET_WM_WINDOW_TYPE_DIALOG").unwrap()],
                    );
                }
                let resize = glib::clone!(@weak window, @strong x11rb_conn => move || {
                    let s = window.surface();
                    let height = window.height();
                    let width = window.width();

                    if let Some((display, _surface)) = x::get_window_x11(&window) {
                        let geom = display
                            .primary_monitor().geometry();
                        let monitor_x = geom.x();
                        let monitor_y = geom.y();
                        let monitor_width = geom.width();
                        let monitor_height = geom.height();
                        // dbg!(monitor_width);
                        // dbg!(monitor_height);
                        // dbg!(width);
                        // dbg!(height);
                        let w_conf = ConfigureWindowAux::default()
                            .x(monitor_x + monitor_width / 2 - width / 2)
                            .y(monitor_y + monitor_height / 2 - height / 2);

                        let x11surface = gdk4_x11::X11Surface::xid(
                            &s.clone().downcast::<X11Surface>()
                                .expect("Failed to downcast Surface to X11Surface"),
                        );
                        let conn = x11rb_conn.get().unwrap();
                        conn.configure_window(
                            x11surface.try_into().expect("Failed to convert XID"),
                            &w_conf,
                        )
                            .expect("failed to configure window...");
                        conn.flush().expect("failed to flush");
                    }
                });
                let s = window.surface();
                let resize_height = resize.clone();
                s.connect_height_notify(move |_s| {
                    glib::source::idle_add_local_once(resize_height.clone());
                });
                let resize_width = resize.clone();
                s.connect_width_notify(move |_s| {
                    glib::source::idle_add_local_once(resize_width.clone());
                });
                s.connect_scale_factor_notify(move |_s| {
                    glib::source::idle_add_local_once(resize.clone());
                });
            } else {
                println!("failed to get X11 window");
            }
        });

        let action_quit = gio::SimpleAction::new("quit", None);
        action_quit.connect_activate(glib::clone!(@weak window => move |_, _| {
            window.close();
        }));
        self.add_action(&action_quit);
        window.connect_is_active_notify(move |win| {
            let app = win
                .application()
                .expect("could not get application from window");
            let active_window = app
                .active_window()
                .expect("no active window available, closing app library.");
            dbg!(&active_window);
            if win == &active_window && !win.is_active() {
                win.close();
            }
        });
    }
}
