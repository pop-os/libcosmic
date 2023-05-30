// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub use iced::Font;
use iced::{
    font::{load, Error},
    Command,
};
use iced_core::font::Family;

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

pub fn load_fonts() -> Command<Result<(), Error>> {
    Command::batch(vec![
        load(FONT_DATA),
        load(FONT_LIGHT_DATA),
        load(FONT_SEMIBOLD_DATA),
    ])
}
