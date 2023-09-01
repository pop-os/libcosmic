// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{button, Builder, Style};
use crate::widget::{self, icon::Handle, row};
use crate::{ext::CollectionWidget, Element};
use apply::Apply;
use iced_core::{font::Weight, text::LineHeight, widget::Id, Alignment, Length, Padding};
use std::borrow::Cow;

/// A [`Button`] with the highest level of attention.
///
/// There should only be one primary button used per page.
pub type Button<'a, Message> = Builder<'a, Message, Text>;

pub fn destructive<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message> {
    Button::new(Text::new())
        .label(label)
        .style(Style::Destructive)
}

pub fn suggested<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message> {
    Button::new(Text::new())
        .label(label)
        .style(Style::Suggested)
}

pub fn standard<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message> {
    Button::new(Text::new()).label(label)
}

pub fn text<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message> {
    Button::new(Text::new()).label(label).style(Style::Text)
}

pub struct Text {
    pub(super) leading_icon: Option<crate::widget::icon::Handle>,
    pub(super) trailing_icon: Option<crate::widget::icon::Handle>,
}

impl Text {
    pub const fn new() -> Self {
        Self {
            leading_icon: None,
            trailing_icon: None,
        }
    }
}

impl<'a, Message> Button<'a, Message> {
    pub fn new(text: Text) -> Self {
        crate::theme::THEME.with(|theme_cell| {
            let theme = theme_cell.borrow();
            let theme = theme.cosmic();
            Self {
                id: Id::unique(),
                label: Cow::Borrowed(""),
                on_press: None,
                width: Length::Shrink,
                height: Length::Fixed(theme.space_l().into()),
                padding: Padding::from([0, theme.space_s()]),
                spacing: theme.space_xxxs(),
                icon_size: 16,
                line_height: 20,
                font_size: 14,
                font_weight: Weight::Normal,
                style: Style::Standard,
                variant: text,
            }
        })
    }

    pub fn leading_icon(mut self, icon: impl Into<Handle>) -> Self {
        self.variant.leading_icon = Some(icon.into());
        self
    }

    pub fn trailing_icon(mut self, icon: impl Into<Handle>) -> Self {
        self.variant.trailing_icon = Some(icon.into());
        self
    }
}

impl<'a, Message: Clone + 'static> From<Button<'a, Message>> for Element<'a, Message> {
    fn from(mut b: Button<'a, Message>) -> Element<'a, Message> {
        // TODO: Determine why this needs to be set before the label to prevent lifetime conflict.
        let trailing_icon = b
            .variant
            .trailing_icon
            .map(|i| Element::from(widget::icon(i).size(b.icon_size)));

        row::with_capacity(3)
            // Optional icon to place before label.
            .push_maybe(
                b.variant
                    .leading_icon
                    .map(|i| widget::icon(i).size(b.icon_size)),
            )
            // Optional label between icons.
            .push_maybe((!b.label.is_empty()).then(|| {
                let mut font = crate::font::DEFAULT;
                font.weight = b.font_weight;

                crate::widget::text(b.label)
                    .size(b.font_size)
                    .line_height(LineHeight::Absolute(b.line_height.into()))
                    .font(font)
            }))
            // Optional icon to place behind the label.
            .push_maybe(trailing_icon)
            .padding(b.padding)
            .width(b.width)
            .height(b.height)
            .spacing(b.spacing)
            .align_items(Alignment::Center)
            .apply(button)
            .padding(0)
            .id(b.id)
            .on_press_maybe(b.on_press.take())
            .style(b.style)
            .into()
    }
}
