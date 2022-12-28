// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced_core::{Background, BorderRadius, Color};

/// The appearance of a segmented button.
#[derive(Clone, Copy)]
pub struct Appearance {
    pub background: Option<Background>,
    pub border_color: Color,
    pub border_radius: BorderRadius,
    pub border_width: f32,
    pub button_active: ButtonAppearance,
    pub button_inactive: ButtonAppearance,
    pub button_hover: ButtonAppearance,
}

/// The appearance of a button in the segmented button
#[derive(Clone, Copy)]
pub struct ButtonAppearance {
    pub background: Option<Background>,
    pub border_bottom: Option<(f32, Color)>,
    pub border_radius_first: BorderRadius,
    pub border_radius_middle: BorderRadius,
    pub border_radius_last: BorderRadius,
    pub text_color: Color,
}

/// Defines the [`Appearance`] of a segmented button.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// The [`Appearance`] of the segmented button.
    fn appearance(&self, style: &Self::Style) -> Appearance;
}
