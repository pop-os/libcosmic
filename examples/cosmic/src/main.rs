// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use cosmic::iced::{Application, Settings};

mod window;
use env_logger::Env;
pub use window::*;

pub fn main() -> cosmic::iced::Result {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "info")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
    cosmic::icon_theme::set_default("Pop");
    #[allow(clippy::field_reassign_with_default)]
    let settings = Settings {
        default_font: cosmic::font::default(),
        window: cosmic::iced::window::Settings {
            min_size: Some(cosmic::iced::Size::new(600., 300.)),
            ..cosmic::iced::window::Settings::default()
        },
        ..Settings::default()
    };
    Window::run(settings)
}
