use cascade::cascade;
use gtk4::glib;
use gtk4::pango::EllipsizeMode;
use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use gtk4::Align;
use gtk4::Box;
use gtk4::Image;
use gtk4::Label;
use gtk4::Orientation;

use crate::utils::icon_source;
use crate::BoxedSearchResult;
use crate::SearchResultObject;

mod imp;

glib::wrapper! {
    pub struct SearchResultRow(ObjectSubclass<imp::SearchResultRow>)
        @extends gtk4::Widget, gtk4::Box,
    @implements gtk4::Accessible, gtk4::Buildable, gtk4::ConstraintTarget, gtk4::Orientable;
}

impl Default for SearchResultRow {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchResultRow {
    pub fn new() -> Self {
        let self_ = glib::Object::new(&[]).expect("Failed to create SearchResultRow");
        let imp = imp::SearchResultRow::from_instance(&self_);

        cascade! {
            &self_;
            ..set_orientation(Orientation::Horizontal);
            ..set_spacing(12);
            ..set_margin_start(4);
            ..set_margin_end(4);
            ..set_hexpand(true);
        };

        let category_image = cascade! {
            Image::new();
            ..set_pixel_size(24);
        };
        self_.append(&category_image);

        let image = cascade! {
            Image::new();
            ..set_margin_top(4);
            ..set_margin_bottom(4);
            ..set_pixel_size(40);
        };
        self_.append(&image);

        let text_container = cascade! {
            Box::new(Orientation::Vertical, 0);
            ..set_halign(Align::Fill);
            ..set_hexpand(true);
            ..set_margin_top(4);
            ..set_margin_end(4);
            ..set_margin_bottom(4);
        };
        self_.append(&text_container);

        let shortcut = cascade! {
            Label::new(None);
            ..set_halign(Align::End);
            ..set_wrap(false);
            ..add_css_class("body");
        };
        self_.append(&shortcut);

        let name = cascade! {
            Label::new(None);
            ..set_halign(Align::Start);
            ..set_ellipsize(EllipsizeMode::End);
            ..set_max_width_chars(40);
            ..add_css_class("title-4");
        };
        text_container.append(&name);

        let description = cascade! {
            Label::new(None);
            ..set_halign(Align::Start);
            ..set_ellipsize(EllipsizeMode::End);
            ..set_max_width_chars(50);
            ..add_css_class("body");
        };
        text_container.append(&description);

        imp.category_image.replace(category_image);
        imp.image.replace(image);
        imp.name.replace(name);
        imp.description.replace(description);
        imp.shortcut.replace(shortcut);

        self_
    }

    pub fn set_search_result(&self, search_obj: SearchResultObject) {
        let self_ = imp::SearchResultRow::from_instance(self);
        let search_result = search_obj.property::<BoxedSearchResult>("data");
        if let Some(search_result) = search_result.0 {
            self_.name.borrow().set_text(&search_result.name);
            self_
                .description
                .borrow()
                .set_text(&search_result.description);
            icon_source(&self_.image, &search_result.icon);
            icon_source(&self_.category_image, &search_result.category_icon);
        }
    }

    pub fn set_shortcut(&self, indx: u32) {
        let self_ = imp::SearchResultRow::from_instance(self);
        self_
            .shortcut
            .borrow()
            .set_text(&format!("Ctrl + {}", indx));
    }
}
