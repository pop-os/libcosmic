use cascade::cascade;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::Align;
use gtk4::Box;
use gtk4::Image;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::Popover;

use crate::dock_object::DockObject;
use crate::dock_popover::DockPopover;
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
        cascade! {
            &self_;
            ..set_child(Some(&item_box));
            ..add_css_class("dock_item");
        };

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
        let popover = cascade! {
            Popover::new();
            ..set_autohide(true);
        };
        item_box.append(&popover);
        let self_clone = self_.clone();
        popover.connect_closed(move |_| {
            let _ = self_clone.emit_by_name::<()>("popover-closed", &[]);
        });

        let popover_menu = cascade! {
            DockPopover::new();
        };
        popover.set_child(Some(&popover_menu));
        popover_menu.connect_local(
            "menu-hide",
            false,
            glib::clone!(@weak popover, @weak popover_menu => @default-return None, move |_| {
                popover.popdown();
                popover_menu.reset_menu();
                None
            }),
        );

        let imp = imp::DockItem::from_instance(&self_);
        imp.image.replace(Some(image));
        imp.dots.replace(dots);
        imp.item_box.replace(item_box);
        imp.popover.replace(popover);
        imp.popover_menu.replace(popover_menu);

        self_
    }

    // refactor to emit event for removing the item?
    pub fn set_dock_object(&self, dock_object: &DockObject) {
        let self_ = imp::DockItem::from_instance(self);
        let image = cascade! {
            dock_object.get_image();
            ..set_hexpand(true);
            ..set_halign(Align::Center);
            ..set_pixel_size(64);
            ..set_tooltip_text(dock_object.get_name().as_deref());
        };
        let old_image = self_.image.replace(None);
        if let Some(old_image) = old_image {
            self_.item_box.borrow().remove(&old_image);
            self_.item_box.borrow().prepend(&image);
            self_.image.replace(Some(image));
        }
        let active = dock_object.property::<BoxedWindowList>("active");
        let dots = self_.dots.borrow();
        dots.set_text("");
        for _ in active.0 {
            dots.set_text(format!("{}{}", dots.text(), " · ").as_str());
        }

        let popover = dock_object.property::<bool>("popover");
        // dbg!(popover);
        // dbg!(dock_object);
        if popover {
            self.add_popover(dock_object);
        } else {
            self.clear_popover();
        }
    }

    pub fn add_popover(&self, item: &DockObject) {
        let imp = imp::DockItem::from_instance(self);
        let popover = imp.popover.borrow();
        let popover_menu = imp.popover_menu.borrow();

        popover_menu.set_dock_object(item, true);
        popover.popup();
    }

    pub fn clear_popover(&self) {
        let imp = imp::DockItem::from_instance(self);
        let popover = imp.popover.borrow();
        let popover_menu = imp.popover_menu.borrow();
        popover.popdown();
        popover_menu.reset_menu();
    }
}
