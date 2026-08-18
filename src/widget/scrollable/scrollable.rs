// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::sync::Mutex;
use crate::{Element, Renderer};
use iced::widget;
pub static AUTO_SCROLLING: Mutex<bool> = Mutex::new(false);

pub fn scrollable<'a, Message>(
    element: impl Into<Element<'a, Message>>,
) -> widget::Scrollable<'a, Message, crate::Theme, Renderer> {
    vertical(element)
}

pub fn vertical<'a, Message>(
    element: impl Into<Element<'a, Message>>,
) -> widget::Scrollable<'a, Message, crate::Theme, Renderer> {
    let guard = AUTO_SCROLLING.lock().unwrap();
    let auto_scroll = *guard;
    widget::scrollable(element)
        .scroller_width(8.0)
        .scrollbar_width(8.0)
        .scrollbar_padding(8.0)
        .auto_scroll(auto_scroll)
}

pub fn horizontal<'a, Message>(
    element: impl Into<Element<'a, Message>>,
) -> widget::Scrollable<'a, Message, crate::Theme, Renderer> {
    let guard = AUTO_SCROLLING.lock().unwrap();
    let auto_scroll = *guard;
    widget::scrollable(element)
        .direction(widget::scrollable::Direction::Horizontal(
            widget::scrollable::Scrollbar::new(),
        ))
        .scroller_width(8.0)
        .scrollbar_width(8.0)
        .auto_scroll(auto_scroll)
}
