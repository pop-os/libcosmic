// Copyright 2023 System76 <info@system76.com>
// Copyright 2019 Héctor Ramón, Iced contributors
// SPDX-License-Identifier: MPL-2.0 AND MIT

//! Change the appearance of menus.
use iced_core::{Background, Color, border::Radius};

/// The appearance of a menu.
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// Menu text color
    pub text_color: Color,
    /// Menu background
    pub background: Background,
    /// Menu border width
    pub border_width: f32,
    /// Menu border radius
    pub border_radius: Radius,
    /// Menu border color
    pub border_color: Color,
    /// Text color when hovered
    pub hovered_text_color: Color,
    /// Background when hovered
    pub hovered_background: Background,
    /// Text color when selected
    pub selected_text_color: Color,
    /// Background when selected
    pub selected_background: Background,
    /// Description text color
    pub description_color: Color,
}

/// The style sheet of a menu.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default + Clone;

    /// Produces the [`Appearance`] of a menu.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}
