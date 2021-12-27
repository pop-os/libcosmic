use crate::utils::BoxedWindowList;
use gdk4::ContentProvider;
use gdk4::Display;
use gio::DesktopAppInfo;
use gio::Icon;
use gio::ListStore;
use gtk4 as gtk;
use gtk4::DragSource;
use gtk4::IconTheme;
mod imp;

use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::dock_object::DockObject;

glib::wrapper! {
    pub struct DockItem(ObjectSubclass<imp::DockItem>)
        @extends gtk::Widget, gtk::Box;
}

impl Default for DockItem {
    fn default() -> Self {
        Self::new()
    }
}

impl DockItem {
    pub fn new() -> Self {
        let mut actions = gdk4::DragAction::MOVE;
        actions.insert(gdk4::DragAction::MOVE);

        let self_: DockItem = glib::Object::new(&[]).expect("Failed to create DockItem");
        self_
    }

    pub fn drag_controller(&self) -> &DragSource {
        let imp = imp::DockItem::from_instance(self);
        imp.drag_controller
            .get()
            .expect("Could not get drag_controller")
    }

    // refactor to emit event for removing the item?
    pub fn set_app_info(&self, dock_object: &DockObject, i: u32, saved_app_model: &ListStore) {
        let self_ = imp::DockItem::from_instance(self);
        if let Ok(app_info_value) = dock_object.property("appinfo") {
            if let Ok(Some(app_info)) = app_info_value.get::<Option<DesktopAppInfo>>() {
                self_.image.set_tooltip_text(Some(&app_info.name()));

                let icon = app_info.icon().unwrap_or(
                    Icon::for_string("image-missing").expect("Failed to set default icon"),
                );

                self_.image.set_from_gicon(&icon);
                if let Some(drag_controller) = self_.drag_controller.get() {
                    if let Some(file) = app_info.filename() {
                        let provider =
                            ContentProvider::for_value(&file.to_string_lossy().to_value());
                        drag_controller.set_content(Some(&provider));
                    }
                    drag_controller.connect_drag_cancel(
                       glib::clone!(@weak saved_app_model, @weak app_info => @default-return true, move |_self, _drag, _delete_data| {
                           if let Some(item) = saved_app_model.item(i) {
                               if let Ok(cur_app_info) = item.downcast::<DockObject>() {
                                   if let Ok(Some(cur_app_info)) = cur_app_info.property("appinfo").expect("property appinfo missing from DockObject").get::<Option<DesktopAppInfo>>() {
                                       // dbg!(cur_app_info.filename());
                                       if cur_app_info.filename() == app_info.filename() {
                                           saved_app_model.remove(i);
                                       }
                                   }
                               }
                           }
                           true
                       }),
                    );
                    drag_controller.connect_drag_end(
                        glib::clone!(@weak saved_app_model, @weak app_info => move |_self, _drag, _delete_data| {
                            dbg!(i);
                            dbg!(_delete_data);
                            if let Some(item) = saved_app_model.item(i) {
                                if let Ok(cur_app_info) = item.downcast::<DockObject>() {
                                    if let Ok(Some(cur_app_info)) = cur_app_info.property("appinfo").expect("property appinfo missing from DockObject").get::<Option<DesktopAppInfo>>() {
                                        // dbg!(cur_app_info.filename());
                                        if cur_app_info.filename() == app_info.filename() {
                                            saved_app_model.remove(i);
                                        }
                                    }
                                }
                            }
                        }),
                    );

                    let icon = app_info.icon().unwrap_or(
                        Icon::for_string("image-missing").expect("Failed to set default icon"),
                    );
                    drag_controller.connect_drag_begin(
                        glib::clone!(@weak icon, => move |_self, _drag| {
                            // set drag source icon if possible...
                            // gio Icon is not easily converted to a Paintable, but this seems to be the correct method
                            _drag.set_selected_action(gdk4::DragAction::MOVE);
                            if let Some(default_display) = &Display::default() {
                                if let Some(icon_theme) = IconTheme::for_display(default_display) {
                                    if let Some(paintable_icon) = icon_theme.lookup_by_gicon(
                                        &icon,
                                        64,
                                        1,
                                        gtk4::TextDirection::None,
                                        gtk4::IconLookupFlags::empty(),
                                    ) {
                                        _self.set_icon(Some(&paintable_icon), 32, 32);
                                    }
                                }
                            }
                        }),
                    );
                }
            }
        } else {
            println!("initializing dock item failed...");
        }
        if let Ok(active_value) = dock_object.property("active") {
            if let Ok(active) = active_value.get::<BoxedWindowList>() {
                self_.dots.set_text("");
                for _ in active.0 {
                    self_
                        .dots
                        .set_text(format!("{}{}", self_.dots.text(), " · ").as_str());
                }
            }
        }
    }
}
