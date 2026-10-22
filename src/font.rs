// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Select preferred fonts.

pub use iced::Font;
use iced_core::font::Weight;

use crate::cosmic_theme::FontConfig;

/// The font used by the active theme.
#[inline]
pub fn default() -> Font {
    let theme = crate::theme::active();
    Font::from(&theme.cosmic().font)
}

#[inline]
pub fn light() -> Font {
    Font {
        weight: Weight::Light,
        ..default()
    }
}

#[inline]
pub fn semibold() -> Font {
    Font {
        weight: Weight::Semibold,
        ..default()
    }
}

#[inline]
pub fn bold() -> Font {
    Font {
        weight: Weight::Bold,
        ..default()
    }
}

/// The monospace font used by the active theme.
#[inline]
pub fn mono() -> Font {
    let theme = crate::theme::active();
    Font::from(&theme.cosmic().mono_font)
}

/// Convert a theme font configuration into an [`iced::Font`].
#[must_use]
pub fn from_config(font: &FontConfig) -> Font {
    Font::from(font)
}
