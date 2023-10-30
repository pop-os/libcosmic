// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Select preferred fonts.

pub use iced::Font;
use iced::{
    font::{load, Error},
    Command,
};
use iced_core::font::Family;

pub const DEFAULT: Font = FONT;

pub const FONT: Font = Font {
    family: Family::Name("Fira Sans"),
    weight: iced_core::font::Weight::Normal,
    stretch: iced_core::font::Stretch::Normal,
    monospaced: false,
};

pub const FONT_DATA: &[u8] = include_bytes!("../res/Fira/FiraSans-Regular.otf");

pub const FONT_LIGHT: Font = Font {
    family: Family::Name("Fira Sans"),
    weight: iced_core::font::Weight::Light,
    stretch: iced_core::font::Stretch::Normal,
    monospaced: false,
};

pub const FONT_LIGHT_DATA: &[u8] = include_bytes!("../res/Fira/FiraSans-Light.otf");

pub const FONT_SEMIBOLD: Font = Font {
    family: Family::Name("Fira Sans"),
    weight: iced_core::font::Weight::Semibold,
    stretch: iced_core::font::Stretch::Normal,
    monospaced: false,
};

pub const FONT_SEMIBOLD_DATA: &[u8] = include_bytes!("../res/Fira/FiraSans-SemiBold.otf");

pub const FONT_BOLD: Font = Font {
    family: Family::Name("Fira Sans"),
    weight: iced_core::font::Weight::Bold,
    stretch: iced_core::font::Stretch::Normal,
    monospaced: false,
};

pub const FONT_BOLD_DATA: &[u8] = include_bytes!("../res/Fira/FiraSans-Bold.otf");

pub const FONT_MONO_REGULAR: Font = Font {
    family: Family::Name("Fira Mono"),
    weight: iced_core::font::Weight::Normal,
    stretch: iced_core::font::Stretch::Normal,
    monospaced: true,
};

pub const FONT_MONO_REGULAR_DATA: &[u8] = include_bytes!("../res/Fira/FiraMono-Regular.otf");

pub fn load_fonts() -> Command<Result<(), Error>> {
    Command::batch(vec![
        load(FONT_DATA),
        load(FONT_LIGHT_DATA),
        load(FONT_SEMIBOLD_DATA),
        load(FONT_BOLD_DATA),
        load(FONT_MONO_REGULAR_DATA),
    ])
}
