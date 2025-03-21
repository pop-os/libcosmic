// Copyright 2019 H�ctor Ram�n, Iced contributors
// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MIT

//! Change the appearance of a text input.

use iced_core::{Background, Color, border::Radius};

/// The appearance of a text input.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The [`Background`] of the text input.
    pub background: Background,
    /// The border radius of the text input.
    pub border_radius: Radius,
    /// The border offset
    pub border_offset: Option<f32>,
    /// The border width of the text input.
    pub border_width: f32,
    /// The border [`Color`] of the text input.
    pub border_color: Color,
    /// The label [`Color`] of the text input.
    pub label_color: Color,
    /// The placeholder text [`Color`].
    pub placeholder_color: Color,
    /// The text [`Color`] of the text input.
    pub selected_text_color: Color,
    /// The icon [`Color`] of the text input.
    pub icon_color: Option<Color>,
    /// The text [`Color`] of the text input.
    pub text_color: Option<Color>,
    /// The selected fill [`Color`] of the text input.
    pub selected_fill: Color,
}

/// A set of rules that dictate the style of a text input.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the style of an active text input.
    fn active(&self, style: &Self::Style) -> Appearance;

    /// Produces the style of an errored text input.
    fn error(&self, style: &Self::Style) -> Appearance;

    /// Produces the style of a focused text input.
    fn focused(&self, style: &Self::Style) -> Appearance;

    /// Produces the style of an hovered text input.
    fn hovered(&self, style: &Self::Style) -> Appearance {
        self.focused(style)
    }

    /// Produces the style of a disabled text input.
    fn disabled(&self, style: &Self::Style) -> Appearance;
}
