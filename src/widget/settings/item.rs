// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::borrow::Cow;

use crate::{widget::text, Element, Renderer};
use iced::widget::{horizontal_space, row, Row};

/// A setting within a settings view section.
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn item<'a, Message: 'static>(
    title: impl Into<Cow<'a, str>>,
    widget: impl Into<Element<'a, Message>>,
) -> Row<'a, Message, Renderer> {
    item_row(vec![
        text(title).into(),
        horizontal_space(iced::Length::Fill).into(),
        widget.into(),
    ])
}

/// A settings item aligned in a row
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn item_row<Message>(children: Vec<Element<Message>>) -> Row<Message, Renderer> {
    row(children)
        .align_items(iced::Alignment::Center)
        .padding([0, 18])
        .spacing(12)
}
