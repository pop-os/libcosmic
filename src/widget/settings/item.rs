// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{Element, Renderer};

/// A setting within a settings view section.
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn item<'a, Message: 'static>(title: &'a str, widget: impl Into<Element<'a, Message>>) -> iced::widget::Row<'a, Message, Renderer> {
    item_row(vec![
        iced::widget::text(title).into(),
        iced::widget::horizontal_space(iced::Length::Fill).into(),
        widget.into()
    ])
}

/// A settings item aligned in a row
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn item_row<Message>(children: Vec<Element<Message>>) -> iced::widget::Row<Message, Renderer> {
    iced::widget::row(children)
        .align_items(iced::Alignment::Center)
        .padding([0, 8])
        .spacing(12)
}

