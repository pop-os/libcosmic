// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced::{widget, Length};
use iced_core::text;

pub fn toggler<'a, Message, Theme: iced_widget::toggler::StyleSheet, Renderer>(
    label: impl Into<Option<String>>,
    is_checked: bool,
    f: impl Fn(bool) -> Message + 'a,
) -> widget::Toggler<'a, Message, Theme, Renderer>
where
    Renderer: iced_core::Renderer + text::Renderer,
{
    widget::Toggler::new(label, is_checked, f)
        .size(24)
        .width(Length::Shrink)
}
