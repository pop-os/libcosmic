// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use cosmic::{iced::Application, settings};

mod window;
pub use window::*;

pub fn main() -> cosmic::iced::Result {
    let mut settings = settings();
    settings.window.min_size = Some((600, 300));
    // TODO: Window resize handles not functioning yet
    settings.window.decorations = true;
    Window::run(settings)
}
