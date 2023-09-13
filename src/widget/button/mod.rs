// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub use crate::theme::Button as Style;

pub mod link;
pub use link::link;
pub use link::Button as LinkButton;

mod icon;
pub use icon::icon;
pub use icon::Button as IconButton;

mod style;
pub use style::{Appearance, StyleSheet};

mod text;
pub use text::Button as TextButton;
pub use text::{destructive, standard, suggested, text};

mod widget;
pub use widget::{draw, focus, layout, mouse_interaction, Button};

use crate::Element;
use iced_core::font::Weight;
use iced_core::widget::Id;
use iced_core::{Length, Padding};
use std::borrow::Cow;

pub fn button<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> Button<'a, Message, crate::Renderer> {
    Button::new(content)
}

#[must_use]
pub struct Builder<'a, Message, Variant> {
    id: Id,
    label: Cow<'a, str>,
    tooltip: Cow<'a, str>,
    on_press: Option<Message>,
    width: Length,
    height: Length,
    padding: Padding,
    spacing: u16,
    icon_size: u16,
    line_height: u16,
    font_size: u16,
    font_weight: Weight,
    style: Style,
    variant: Variant,
}

impl<'a, Message, Variant> Builder<'a, Message, Variant> {
    /// Sets the [`Id`] of the button.
    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    pub fn label(mut self, label: impl Into<Cow<'a, str>>) -> Self {
        self.label = label.into();
        self
    }

    /// Sets the width of the button.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the button.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Padding`] of the button.
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the message that will be produced when the button is pressed.
    ///
    /// Unless `on_press` is called, the button will be disabled.
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(on_press);
        self
    }

    /// Sets the message that will be produced when the button is pressed,
    /// if `Some`.
    ///
    /// If `None`, the button will be disabled.
    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        self.on_press = on_press;
        self
    }

    /// Overrides the preferred style of the button.
    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    /// Adds a tooltip to the button.
    pub fn tooltip(mut self, tooltip: impl Into<Cow<'a, str>>) -> Self {
        self.tooltip = tooltip.into();
        self
    }
}
