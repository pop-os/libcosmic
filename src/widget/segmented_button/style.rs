// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced_core::{Background, BorderRadius, Color};

/// The appearance of a segmented button.
#[derive(Default, Clone, Copy)]
pub struct Appearance {
    pub background: Option<Background>,
    pub border_radius: BorderRadius,
    pub border_bottom: Option<(f32, Color)>,
    pub border_end: Option<(f32, Color)>,
    pub border_start: Option<(f32, Color)>,
    pub border_top: Option<(f32, Color)>,
    pub active: ButtonStatusAppearance,
    pub inactive: ButtonStatusAppearance,
    pub hover: ButtonStatusAppearance,
    pub focus: ButtonStatusAppearance,
}

/// The appearance of a button in the segmented button
#[derive(Default, Clone, Copy)]
pub struct ButtonAppearance {
    pub border_radius: BorderRadius,
    pub border_bottom: Option<(f32, Color)>,
    pub border_end: Option<(f32, Color)>,
    pub border_start: Option<(f32, Color)>,
    pub border_top: Option<(f32, Color)>,
}

#[derive(Default, Clone, Copy)]
pub struct ButtonStatusAppearance {
    pub background: Option<Background>,
    pub first: ButtonAppearance,
    pub middle: ButtonAppearance,
    pub last: ButtonAppearance,
    pub text_color: Color,
}

/// Defines the [`Appearance`] of a segmented button.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// The horizontal [`Appearance`] of the segmented button.
    fn horizontal(&self, style: &Self::Style) -> Appearance;

    /// The vertical [`Appearance`] of the segmented button.
    fn vertical(&self, style: &Self::Style) -> Appearance;
}
