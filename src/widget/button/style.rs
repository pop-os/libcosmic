// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Change the apperance of a button.
use iced_core::{Background, Color, Vector, border::Radius};

use crate::theme::THEME;

/// The appearance of a button.
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The amount of offset to apply to the shadow of the button.
    pub shadow_offset: Vector,

    /// The [`Background`] of the button.
    pub background: Option<Background>,

    /// The [`Background`] overlay of the button.
    pub overlay: Option<Background>,

    /// The border radius of the button.
    pub border_radius: Radius,

    /// The border width of the button.
    pub border_width: f32,

    /// The border [`Color`] of the button.
    pub border_color: Color,

    /// An outline placed around the border.
    pub outline_width: f32,

    /// The [`Color`] of the outline.
    pub outline_color: Color,

    /// The icon [`Color`] of the button.
    pub icon_color: Option<Color>,

    /// The text [`Color`] of the button.
    pub text_color: Option<Color>,
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
            text_color: None,
            overlay: None,
        }
    }
}

impl std::default::Default for Style {
    fn default() -> Self {
        Self::new()
    }
}

// TODO update to match other styles
/// A set of rules that dictate the style of a button.
pub trait Catalog {
    /// The supported style of the [`StyleSheet`].
    type Class: Default;

    /// Produces the active [`Appearance`] of a button.
    fn active(&self, focused: bool, selected: bool, style: &Self::Class) -> Style;

    /// Produces the disabled [`Appearance`] of a button.
    fn disabled(&self, style: &Self::Class) -> Style;

    /// [`Appearance`] when the button is the target of a DND operation.
    fn drop_target(&self, style: &Self::Class) -> Style {
        self.hovered(false, false, style)
    }

    /// Produces the hovered [`Appearance`] of a button.
    fn hovered(&self, focused: bool, selected: bool, style: &Self::Class) -> Style;

    /// Produces the pressed [`Appearance`] of a button.
    fn pressed(&self, focused: bool, selected: bool, style: &Self::Class) -> Style;

    /// Background color of the selection indicator
    fn selection_background(&self) -> Background;
}
