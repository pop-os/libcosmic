// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced_core::Padding;
use iced_style::container::StyleSheet;

use crate::{widget::divider, Apply, Element};

pub fn list_column<'a, Message: 'static>() -> ListColumn<'a, Message> {
    ListColumn::default()
}

#[must_use]
pub struct ListColumn<'a, Message> {
    spacing: u16,
    padding: Padding,
    style: <crate::Theme as StyleSheet>::Style,
    children: Vec<Element<'a, Message>>,
}

impl<'a, Message: 'static> Default for ListColumn<'a, Message> {
    fn default() -> Self {
        Self {
            spacing: 12,
            padding: Padding::from(0),
            style: <crate::Theme as StyleSheet>::Style::List,
            children: Vec::with_capacity(4),
        }
    }
}

impl<'a, Message: 'static> ListColumn<'a, Message> {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, item: impl Into<Element<'a, Message>>) -> Self {
        if !self.children.is_empty() {
            self.children.push(divider::horizontal::light().into());
        }

        self.children.push(item.into());
        self
    }

    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the style variant of this [`Circular`].
    pub fn style(mut self, style: <crate::Theme as StyleSheet>::Style) -> Self {
        self.style = style;
        self
    }

    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    #[must_use]
    pub fn into_element(self) -> Element<'a, Message> {
        crate::widget::column::with_children(self.children)
            .spacing(self.spacing)
            .padding(self.padding)
            .apply(super::container)
            .padding([self.spacing, 0])
            .style(self.style)
            .into()
    }
}

impl<'a, Message: 'static> From<ListColumn<'a, Message>> for Element<'a, Message> {
    fn from(column: ListColumn<'a, Message>) -> Self {
        column.into_element()
    }
}
