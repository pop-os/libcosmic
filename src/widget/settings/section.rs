// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::widget::{column, text, ListColumn};
use crate::Element;
use std::borrow::Cow;

/// A section within a settings view column.
#[deprecated(note = "use `settings::section().title()` instead")]
pub fn view_section<'a, Message: 'static>(title: impl Into<Cow<'a, str>>) -> Section<'a, Message> {
    Section {
        title: title.into(),
        children: ListColumn::default(),
    }
}

/// A section within a settings view column.
pub fn section<'a, Message: 'static>() -> Section<'a, Message> {
    with_column(ListColumn::default())
}

/// A section with a pre-defined list column.
pub fn with_column<Message: 'static>(children: ListColumn<'_, Message>) -> Section<'_, Message> {
    Section {
        title: Cow::Borrowed(""),
        children,
    }
}

#[must_use]
pub struct Section<'a, Message> {
    title: Cow<'a, str>,
    children: ListColumn<'a, Message>,
}

impl<'a, Message: 'static> Section<'a, Message> {
    /// Define an optional title for the section.
    pub fn title(mut self, title: impl Into<Cow<'a, str>>) -> Self {
        self.title = title.into();
        self
    }

    /// Add a child element to the section's list column.
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, item: impl Into<Element<'a, Message>>) -> Self {
        self.children = self.children.add(item.into());
        self
    }

    /// Add a child element to the section's list column, if `Some`.
    pub fn add_maybe(self, item: Option<impl Into<Element<'a, Message>>>) -> Self {
        if let Some(item) = item {
            self.add(item)
        } else {
            self
        }
    }

    /// Extends the [`Section`] with the given children.
    pub fn extend(
        self,
        children: impl IntoIterator<Item = impl Into<Element<'a, Message>>>,
    ) -> Self {
        children.into_iter().fold(self, Self::add)
    }
}

impl<'a, Message: 'static> From<Section<'a, Message>> for Element<'a, Message> {
    fn from(data: Section<'a, Message>) -> Self {
        column::with_capacity(2)
            .spacing(8)
            .push_maybe(if data.title.is_empty() {
                None
            } else {
                Some(text::heading(data.title))
            })
            .push(data.children)
            .into()
    }
}
