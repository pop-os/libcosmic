use cascade::cascade;
use gdk4_x11::X11Display;
use glib::Object;
use glib::Type;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::Align;
use gtk4::Application;
use gtk4::Box;
use gtk4::DropTarget;
use gtk4::EventControllerMotion;
use gtk4::Orientation;
use gtk4::Revealer;
use gtk4::RevealerTransitionType;
use gtk4::Separator;
use gtk4::{gio, glib};

use libcosmic::x;

use crate::dock_list::DockList;
use crate::dock_list::DockListType;

mod imp;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk4::ApplicationWindow, gtk4::Window, gtk4::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Native, gtk4::Root, gtk4::ShortcutManager;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        let self_: Self = Object::new(&[("application", app)]).expect("Failed to create `Window`.");
        let imp = imp::Window::from_instance(&self_);
        cascade! {
            &self_;
            ..set_height_request(100);
            ..set_width_request(128);
            ..set_title(Some("Cosmic Dock"));
            ..set_decorated(false);
            ..set_resizable(false);
            ..add_css_class("root_window");
        };
        let cursor_handle = Box::new(Orientation::Vertical, 0);
        self_.set_child(Some(&cursor_handle));

        let window_filler = cascade! {
            Box::new(Orientation::Vertical, 0);
            ..set_height_request(0); // shrinks to nothing when revealer is shown
            ..set_vexpand(true); // expands to fill window when revealer is hidden, preventingb window from changing size so much...
        };
        cursor_handle.append(&window_filler);

        let revealer = cascade! {
            Revealer::new();
            ..set_reveal_child(true);
            ..set_valign(Align::Baseline);
            ..set_transition_duration(150);
            ..set_transition_type(RevealerTransitionType::SwingUp);
        };
        cursor_handle.append(&revealer);

        let dock = cascade! {
            Box::new(Orientation::Horizontal, 4);
            ..set_margin_start(4);
            ..set_margin_end(4);
            ..set_margin_bottom(4);
        };
        dock.add_css_class("dock");
        revealer.set_child(Some(&dock));

        let saved_app_list_view = DockList::new(DockListType::Saved);
        dock.append(&saved_app_list_view);

        let separator = cascade! {
            Separator::new(Orientation::Vertical);
            ..set_margin_start(8);
            ..set_margin_end(8);
        };
        dock.append(&separator);

        let active_app_list_view = DockList::new(DockListType::Active);
        dock.append(&active_app_list_view);

        imp.cursor_handle.set(cursor_handle).unwrap();
        imp.revealer.set(revealer).unwrap();
        imp.saved_list.set(saved_app_list_view).unwrap();
        imp.active_list.set(active_app_list_view).unwrap();
        // Setup
        self_.setup_motion_controller();
        self_.setup_drop_target();
        self_.setup_callbacks();

        self_
    }

    pub fn model(&self, type_: DockListType) -> &gio::ListStore {
        // Get state
        let imp = imp::Window::from_instance(self);
        match type_ {
            DockListType::Active => imp.active_list.get().unwrap().model(),
            DockListType::Saved => imp.saved_list.get().unwrap().model(),
        }
    }

    fn setup_callbacks(&self) {
        // Get state
        let imp = imp::Window::from_instance(self);
        let window = self.clone().upcast::<gtk4::Window>();
        let cursor_event_controller = &imp.cursor_motion_controller.get().unwrap();
        // let drop_controller = &imp.drop_controller.get().unwrap();
        let window_drop_controller = &imp.window_drop_controller.get().unwrap();
        let revealer = &imp.revealer.get().unwrap();
        window.connect_show(
            glib::clone!(@weak revealer, @weak cursor_event_controller => move |_| {
                // dbg!(!cursor_event_controller.contains_pointer());
                if !cursor_event_controller.contains_pointer() {
                    revealer.set_reveal_child(false);
                }
            }),
        );
        window.connect_realize(glib::clone!(@weak revealer, @weak window_drop_controller, @weak cursor_event_controller => move |window| {
            if let Some((display, surface)) = x::get_window_x11(window) {
                // ignore all x11 errors...
                let xdisplay = display.clone().downcast::<X11Display>().expect("Failed to downgrade X11 Display.");
                xdisplay.error_trap_push();
                unsafe {
                    x::change_property(
                        &display,
                        &surface,
                        "_NET_WM_WINDOW_TYPE",
                        x::PropMode::Replace,
                        &[x::Atom::new(&display, "_NET_WM_WINDOW_TYPE_DOCK").unwrap()],
                    );
                }
                let resize = glib::clone!(@weak window, @weak revealer => move || {
                    let height = if revealer.reveals_child() { window.height() } else { 4 };
                    let width = window.width();

                    if let Some((display, _surface)) = x::get_window_x11(&window) {
                        let geom = display
                            .primary_monitor().geometry();
                        let monitor_x = geom.x();
                        let monitor_y = geom.y();
                        let monitor_width = geom.width();
                        let monitor_height = geom.height();
                        // dbg!(monitor_x);
                        // dbg!(monitor_y);
                        // dbg!(monitor_width);
                        // dbg!(monitor_height);
                        // dbg!(width);
                        // dbg!(height);
                        unsafe { x::set_position(&display, &surface,
                            (monitor_x + monitor_width / 2 - width / 2).clamp(0, monitor_x + monitor_width - 1),
                                                    (monitor_y + monitor_height - height).clamp(0, monitor_y + monitor_height - 1));}
                    }
                });

                let resize_drop = resize.clone();
                window_drop_controller.connect_enter(glib::clone!(@weak revealer, @weak window => @default-return gdk4::DragAction::COPY, move |_self, _x, _y| {
                    glib::source::idle_add_local_once(resize_drop.clone());
                    revealer.set_reveal_child(true);
                    gdk4::DragAction::COPY
                }));

                let resize_cursor = resize.clone();
                cursor_event_controller.connect_enter(glib::clone!(@weak revealer, @weak window => move |_evc, _x, _y| {
                    // dbg!("hello, mouse entered me :)");
                    revealer.set_reveal_child(true);
                    glib::source::idle_add_local_once(resize_cursor.clone());
                }));

                let resize_revealed = resize.clone();
                revealer.connect_child_revealed_notify(glib::clone!(@weak window => move |r| {
                    if !r.is_child_revealed() {
                        glib::source::idle_add_local_once(resize_revealed.clone());
                    }
                    }));

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
        }));

        let drop_controller = imp.saved_list.get().unwrap().drop_controller();
        cursor_event_controller.connect_leave(
            glib::clone!(@weak revealer, @weak drop_controller => move |_evc| {
                // only hide if DnD is not happening
                if drop_controller.current_drop().is_none() {
                    // dbg!("hello, mouse left me :)");
                    revealer.set_reveal_child(false);
                }
            }),
        );

        // hack to prevent hiding window when dnd from other apps
        drop_controller.connect_enter(glib::clone!(@weak revealer => @default-return gdk4::DragAction::COPY, move |_self, _x, _y| {

            revealer.set_reveal_child(true);
            gdk4::DragAction::COPY
        }));
        window_drop_controller.connect_drop(|_, _, _, _| {
            println!("dropping into window");
            false
        });
    }

    fn setup_motion_controller(&self) {
        let imp = imp::Window::from_instance(self);
        let handle = &imp.cursor_handle.get().unwrap();
        let ev = EventControllerMotion::builder()
            .propagation_limit(gtk4::PropagationLimit::None)
            .propagation_phase(gtk4::PropagationPhase::Capture)
            .build();
        handle.add_controller(&ev);

        imp.cursor_motion_controller
            .set(ev)
            .expect("Could not set event controller");
    }
    fn setup_drop_target(&self) {
        // hack for revealing hidden dock when drag enters dock window
        let imp = imp::Window::from_instance(self);
        let mut drop_actions = gdk4::DragAction::COPY;
        drop_actions.insert(gdk4::DragAction::MOVE);
        let drop_format = gdk4::ContentFormats::for_type(Type::STRING);
        let drop_format = drop_format.union(&gdk4::ContentFormats::for_type(Type::U32));

        let window_drop_target_controller = DropTarget::builder()
            .actions(drop_actions)
            .formats(&drop_format)
            .build();

        let enter_handle = &imp.cursor_handle.get().unwrap();
        enter_handle.add_controller(&window_drop_target_controller);
        imp.window_drop_controller
            .set(window_drop_target_controller)
            .expect("Could not set dock dnd drop controller");
    }
}
