// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced::Border;
use iced_core::{Background, Color};

/// Appearance of the segmented button.
#[derive(Default, Clone, Copy)]
pub struct Appearance {
    pub background: Option<Background>,
    pub border: Border,
    pub active_width: f32,
    pub active: ItemStatusAppearance,
    pub inactive: ItemStatusAppearance,
    pub hover: ItemStatusAppearance,
    pub pressed: ItemStatusAppearance,
}

/// Appearance of an item in the segmented button.
#[derive(Default, Clone, Copy)]
pub struct ItemAppearance {
    pub border: Border,
}

/// Appearance of an item based on its status.
#[derive(Default, Clone, Copy)]
pub struct ItemStatusAppearance {
    pub background: Option<Background>,
    pub first: ItemAppearance,
    pub middle: ItemAppearance,
    pub last: ItemAppearance,
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
