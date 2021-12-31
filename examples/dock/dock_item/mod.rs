use cascade::cascade;
use gio::DesktopAppInfo;
use gio::Icon;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::Align;
use gtk4::AspectFrame;
use gtk4::Box;
use gtk4::Image;
use gtk4::Label;
use gtk4::Orientation;

use crate::dock_object::DockObject;
use crate::utils::BoxedWindowList;

mod imp;

glib::wrapper! {
    pub struct DockItem(ObjectSubclass<imp::DockItem>)
        @extends gtk4::Button, gtk4::Widget,
        @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for DockItem {
    fn default() -> Self {
        Self::new()
    }
}

impl DockItem {
    pub fn new() -> Self {
        let self_: DockItem = glib::Object::new(&[]).expect("Failed to create DockItem");

        let item_box = Box::new(Orientation::Vertical, 0);
        self_.set_child(Some(&item_box));

        let image = cascade! {
            Image::new();
            ..set_hexpand(true);
            ..set_halign(Align::Center);
            ..set_pixel_size(64);
        };
        let dots = cascade! {
            Label::new(Some(""));
            ..set_hexpand(true);
            ..set_halign(Align::Center);
        };
        item_box.append(&image);
        item_box.append(&dots);

        let imp = imp::DockItem::from_instance(&self_);
        imp.image.replace(image);
        imp.dots.replace(dots);
        self_.show();
        self_
    }

    // refactor to emit event for removing the item?
    pub fn set_app_info(&self, dock_object: &DockObject) {
        let self_ = imp::DockItem::from_instance(self);
        if let Ok(app_info_value) = dock_object.property("appinfo") {
            if let Ok(Some(app_info)) = app_info_value.get::<Option<DesktopAppInfo>>() {
                self_
                    .image
                    .borrow()
                    .set_tooltip_text(Some(&app_info.name()));

                let icon = app_info.icon().unwrap_or(
                    Icon::for_string("image-missing").expect("Failed to set default icon"),
                );

                self_.image.borrow().set_from_gicon(&icon);
            }
        } else {
            println!("initializing dock item failed...");
        }
        if let Ok(active_value) = dock_object.property("active") {
            if let Ok(active) = active_value.get::<BoxedWindowList>() {
                let dots = self_.dots.borrow();
                dots.set_text("");
                for _ in active.0 {
                    dots.set_text(format!("{}{}", dots.text(), " · ").as_str());
                }
            }
        }
    }
}
