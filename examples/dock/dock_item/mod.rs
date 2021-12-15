use crate::icon_source;
use gdk4::ContentProvider;
use gdk4::Display;
use gio::File;
use glib::FromVariant;
use glib::Variant;
use gtk4 as gtk;
use gtk4::DragSource;
use gtk4::IconTheme;
mod imp;

use crate::ApplicationObject;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

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
        glib::Object::new(&[]).expect("Failed to create DockItem")
    }

    pub fn set_app_info(&self, app_info: &gio::DesktopAppInfo) {
        dbg!("setting app info");
        let self_ = imp::DockItem::from_instance(self);
        self_.image.set_tooltip_text(Some(&app_info.name()));

        let drag = DragSource::builder()
            .name("application library drag source")
            .actions(gdk4::DragAction::COPY)
            // .content()
            .build();
        self.add_controller(&drag);
        if let Some(file) = app_info.filename() {
            let file = File::for_path(file);
            let provider = ContentProvider::for_value(&file.to_value());
            drag.set_content(Some(&provider));
        }
        if let Some(icon) = app_info.icon() {
            dbg!("setting icon {}", app_info.name());
            self_.image.set_from_gicon(&icon);
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
                        drag.set_icon(Some(&paintable_icon), 32, 32);
                    }
                }
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
