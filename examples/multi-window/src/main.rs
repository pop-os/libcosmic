// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod window;
pub use window::*;

pub fn main() -> cosmic::iced::Result {
    cosmic::app::run::<MultiWindow>(Default::default(), ())
}
