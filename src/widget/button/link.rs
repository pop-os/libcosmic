// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Hyperlink button widget

use super::Builder;
use super::ButtonClass;
use crate::Element;
use crate::prelude::*;
use crate::widget::icon::{self, Handle};
use crate::widget::{button, row, tooltip};
use iced_core::text::LineHeight;
use iced_core::{Alignment, Length, Padding, font::Weight, widget::Id};
use std::borrow::Cow;

pub type Button<'a, Message> = Builder<'a, Message, Hyperlink>;

pub struct Hyperlink {
    trailing_icon: bool,
}

/// A hyperlink button.
pub fn link<'a, Message>(label: impl Into<Cow<'a, str>> + 'static) -> Button<'a, Message> {
    Button::new(
        label,
        Hyperlink {
            trailing_icon: false,
        },
    )
}

impl<'a, Message> Button<'a, Message> {
    pub fn new(label: impl Into<Cow<'a, str>> + 'static, link: Hyperlink) -> Self {
        Self {
            id: Id::unique(),
            label: label.into(),
            tooltip: Cow::Borrowed(""),
            on_press: None,
            width: Length::Shrink,
            height: Length::Shrink,
            padding: Padding::from(4),
            spacing: 0,
            icon_size: 16,
            line_height: 20,
            font_size: 14,
            font_weight: Weight::Normal,
            class: ButtonClass::Link,
            variant: link,
        }
    }

    pub const fn trailing_icon(mut self, set: bool) -> Self {
        self.variant.trailing_icon = set;
        self
    }
}

#[inline(never)]
pub fn icon() -> Handle {
    icon::from_svg_bytes(&include_bytes!("external-link.svg")[..]).symbolic(true)
}

impl<'a, Message: Clone + 'static> From<Button<'a, Message>> for Element<'a, Message> {
    fn from(mut builder: Button<'a, Message>) -> Element<'a, Message> {
        let button: super::Button<'a, Message> = row::with_capacity(2)
            .push({
                // TODO: Avoid allocation
                crate::widget::text(builder.label.to_string())
                    .size(builder.font_size)
                    .line_height(LineHeight::Absolute(builder.line_height.into()))
                    .font(crate::font::Font {
                        weight: builder.font_weight,
                        ..crate::font::default()
                    })
            })
            .push_maybe(if builder.variant.trailing_icon {
                Some(icon().icon().size(builder.icon_size))
            } else {
                None
            })
            .padding(builder.padding)
            .width(builder.width)
            .height(builder.height)
            .spacing(builder.spacing)
            .align_y(Alignment::Center)
            .apply(button::custom)
            .padding(0)
            .id(builder.id)
            .on_press_maybe(builder.on_press.take())
            .class(builder.class);

        if builder.tooltip.is_empty() {
            button.into()
        } else {
            tooltip(
                button,
                crate::widget::text(builder.tooltip)
                    .size(builder.font_size)
                    .font(crate::font::Font {
                        weight: builder.font_weight,
                        ..crate::font::default()
                    }),
                tooltip::Position::Top,
            )
            .into()
        }
    }
}
