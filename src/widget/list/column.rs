// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{widget::divider, Apply, Element};

#[must_use]
pub fn list_column<'a, Message: 'static>() -> ListColumn<'a, Message> {
    ListColumn::default()
}

pub struct ListColumn<'a, Message> {
    children: Vec<Element<'a, Message>>,
}

impl<'a, Message: 'static> Default for ListColumn<'a, Message> {
    fn default() -> Self {
        Self {
            children: Vec::with_capacity(4),
        }
    }
}

impl<'a, Message: 'static> ListColumn<'a, Message> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, item: impl Into<Element<'a, Message>>) -> Self {
        if !self.children.is_empty() {
            self.children.push(divider::horizontal::light().into());
        }

        self.children.push(item.into());
        self
    }

    #[must_use]
    pub fn into_element(self) -> Element<'a, Message> {
        crate::widget::column::with_children(self.children)
            .spacing(12)
            .apply(super::container)
            .into()
    }
}

impl<'a, Message: 'static> From<ListColumn<'a, Message>> for Element<'a, Message> {
    fn from(column: ListColumn<'a, Message>) -> Self {
        column.into_element()
    }
}
