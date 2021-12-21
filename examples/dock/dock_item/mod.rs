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
        let dc = DragSource::builder()
            .name("dock drag source")
            .actions(gdk4::DragAction::COPY)
            .build();

        let self_: DockItem = glib::Object::new(&[]).expect("Failed to create DockItem");
        self_.add_controller(&dc);
        imp::DockItem::from_instance(&self_)
            .drag_controller
            .set(dc)
            .expect("Failed to set dock item");
        self_
    }

    pub fn drag_controller(&self) -> &DragSource {
        let imp = imp::DockItem::from_instance(self);
        imp.drag_controller
            .get()
            .expect("Could not get drag_controller")
    }

    // TODO current method seems very messy...
    // refactor to emit event for removing the item?
    pub fn set_app_info(&self, app_info: &DockObject, i: u32, saved_app_model: &ListStore) {
        if let Ok(app_info_value) = app_info.property("appinfo") {
            if let Ok(Some(app_info)) = app_info_value.get::<Option<DesktopAppInfo>>() {
                dbg!("setting app info");
                let self_ = imp::DockItem::from_instance(self);
                self_.image.set_tooltip_text(Some(&app_info.name()));

                if let Some(drag_controller) = self_.drag_controller.get() {
                    if let Some(file) = app_info.filename() {
                        let provider =
                            ContentProvider::for_value(&file.to_string_lossy().to_value());
                        drag_controller.set_content(Some(&provider));
                    }
                    drag_controller.connect_drag_end(move |_self, _drag, delete_data| {
                        dbg!("removing", delete_data);
                    });
                    //TODO investigate rare X11 errors when reordering dock items
                    drag_controller.connect_drag_cancel(
                            glib::clone!(@weak saved_app_model => @default-return true, move |_self, _drag, _delete_data| {
                                dbg!("removing {}", i);
                                if saved_app_model.n_items() > i {
                                    saved_app_model.remove(i);
                                }
                                true
                            }),
                        );
                    drag_controller.connect_drag_end(
                        glib::clone!(@weak saved_app_model => move |_self, _drag, delete_data| {
                            dbg!("removing {}", i);
                            if delete_data && saved_app_model.n_items() > i {
                                saved_app_model.remove(i);
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

                let icon = app_info.icon().unwrap_or(
                    Icon::for_string("image-missing").expect("Failed to set default icon"),
                );

                self_.image.set_from_gicon(&icon);
            }
        }
    }

    // pub fn set_app_info(&self, app_obj: ApplicationObject) {
    //     let self_ = imp::DockItem::from_instance(self);

    //     if let Ok(name) = app_obj.property("name") {
    //         self_.image.set_tooltip_text(Some(
    //             &name
    //                 .get::<String>()
    //                 .expect("Property name needs to be a String."),
    //         ));
    //     }
    //     if let Ok(icon) = app_obj.property("icon") {
    //         if let Ok(icon) = icon.get::<Variant>() {
    //             let icon = match <(i32, String)>::from_variant(&icon) {
    //                 Some((i_type, name)) if i_type == pop_launcher::IconSource::Name as i32 => {
    //                     Some(pop_launcher::IconSource::Name(name.into()))
    //                 }
    //                 Some((i_type, name)) if i_type == pop_launcher::IconSource::Mime as i32 => {
    //                     Some(pop_launcher::IconSource::Mime(name.into()))
    //                 }
    //                 _ => None,
    //             };
    //             icon_source(&self_.image, &icon);
    //         }
    //     }
    // }
}
