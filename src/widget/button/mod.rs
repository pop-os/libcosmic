// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub use crate::theme::Button as Style;

pub mod link;
use derive_setters::Setters;
pub use link::link;
pub use link::Button as LinkButton;

mod icon;
pub use icon::icon;
pub use icon::Button as IconButton;

mod image;
pub use image::image;
pub use image::Button as ImageButton;

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
#[derive(Setters)]
pub struct Builder<'a, Message, Variant> {
    /// Sets the [`Id`] of the button.
    id: Id,

    /// The label to display within the button.
    #[setters(into)]
    label: Cow<'a, str>,

    // Adds a tooltip to the button.
    #[setters(into)]
    tooltip: Cow<'a, str>,

    /// Sets the message that will be produced when the button is pressed.
    ///
    /// If `None`, the button will be disabled.
    #[setters(strip_option)]
    on_press: Option<Message>,

    /// Sets the preferred width of the button.
    #[setters(into)]
    width: Length,

    /// Sets the preferred height of the button.
    #[setters(into)]
    height: Length,

    /// Sets the preferred padding of the button.
    #[setters(into)]
    padding: Padding,

    /// Sets the preferred spacing between elements in the button.
    spacing: u16,

    /// Sets the preferred size of icons.
    icon_size: u16,

    /// Sets the prefered font line height.
    line_height: u16,

    /// Sets the preferred font size.
    font_size: u16,

    /// Sets the preferred font weight.
    font_weight: Weight,

    /// The preferred style of the button.
    style: Style,

    #[setters(skip)]
    variant: Variant,
}

impl<'a, Message, Variant> Builder<'a, Message, Variant> {
    /// Set the value of [`on_press`] as either `Some` or `None`.
    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        self.on_press = on_press;
        self
    }
}
