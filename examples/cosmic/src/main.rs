// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use cosmic::{iced::Application, settings};

mod window;
use env_logger::Env;
pub use window::*;

pub fn main() -> cosmic::iced::Result {
    let env = Env::default()
        .filter_or("MY_LOG_LEVEL", "debug")
        .write_style_or("MY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
    settings::set_default_icon_theme("Pop");
    let mut settings = settings();
    settings.window.min_size = Some((600, 300));
    Window::run(settings)
}
