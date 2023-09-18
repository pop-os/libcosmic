// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Change the apperance of a button.
use iced_core::{Background, BorderRadius, Color, Vector};

/// The appearance of a button.
#[must_use]
#[derive(Debug, Clone, Copy)]
pub struct Appearance {
    /// The amount of offset to apply to the shadow of the button.
    pub shadow_offset: Vector,

    /// The [`Background`] of the button.
    pub background: Option<Background>,

    /// The border radius of the button.
    pub border_radius: BorderRadius,

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

impl Appearance {
    // TODO: `BorderRadius` is not `const fn` compatible.
    pub fn new() -> Self {
        Self {
            shadow_offset: Vector::new(0.0, 0.0),
            background: None,
            border_radius: BorderRadius::from(0.0),
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            outline_width: 0.0,
            outline_color: Color::TRANSPARENT,
            icon_color: None,
            text_color: None,
        }
    }
}

impl std::default::Default for Appearance {
    fn default() -> Self {
        Self::new()
    }
}

/// A set of rules that dictate the style of a button.
pub trait StyleSheet {
    /// The supported style of the [`StyleSheet`].
    type Style: Default;

    /// Produces the active [`Appearance`] of a button.
    fn active(&self, focused: bool, style: &Self::Style) -> Appearance;

    /// Produces the disabled [`Appearance`] of a button.
    fn disabled(&self, style: &Self::Style) -> Appearance;

    /// [`Appearance`] when the button is the target of a DND operation.
    fn drop_target(&self, style: &Self::Style) -> Appearance {
        self.hovered(false, style)
    }

    /// Produces the hovered [`Appearance`] of a button.
    fn hovered(&self, focused: bool, style: &Self::Style) -> Appearance;

    /// Produces the pressed [`Appearance`] of a button.
    fn pressed(&self, focused: bool, style: &Self::Style) -> Appearance;
}
