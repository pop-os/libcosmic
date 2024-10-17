// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced_core::{Background, Color};

/// Appearance of the cards.
#[derive(Clone, Copy)]
pub struct Style {
    pub card_1: Background,
    pub card_2: Background,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            card_1: Background::Color(Color::WHITE),
            card_2: Background::Color(Color::WHITE),
        }
    }
}

/// Defines the [`Appearance`] of a cards.
pub trait Catalog {
    /// The default [`Appearance`] of the cards.
    fn default(&self) -> Style;
}
