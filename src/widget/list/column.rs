// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use apply::Apply;
use crate::{Element, theme};
use crate::widget::horizontal_rule;
use iced::{Background, Color};

#[must_use]
pub fn list_column<'a, Message: 'static>() -> ListColumn<'a, Message> {
    ListColumn::default()
}

pub struct ListColumn<'a, Message> {
    children: Vec<Element<'a, Message>>,
}

impl<'a, Message: 'static> Default for ListColumn<'a, Message> {
    fn default() -> Self {
        Self { children: Vec::with_capacity(4) }
    }
}

impl<'a, Message: 'static> ListColumn<'a, Message> {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn add(mut self, item: impl Into<Element<'a, Message>>) -> Self {
        if !self.children.is_empty() {
            self.children.push(horizontal_rule(12).into());
        }

        self.children.push(item.into());
        self
    }

    #[must_use]
    pub fn into_element(self) -> Element<'a, Message> {
        iced::widget::column(self.children)
            .spacing(12)
            .apply(iced::widget::container)
            .padding([12, 16])
            .style(theme::Container::Custom(style))
            .into()
    }
}

impl<'a, Message: 'static> From<ListColumn<'a, Message>> for Element<'a, Message> {
    fn from(column: ListColumn<'a, Message>) -> Self {
        column.into_element()
    }
}

fn style(theme: &crate::Theme) -> iced::widget::container::Appearance {
    let cosmic = &theme.cosmic().primary;
    iced::widget::container::Appearance {
        text_color: Some(cosmic.on.into()),
        background: Some(Background::Color(cosmic.base.into())),
        border_radius: 8.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }
}