// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Button widgets for COSMIC applications.

pub use crate::theme::Button as Style;

pub mod link;
use derive_setters::Setters;
#[doc(inline)]
pub use link::link;
#[doc(inline)]
pub use link::Button as LinkButton;

mod icon;
#[doc(inline)]
pub use icon::icon;
#[doc(inline)]
pub use icon::Button as IconButton;

mod image;
#[doc(inline)]
pub use image::image;
#[doc(inline)]
pub use image::Button as ImageButton;

mod style;
#[doc(inline)]
pub use style::{Appearance, StyleSheet};

mod text;
#[doc(inline)]
pub use text::Button as TextButton;
#[doc(inline)]
pub use text::{destructive, standard, suggested, text};

mod widget;
#[doc(inline)]
pub use widget::{draw, focus, layout, mouse_interaction, Button};

use iced_core::font::Weight;
use iced_core::widget::Id;
use iced_core::{Length, Padding};
use std::borrow::Cow;

/// A button with a custom element for its content.
pub fn custom<'a, Message>(content: impl Into<crate::Element<'a, Message>>) -> Button<'a, Message> {
    Button::new(content)
}

/// An image button which may contain any widget as its content.
pub fn custom_image_button<'a, Message>(
    content: impl Into<crate::Element<'a, Message>>,
    on_remove: Option<Message>,
) -> Button<'a, Message> {
    Button::new_image(content, on_remove)
}

/// A builder for constructing a custom [`Button`].
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
