// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced_core::{Background, Color};

/// Appearance of the cards.
#[derive(Clone, Copy)]
pub struct Appearance {
    pub card_1: Background,
    pub card_2: Background,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            card_1: Background::Color(Color::WHITE),
            card_2: Background::Color(Color::WHITE),
        }
    }
}

/// Defines the [`Appearance`] of a cards.
pub trait StyleSheet {
    /// The default [`Appearance`] of the cards.
    fn default(&self) -> Appearance;
}
