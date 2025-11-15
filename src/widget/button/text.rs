// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{Builder, ButtonClass};
use crate::widget::{icon, row, tooltip};
use crate::{Apply, Element};
use iced_core::{Alignment, Length, Padding, font::Weight, text::LineHeight, widget::Id};
use std::borrow::Cow;

pub type Button<'a, Message> = Builder<'a, Message, Text>;

/// A text button with the destructive style
pub fn destructive<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message> {
    Button::new(Text::new())
        .label(label)
        .class(ButtonClass::Destructive)
}

/// A text button with the suggested style
pub fn suggested<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message> {
    Button::new(Text::new())
        .label(label)
        .class(ButtonClass::Suggested)
}

/// A text button with the standard style
pub fn standard<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message> {
    Button::new(Text::new()).label(label)
}

/// A text button with the text style
pub fn text<'a, Message>(label: impl Into<Cow<'a, str>>) -> Button<'a, Message> {
    Button::new(Text::new())
        .label(label)
        .class(ButtonClass::Text)
}

/// The text variant of a button.
pub struct Text {
    pub(super) leading_icon: Option<icon::Handle>,
    pub(super) trailing_icon: Option<icon::Handle>,
}

impl Default for Text {
    fn default() -> Self {
        Self::new()
    }
}

impl Text {
    pub const fn new() -> Self {
        Self {
            leading_icon: None,
            trailing_icon: None,
        }
    }
}

impl<Message> Button<'_, Message> {
    pub fn new(text: Text) -> Self {
        let guard = crate::theme::THEME.lock().unwrap();
        let theme = guard.cosmic();
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
            class: ButtonClass::Standard,
            variant: text,
        }
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
        let trailing_icon = builder
            .variant
            .trailing_icon
            .map(crate::widget::icon::Handle::icon);

        let leading_icon = builder
            .variant
            .leading_icon
            .map(crate::widget::icon::Handle::icon);

        let label: Option<Element<'_, _>> = (!builder.label.is_empty()).then(|| {
            let font = crate::font::Font {
                weight: builder.font_weight,
                ..crate::font::default()
            };

            // TODO: Avoid allocation
            crate::widget::text(builder.label.to_string())
                .size(builder.font_size)
                .line_height(LineHeight::Absolute(builder.line_height.into()))
                .font(font)
                .into()
        });

        let mut button: super::Button<'a, Message> = row::with_capacity(3)
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
            .align_y(Alignment::Center)
            .apply(super::custom)
            .padding(0)
            .id(builder.id)
            .on_press_maybe(builder.on_press.take())
            .class(builder.class);

        #[cfg(feature = "a11y")]
        {
            if !builder.label.is_empty() {
                button = button.name(builder.label);
            }
        }

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
