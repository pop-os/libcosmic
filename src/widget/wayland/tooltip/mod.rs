//! Change the apperance of a tooltip.

pub mod widget;

// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced_core::{Background, Color, Vector, border::Radius};

use crate::theme::THEME;

/// The appearance of a tooltip.
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The amount of offset to apply to the shadow of the tooltip.
    pub shadow_offset: Vector,

    /// The [`Background`] of the tooltip.
    pub background: Option<Background>,

    /// The border radius of the tooltip.
    pub border_radius: Radius,

    /// The border width of the tooltip.
    pub border_width: f32,

    /// The border [`Color`] of the tooltip.
    pub border_color: Color,

    /// An outline placed around the border.
    pub outline_width: f32,

    /// The [`Color`] of the outline.
    pub outline_color: Color,

    /// The icon [`Color`] of the tooltip.
    pub icon_color: Option<Color>,

    /// The text [`Color`] of the tooltip.
    pub text_color: Color,
}

impl Style {
    // TODO: `Radius` is not `const fn` compatible.
    pub fn new() -> Self {
        let rad_0 = THEME.lock().unwrap().cosmic().corner_radii.radius_0;
        Self {
            shadow_offset: Vector::new(0.0, 0.0),
            background: None,
            border_radius: Radius::from(rad_0),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            outline_width: 0.0,
            outline_color: Color::TRANSPARENT,
            icon_color: None,
            text_color: Color::BLACK,
        }
    }
}

impl std::default::Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

// TODO update to match other styles
/// A set of rules that dictate the style of a tooltip.
pub trait Catalog {
    /// The supported style of the [`StyleSheet`].
    type Class: Default;

    /// Produces the active [`Appearance`] of a tooltip.
    fn style(&self, style: &Self::Class) -> Style;
}
