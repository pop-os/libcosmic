// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced_core::{Background, Color};

/// Appearance of the color picker.
#[derive(Clone, Copy)]
pub struct Appearance {}

impl Default for Appearance {
    fn default() -> Self {
        Self {}
    }
}

/// Defines the [`Appearance`] of a color picker.
pub trait StyleSheet {
    /// The default [`Appearance`] of color picker.
    fn default(&self) -> Appearance;
}
