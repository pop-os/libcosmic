// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Select preferred fonts.

pub use iced::Font;
use iced_core::font::Weight;

#[inline]
pub fn default() -> Font {
    Font::from(crate::config::interface_font())
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

#[inline]
pub fn mono() -> Font {
    Font::from(crate::config::monospace_font())
}
