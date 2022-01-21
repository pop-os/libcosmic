use cascade::cascade;
use gdk4::ContentProvider;
use gdk4::Display;
use gio::File;
use gio::Icon;
use gtk4::pango::EllipsizeMode;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::traits::WidgetExt;
use gtk4::Align;
use gtk4::Button;
use gtk4::DragSource;
use gtk4::IconTheme;
use gtk4::Image;
use gtk4::Label;
use gtk4::Orientation;
use gtk4::{gio, glib};

use crate::app_group::AppGroup;
use crate::app_group::BoxedAppGroupType;

mod imp;

glib::wrapper! {
pub struct GridItem(ObjectSubclass<imp::GridItem>)
        @extends gtk4::Widget, gtk4::Box,
        @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl Default for GridItem {
    fn default() -> Self {
        Self::new()
    }
}

impl GridItem {
    pub fn new() -> Self {
        let self_ = glib::Object::new(&[]).expect("Failed to create GridItem");
        let imp = imp::GridItem::from_instance(&self_);

        cascade! {
            &self_;
            ..set_orientation(Orientation::Vertical);
            ..set_halign(Align::Center);
            ..set_hexpand(true);
            ..set_margin_top(4);
            ..set_margin_bottom(4);
            ..set_margin_end(4);
            ..set_margin_start(4);
        };

        let image = cascade! {
            Image::new();
            ..set_margin_top(4);
            ..set_margin_bottom(4);
            ..set_pixel_size(64);
        };
        self_.append(&image);

        let name = cascade! {
            Label::new(None);
            ..set_halign(Align::Center);
            ..set_hexpand(true);
            ..set_ellipsize(EllipsizeMode::End);
            ..add_css_class("title-5");
        };
        self_.append(&name);

        imp.name.replace(name);
        imp.image.replace(image);
        self_
    }

    pub fn set_app_info(&self, app_info: &gio::DesktopAppInfo) {
        let self_ = imp::GridItem::from_instance(self);
        self_.name.borrow().set_text(&app_info.name());

        let drag_controller = DragSource::builder()
            .name("application library drag source")
            .actions(gdk4::DragAction::COPY)
            // .content()
            .build();
        self.add_controller(&drag_controller);
        if let Some(file) = app_info.filename() {
            let file = File::for_path(file);
            let provider = ContentProvider::for_value(&file.to_value());
            drag_controller.set_content(Some(&provider));
        }
        let icon = app_info
            .icon()
            .unwrap_or(Icon::for_string("image-missing").expect("Failed to set default icon"));
        self_.image.borrow().set_from_gicon(&icon);
        drag_controller.connect_drag_begin(glib::clone!(@weak icon, => move |_self, drag| {
            drag.set_selected_action(gdk4::DragAction::MOVE);
            // set drag source icon if possible...
            // gio Icon is not easily converted to a Paintable, but this seems to be the correct method
            if let Some(default_display) = &Display::default() {
                let icon_theme = IconTheme::for_display(default_display);
                let paintable_icon = icon_theme.lookup_by_gicon(
                    &icon,
                    64,
                    1,
                    gtk4::TextDirection::None,
                    gtk4::IconLookupFlags::empty(),
                );
                _self.set_icon(Some(&paintable_icon), 32, 32);
            }
        }));
    }

    pub fn set_group_info(&self, app_group: AppGroup) {
        // if data type set name and icon to values in data
        let imp = imp::GridItem::from_instance(self);
        match app_group.property::<BoxedAppGroupType>("inner") {
            BoxedAppGroupType::Group(data) => {
                imp.name.borrow().set_text(&data.name);
                imp.image.borrow().set_from_icon_name(Some(&data.icon));
            }
            BoxedAppGroupType::NewGroup(popover_active) => {
                // else must be add group
                imp.name.borrow().set_text("New Group");
                imp.image.borrow().set_from_icon_name(Some("folder-new"));

                let popover_menu = gtk4::Box::builder()
                    .spacing(12)
                    .hexpand(true)
                    .orientation(gtk4::Orientation::Vertical)
                    .margin_top(12)
                    .margin_bottom(12)
                    .margin_end(12)
                    .margin_start(12)
                    .build();

                // build menu
                let dialog_entry = gtk4::Entry::new();
                let label = cascade! {
                    Label::new(Some("Name"));
                    ..set_justify(gtk4::Justification::Left);
                    ..set_xalign(0.0);
                };
                popover_menu.append(&label);
                popover_menu.append(&dialog_entry);
                let btn_container = cascade! {
                    gtk4::Box::new(Orientation::Horizontal, 8);
                };
                let ok_btn = cascade! {
                    Button::with_label("Ok");
                };
                let cancel_btn = cascade! {
                    Button::with_label("Cancel");
                };
                btn_container.append(&ok_btn);
                btn_container.append(&cancel_btn);
                popover_menu.append(&btn_container);
                let popover = cascade! {
                    gtk4::Popover::new();
                    ..set_autohide(true);
                    ..set_child(Some(&popover_menu));
                };
                self.append(&popover);

                popover.connect_closed(
                    glib::clone!(@weak self as self_, @weak dialog_entry => move |_| {
                        dialog_entry.set_text("");
                        self_.emit_by_name::<()>("popover-closed", &[]);
                    }),
                );
                ok_btn.connect_clicked(
                    glib::clone!(@weak self as self_, @weak dialog_entry, @weak popover => move |_| {
                        let new_name = dialog_entry.text().to_string();
                        popover.popdown();
                        glib::idle_add_local_once(glib::clone!(@weak self_ => move || {
                            self_.emit_by_name::<()>("new-group", &[&new_name]);
                        }));
                    }),
                );
                cancel_btn.connect_clicked(glib::clone!(@weak popover => move |_| {
                    popover.popdown();
                }));
                if popover_active {
                    popover.popup();
                }

                imp.popover.replace(Some(popover));
            }
        }
    }

    pub fn set_index(&self, index: u32) {
        imp::GridItem::from_instance(self).index.set(index);
    }

    pub fn popup(&self) {
        let imp = imp::GridItem::from_instance(self);
        if let Some(popover) = imp.popover.borrow().as_ref() {
            popover.popup();
        }
    }
}
