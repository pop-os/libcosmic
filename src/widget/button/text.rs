// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{button, Builder, Style};
use crate::widget::{icon, row, tooltip};
use crate::{ext::CollectionWidget, Element};
use apply::Apply;
use iced_core::{font::Weight, text::LineHeight, widget::Id, Alignment, Length, Padding};
use std::borrow::Cow;

pub type Button<'a, Message> = Builder<'a, Message, Text>;

/// A text button with the destructive style
pub fn destructive<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message> {
    Button::new(Text::new())
        .label(label)
        .style(Style::Destructive)
}

/// A text button with the suggested style
pub fn suggested<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message> {
    Button::new(Text::new())
        .label(label)
        .style(Style::Suggested)
}

/// A text button with the standard style
pub fn standard<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message> {
    Button::new(Text::new()).label(label)
}

/// A text button with the text style
pub fn text<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message> {
    Button::new(Text::new()).label(label).style(Style::Text)
}

/// The text variant of a button.
pub struct Text {
    pub(super) leading_icon: Option<icon::Handle>,
    pub(super) trailing_icon: Option<icon::Handle>,
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
                tooltip: Cow::Borrowed(""),
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

    pub fn leading_icon(mut self, icon: impl Into<icon::Handle>) -> Self {
        self.variant.leading_icon = Some(icon.into());
        self
    }

    pub fn trailing_icon(mut self, icon: impl Into<icon::Handle>) -> Self {
        self.variant.trailing_icon = Some(icon.into());
        self
    }
}

impl<'a, Message: Clone + 'static> From<Button<'a, Message>> for Element<'a, Message> {
    fn from(mut builder: Button<'a, Message>) -> Element<'a, Message> {
        let trailing_icon = builder.variant.trailing_icon.map(|mut i| {
            if let icon::Data::Name(ref mut named) = i.data {
                named.size = Some(builder.icon_size);
            }

            i.icon()
        });

        let leading_icon = builder.variant.leading_icon.map(|mut i| {
            if let icon::Data::Name(ref mut named) = i.data {
                named.size = Some(builder.icon_size);
            }

            i.icon()
        });

        let label: Option<Element<'_, _>> = (!builder.label.is_empty()).then(|| {
            let mut font = crate::font::DEFAULT;
            font.weight = builder.font_weight;

            // TODO: Avoid allocation
            crate::widget::text(builder.label.to_string())
                .size(builder.font_size)
                .line_height(LineHeight::Absolute(builder.line_height.into()))
                .font(font)
                .into()
        });

        let button: super::Button<'a, Message> = row::with_capacity(3)
            // Optional icon to place before label.
            .push_maybe(leading_icon)
            // Optional label between icons.
            .push_maybe(label)
            // Optional icon to place behind the label.
            .push_maybe(trailing_icon)
            .padding(builder.padding)
            .width(builder.width)
            .height(builder.height)
            .spacing(builder.spacing)
            .align_items(Alignment::Center)
            .apply(button)
            .padding(0)
            .id(builder.id)
            .on_press_maybe(builder.on_press.take())
            .style(builder.style);

        if builder.tooltip.is_empty() {
            button.into()
        } else {
            tooltip(button, builder.tooltip, tooltip::Position::Top)
                .size(builder.font_size)
                .font({
                    let mut font = crate::font::DEFAULT;
                    font.weight = builder.font_weight;
                    font
                })
                .into()
        }
    }
}
