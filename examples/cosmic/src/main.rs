// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use cosmic::{app::Settings, iced::Limits};

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
    let settings = Settings::default()
        .default_font(cosmic::font::default())
        .size_limits(Limits::NONE.min_width(600.).min_height(300.));
    cosmic::app::run::<window::Window>(settings, ())?;
    Ok(())
}
