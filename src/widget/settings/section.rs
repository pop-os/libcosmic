// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::widget::ListColumn;
use crate::Element;
use iced::widget::{column, text};
use std::borrow::Cow;

/// A section within a settings view column.
#[must_use]
pub fn view_section<'a, Message: 'static>(title: impl Into<Cow<'a, str>>) -> Section<'a, Message> {
    Section {
        title: title.into(),
        children: ListColumn::default(),
    }
}

pub struct Section<'a, Message> {
    title: Cow<'a, str>,
    children: ListColumn<'a, Message>,
}

impl<'a, Message: 'static> Section<'a, Message> {
    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, item: impl Into<Element<'a, Message>>) -> Self {
        self.children = self.children.add(item.into());
        self
    }
}

impl<'a, Message: 'static> From<Section<'a, Message>> for Element<'a, Message> {
    fn from(data: Section<'a, Message>) -> Self {
        let title = text(data.title).font(crate::font::FONT_SEMIBOLD).into();

        column(vec![title, data.children.into_element()])
            .spacing(8)
            .into()
    }
}
