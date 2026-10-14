// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{Element, Renderer};
use iced::widget::scrollable::{Direction, Scrollable, Scrollbar};

pub fn scrollable<'a, Message>(
    element: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message, crate::Theme, Renderer> {
    vertical(element)
}

pub fn vertical<'a, Message>(
    element: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message, crate::Theme, Renderer> {
    iced::widget::scrollable(element).direction(Direction::Vertical(
        Scrollbar::new().width(8).scroller_width(8).padding(8),
    ))
}

pub fn horizontal<'a, Message>(
    element: impl Into<Element<'a, Message>>,
) -> Scrollable<'a, Message, crate::Theme, Renderer> {
    iced::widget::scrollable(element).direction(Direction::Horizontal(
        Scrollbar::new().width(8).scroller_width(8),
    ))
}
