// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::Element;
use crate::widget::list_column::IntoListItem;
use crate::widget::{ListColumn, column, list_column, text};
use std::borrow::Cow;

/// A section within a settings view column.
pub fn section<'a, Message: Clone + 'static>() -> Section<'a, Message> {
    with_column(ListColumn::default())
}

/// A section with a pre-defined list column of a given capacity.
pub fn with_capacity<'a, Message: Clone + 'static>(capacity: usize) -> Section<'a, Message> {
    with_column(list_column::with_capacity(capacity))
}

/// A section with a pre-defined list column.
pub fn with_column<Message: Clone + 'static>(
    children: ListColumn<'_, Message>,
) -> Section<'_, Message> {
    Section {
        header: None,
        children,
    }
}

#[must_use]
pub struct Section<'a, Message> {
    header: Option<Element<'a, Message>>,
    children: ListColumn<'a, Message>,
}

impl<'a, Message: Clone + 'static> Section<'a, Message> {
    /// Define an optional title for the section.
    pub fn title(self, title: impl Into<Cow<'a, str>>) -> Self {
        self.header(text::heading(title.into()))
    }

    /// Define an optional custom header for the section.
    pub fn header(mut self, header: impl Into<Element<'a, Message>>) -> Self {
        self.header = Some(header.into());
        self
    }

    /// Add a child element to the section's list column.
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, item: impl IntoListItem<'a, Message>) -> Self {
        self.children = self.children.add(item);
        self
    }

    /// Add a child element to the section's list column, if `Some`.
    pub fn add_maybe(self, item: Option<impl IntoListItem<'a, Message>>) -> Self {
        if let Some(item) = item {
            self.add(item)
        } else {
            self
        }
    }

    /// Extends the [`Section`] with the given children.
    pub fn extend(
        self,
        children: impl IntoIterator<Item = impl IntoListItem<'a, Message>>,
    ) -> Self {
        children.into_iter().fold(self, Self::add)
    }
}

impl<'a, Message: Clone + 'static> From<Section<'a, Message>> for Element<'a, Message> {
    fn from(data: Section<'a, Message>) -> Self {
        column::with_capacity(2)
            .spacing(8)
            .push_maybe(data.header)
            .push(data.children)
            .into()
    }
}
