// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod column;

pub use self::column::{list_column, ListColumn};

use crate::widget::Container;
use crate::Element;

pub fn container<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> Container<'a, Message, crate::Theme, crate::Renderer> {
    super::container(content)
        .padding([16, 6])
        .style(crate::theme::Container::List)
        .width(iced::Length::Fill)
}
