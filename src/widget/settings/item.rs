// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::borrow::Cow;

use crate::{widget::text, Element, Renderer};
use derive_setters::Setters;
use iced::widget::{column, horizontal_space, row, Row};

/// A settings item aligned in a row
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

/// Creates a builder for an item, beginning with the title.
pub fn builder<'a, Message: 'static>(title: impl Into<Cow<'a, str>>) -> Item<'a, Message> {
    Item {
        title: title.into(),
        description: None,
        icon: None,
    }
}

/// A builder for a settings item.
#[derive(Setters)]
pub struct Item<'a, Message> {
    /// Describes the item being controlled.
    title: Cow<'a, str>,

    /// A description to display beneath the title.
    #[setters(strip_option, into)]
    description: Option<Cow<'a, str>>,

    /// A custom icon to display before the text.
    #[setters(strip_option, into)]
    icon: Option<Element<'a, Message>>,
}

impl<'a, Message: 'static> Item<'a, Message> {
    /// Assigns a control to the item.
    pub fn control(self, widget: impl Into<Element<'a, Message>>) -> Row<'a, Message, Renderer> {
        let mut contents = Vec::with_capacity(4);

        if let Some(icon) = self.icon {
            contents.push(icon);
        }

        if let Some(description) = self.description {
            let title = text(self.title);
            let desc = text(description).size(10);

            contents.push(column!(title, desc).spacing(2).into());
        } else {
            contents.push(text(self.title).into());
        }

        contents.push(horizontal_space(iced::Length::Fill).into());
        contents.push(widget.into());

        item_row(contents)
    }

    pub fn toggler(
        self,
        is_checked: bool,
        message: impl Fn(bool) -> Message + 'static,
    ) -> Row<'a, Message, Renderer> {
        self.control(crate::widget::toggler(None, is_checked, message))
    }
}
