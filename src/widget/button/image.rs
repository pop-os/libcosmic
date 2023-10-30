// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{Builder, Style};
use crate::{
    widget::{self, image::Handle},
    Element,
};
use apply::Apply;
use iced_core::{font::Weight, widget::Id, Length, Padding};
use std::borrow::Cow;

pub type Button<'a, Message> = Builder<'a, Message, Image<'a, Handle>>;

pub fn image<'a, Message>(handle: impl Into<Handle> + 'a) -> Button<'a, Message> {
    Button::new(Image {
        image: widget::image(handle).border_radius([9.0; 4]),
        selected: false,
    })
}

pub struct Image<'a, Handle> {
    image: widget::Image<'a, Handle>,
    selected: bool,
}

impl<'a, Message> Button<'a, Message> {
    pub fn new(variant: Image<'a, Handle>) -> Self {
        Self {
            id: Id::unique(),
            label: Cow::Borrowed(""),
            tooltip: Cow::Borrowed(""),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::from(0),
            spacing: 0,
            icon_size: 16,
            line_height: 20,
            font_size: 14,
            font_weight: Weight::Normal,
            style: Style::Image,
            variant,
        }
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.variant.selected = selected;
        self
    }
}

impl<'a, Message> From<Button<'a, Message>> for Element<'a, Message>
where
    Handle: Clone + std::hash::Hash,
    Message: Clone + 'static,
{
    fn from(builder: Button<'a, Message>) -> Element<'a, Message> {
        builder
            .variant
            .image
            .width(builder.width)
            .height(builder.height)
            .apply(widget::button)
            .selected(builder.variant.selected)
            .id(builder.id)
            .padding(0)
            .on_press_maybe(builder.on_press)
            .style(builder.style)
            .into()
    }
}
