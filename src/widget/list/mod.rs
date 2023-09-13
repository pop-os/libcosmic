// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod column;
// mod item;

pub use self::column::{list_column, ListColumn};
// pub use self::item::{ListItem, list_item};

use crate::widget::Container;
use crate::Element;
use iced::{Background, Color};

pub fn container<'a, Message>(
    content: impl Into<Element<'a, Message>>,
) -> Container<'a, Message, crate::Renderer> {
    super::container(content)
        .padding([16, 6])
        .style(crate::theme::Container::custom(style))
        .width(iced::Length::Fill)
}

#[must_use]
#[allow(clippy::trivially_copy_pass_by_ref)]
pub fn style(theme: &crate::Theme) -> crate::widget::container::Appearance {
    let container = &theme.current_container().component;
    crate::widget::container::Appearance {
        icon_color: Some(container.on.into()),
        text_color: Some(container.on.into()),
        background: Some(Background::Color(container.base.into())),
        border_radius: 8.0.into(),
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }
}
