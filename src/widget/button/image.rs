// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::Builder;
use crate::{
    Element,
    widget::{self, image::Handle},
};
use iced_core::{Length, Padding, font::Weight, widget::Id};
use std::borrow::Cow;

pub type Button<'a, Message> = Builder<'a, Message, Image<'a, Handle, Message>>;

/// A button constructed from an image handle, using image button styling.
pub fn image<'a, Message>(handle: impl Into<Handle> + 'a) -> Button<'a, Message> {
    Button::new(Image {
        image: widget::image(handle).border_radius([9.0; 4]),
        selected: false,
        on_remove: None,
    })
}

/// The image variant of a button.
pub struct Image<'a, Handle, Message> {
    image: widget::Image<'a, Handle>,
    selected: bool,
    on_remove: Option<Message>,
}

impl<'a, Message> Button<'a, Message> {
    #[inline]
    pub fn new(variant: Image<'a, Handle, Message>) -> Self {
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
            class: crate::theme::style::Button::Image,
            variant,
        }
    }

    #[inline]
    pub fn on_remove(mut self, message: Message) -> Self {
        self.variant.on_remove = Some(message);
        self
    }

    #[inline]
    pub fn on_remove_maybe(mut self, message: Option<Message>) -> Self {
        self.variant.on_remove = message;
        self
    }

    #[inline]
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
        let content = builder
            .variant
            .image
            .width(builder.width)
            .height(builder.height);

        super::custom_image_button(content, builder.variant.on_remove)
            .padding(0)
            .selected(builder.variant.selected)
            .id(builder.id)
            .on_press_maybe(builder.on_press)
            .class(builder.class)
            .into()
    }
}
