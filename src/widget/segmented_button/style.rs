/// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced_core::{Background, BorderRadius, Color};

/// The appearance of a [`SegmentedButton`].
#[derive(Clone, Copy)]
pub struct Appearance {
    pub button_active: ButtonAppearance,
    pub button_inactive: ButtonAppearance,
}

/// The appearance of a button in the [`SegmentedButton`]
#[derive(Clone, Copy)]
pub struct ButtonAppearance {
    pub background: Option<Background>,
    pub border_radius: BorderRadius,
    pub border_bottom: Option<(f32, Color)>,
    pub text_color: Color,
}

/// Defines the [`Appearance`] of a [`SegmentedButton`].
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// The [`Appearance`] of the [`SegmentedButton`].
    fn appearance(&self, style: &Self::Style) -> Appearance;
}
