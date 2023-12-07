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
    let mut settings = Settings::default();
    settings.window.min_size = Some(cosmic::iced::Size::new(600., 300.));
    Window::run(settings)
}
