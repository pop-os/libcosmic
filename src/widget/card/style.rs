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

impl crate::widget::card::style::Catalog for crate::Theme {
    fn default(&self) -> crate::widget::card::style::Style {
        let cosmic = self.cosmic();

        match self.layer {
            cosmic_theme::Layer::Background => crate::widget::card::style::Style {
                card_1: Background::Color(cosmic.background.component.hover.into()),
                card_2: Background::Color(cosmic.background.component.pressed.into()),
            },
            cosmic_theme::Layer::Primary => crate::widget::card::style::Style {
                card_1: Background::Color(cosmic.primary.component.hover.into()),
                card_2: Background::Color(cosmic.primary.component.pressed.into()),
            },
            cosmic_theme::Layer::Secondary => crate::widget::card::style::Style {
                card_1: Background::Color(cosmic.secondary.component.hover.into()),
                card_2: Background::Color(cosmic.secondary.component.pressed.into()),
            },
        }
    }
}
