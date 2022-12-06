// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::Renderer;
use iced::{widget, Length};

pub fn toggler<'a, Message>(
    label: impl Into<Option<String>>,
    is_checked: bool,
    f: impl Fn(bool) -> Message + 'a,
) -> widget::Toggler<'a, Message, Renderer> {
    widget::Toggler::new(is_checked, label, f)
        .size(24)
        .spacing(12)
        .width(Length::Shrink)
}
