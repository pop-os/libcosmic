// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{Element, Renderer};
use iced::widget;

pub fn scrollable<'a, Message>(
    element: impl Into<Element<'a, Message>>,
) -> widget::Scrollable<'a, Message, crate::Theme, Renderer> {
    vertical(element)
}

pub fn vertical<'a, Message>(
    element: impl Into<Element<'a, Message>>,
) -> widget::Scrollable<'a, Message, crate::Theme, Renderer> {
    widget::scrollable(element)
        .scroller_width(8.0)
        .scrollbar_width(8.0)
        .scrollbar_padding(8.0)
}

pub fn horizontal<'a, Message>(
    element: impl Into<Element<'a, Message>>,
) -> widget::Scrollable<'a, Message, crate::Theme, Renderer> {
    widget::scrollable(element)
        .direction(widget::scrollable::Direction::Horizontal(
            widget::scrollable::Scrollbar::new(),
        ))
        .scroller_width(8.0)
        .scrollbar_width(8.0)
}
