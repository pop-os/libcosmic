// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced::{Length, widget};
use iced_core::text;

pub fn toggler<'a, Message, Theme: iced_widget::toggler::Catalog, Renderer>(
    is_checked: bool,
) -> widget::Toggler<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer + text::Renderer,
{
    widget::Toggler::new(is_checked)
        .size(24)
        .spacing(0)
        .width(Length::Shrink)
}
