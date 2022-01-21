use crate::window_inner::AppLibraryWindowInner;
use cascade::cascade;
use gdk4::subclass::prelude::ObjectSubclassExt;
use gdk4_x11::X11Display;
use glib::Object;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4::{gdk, gio, glib};
use libcosmic::x;

pub fn create(app: &Application, monitor: gdk::Monitor) {
    //quit shortcut
    app.set_accels_for_action("app.quit", &["<primary>W", "Escape"]);
    setup_shortcuts(app);

    #[cfg(feature = "layer-shell")]
    if let Some(wayland_monitor) = monitor.downcast_ref() {
        wayland_create(&app, wayland_monitor);
        return;
    }

    cascade! {
        AppLibraryWindow::new(&app);
        ..show();
    };
}

fn setup_shortcuts(app: &Application) {
    let action_quit = gio::SimpleAction::new("quit", None);
    action_quit.connect_activate(glib::clone!(@weak app => move |_, _| {
        app.quit();
    }));
    app.add_action(&action_quit);
}

#[cfg(feature = "layer-shell")]
fn wayland_create(app: &Application, monitor: &gdk4_wayland::WaylandMonitor) {
    use libcosmic::wayland::{Anchor, KeyboardInteractivity, Layer, LayerShellWindow};

    let window = cascade! {
        LayerShellWindow::new(Some(monitor), Layer::Top, "");
        ..set_width_request(800);
        ..set_height_request(600);
        // ..set_title(Some("Cosmic App Library"));
        // ..set_decorated(false);
        ..set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        ..add_css_class("root_window");
        ..set_anchor(Anchor::empty());
        ..show();
    };

    let app_library = AppLibraryWindowInner::new();
    window.set_child(Some(&app_library));
    dbg!(&window);
    window.connect_is_active_notify(glib::clone!(@weak app => move |w| {
        if !w.is_active() {
            app.quit();
        }
    }));
    window.show();

    // setup_shortcuts(window.clone().upcast::<gtk4::ApplicationWindow>());
    // XXX
    unsafe { window.set_data("cosmic-app-hold", app.hold()) };
}

mod imp;

glib::wrapper! {
    pub struct AppLibraryWindow(ObjectSubclass<imp::AppLibraryWindow>)
        @extends gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk4::Accessible, gtk4::Buildable,
                    gtk4::ConstraintTarget, gtk4::Native, gtk4::Root, gtk4::ShortcutManager;
}

impl AppLibraryWindow {
    pub fn new(app: &Application) -> Self {
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
        let app_library = AppLibraryWindowInner::new();
        self_.set_child(Some(&app_library));
        imp.inner.set(app_library).unwrap();

        Self::setup_callbacks(&self_);

        self_
    }

    fn setup_callbacks(&self) {
        // Get state
        let window = self.clone().upcast::<gtk4::Window>();

        window.connect_realize(move |window| {
            println!("gtk window setup");
            if let Some((display, surface)) = x::get_window_x11(window) {
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
                let resize = glib::clone!(@weak window => move || {
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
                        unsafe { x::set_position(&display, &surface,
                            monitor_x + monitor_width / 2 - width / 2,
                                                 monitor_y + monitor_height / 2 - height / 2)};
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

        let imp = imp::AppLibraryWindow::from_instance(&self);
        let inner = imp.inner.get().unwrap();
        window.connect_is_active_notify(glib::clone!(@weak inner => move |win| {
            let app = win
                .application()
                .expect("could not get application from window");
            let active_window = app
                .active_window()
                .expect("no active window available, closing app library.");
            if win == &active_window && !win.is_active() && !inner.is_popup_active() {
                win.close();
            }
        }));
    }
}
