use gdk4::ContentProvider;
use gdk4::Display;
use gio::File;
use gio::Icon;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::traits::WidgetExt;
use gtk4::DragSource;
use gtk4::IconTheme;
use gtk4::{gio, glib};

use crate::app_group::AppGroup;

mod imp;

glib::wrapper! {
pub struct GridItem(ObjectSubclass<imp::GridItem>)
    @extends gtk4::Widget, gtk4::Box,
    @implements gtk4::Accessible, gtk4::Actionable, gtk4::Buildable, gtk4::ConstraintTarget;
}

impl Default for GridItem {
    fn default() -> Self {
        Self::new()
    }
}

impl GridItem {
    pub fn new() -> Self {
        glib::Object::new(&[]).expect("Failed to create GridItem")
    }

    pub fn set_app_info(&self, app_info: &gio::DesktopAppInfo) {
        let self_ = imp::GridItem::from_instance(self);
        self_.name.set_text(&app_info.name());

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
        self_.image.set_from_gicon(&icon);
        drag_controller.connect_drag_begin(glib::clone!(@weak icon, => move |_self, drag| {
            drag.set_selected_action(gdk4::DragAction::MOVE);
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
        }));
    }

    pub fn set_group_info(&self, app_group: AppGroup) {
        let self_ = imp::GridItem::from_instance(self);
        if let Ok(name) = app_group.property("name") {
            self_.name.set_text(
                &name
                    .get::<String>()
                    .expect("property name needs to be a string."),
            );
        }
        if let Ok(icon) = app_group.property("icon") {
            self_.image.set_from_icon_name(Some(
                &icon
                    .get::<String>()
                    .expect("Property name needs to be a String."),
            ));
        }
    }

    pub fn set_index(&self, index: u32) {
        imp::GridItem::from_instance(self).index.set(index);
    }
}
