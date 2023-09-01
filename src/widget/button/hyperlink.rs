// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::Builder;
use super::Style;
use crate::widget::icon::{self, Handle};
use iced_core::{font::Weight, widget::Id, Length, Padding};
use std::borrow::Cow;

pub type Button<'a, Message> = Builder<'a, Message, Hyperlink>;

pub struct Hyperlink {
    trailing_icon: bool,
}

pub fn hyperlink<'a, Message>() -> Button<'a, Message> {
    Button::new(Hyperlink {
        trailing_icon: false,
    })
}

impl<'a, Message> Button<'a, Message> {
    pub fn new(link: Hyperlink) -> Self {
        Self {
            id: Id::unique(),
            label: Cow::Borrowed(""),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::from(0),
            spacing: 0,
            icon_size: 16,
            line_height: 20,
            font_size: 14,
            font_weight: Weight::Normal,
            style: Style::Link,
            variant: link,
        }
    }

    pub const fn with_icon(mut self) -> Self {
        self.variant.trailing_icon = true;
        self
    }
}

pub fn icon() -> Handle {
    icon::handle::from_svg_bytes(&include_bytes!("../../../res/external-link.svg")[..])
        .symbolic(true)
}
