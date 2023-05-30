// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub use iced::Font;
use iced::{
    font::{load, Error},
    Command,
};

pub const FONT: Font = Font::with_name("Fira Sans Regular");
pub const FONT_DATA: &[u8] = include_bytes!("../res/Fira/FiraSans-Regular.otf");

pub const FONT_LIGHT: Font = Font::with_name("Fira Sans Light");
pub const FONT_LIGHT_DATA: &[u8] = include_bytes!("../res/Fira/FiraSans-Light.otf");

pub const FONT_SEMIBOLD: Font = Font::with_name("Fira Sans SemiBold");
pub const FONT_SEMIBOLD_DATA: &[u8] = include_bytes!("../res/Fira/FiraSans-SemiBold.otf");

pub fn load_fonts() -> Command<Result<(), Error>> {
    Command::batch(vec![
        load(FONT_DATA),
        load(FONT_LIGHT_DATA),
        load(FONT_SEMIBOLD_DATA),
    ])
}
