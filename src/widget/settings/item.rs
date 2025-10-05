// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::borrow::Cow;

use crate::{
    Element, theme,
    widget::{FlexRow, Row, column, container, flex_row, horizontal_space, row, text},
};
use derive_setters::Setters;
use iced_core::{Length, text::Wrapping};
use taffy::AlignContent;

/// A settings item aligned in a row
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn item<'a, Message: 'static>(
    title: impl Into<Cow<'a, str>> + 'a,
    widget: impl Into<Element<'a, Message>> + 'a,
) -> Row<'a, Message> {
    #[inline(never)]
    fn inner<'a, Message: 'static>(
        title: Cow<'a, str>,
        widget: Element<'a, Message>,
    ) -> Row<'a, Message> {
        item_row(vec![
            text(title).wrapping(Wrapping::Word).into(),
            horizontal_space().into(),
            widget,
        ])
    }

    inner(title.into(), widget.into())
}

/// A settings item aligned in a row
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn item_row<Message>(children: Vec<Element<Message>>) -> Row<Message> {
    row::with_children(children)
        .spacing(theme::spacing().space_xs)
        .align_y(iced::Alignment::Center)
}

/// A settings item aligned in a flex row
#[allow(clippy::module_name_repetitions)]
pub fn flex_item<'a, Message: 'static>(
    title: impl Into<Cow<'a, str>> + 'a,
    widget: impl Into<Element<'a, Message>> + 'a,
) -> FlexRow<'a, Message> {
    #[inline(never)]
    fn inner<'a, Message: 'static>(
        title: Cow<'a, str>,
        widget: Element<'a, Message>,
    ) -> FlexRow<'a, Message> {
        flex_item_row(vec![
            text(title)
                .wrapping(Wrapping::Word)
                .width(Length::Fill)
                .into(),
            container(widget).into(),
        ])
    }

    inner(title.into(), widget.into())
}

/// A settings item aligned in a flex row
#[allow(clippy::module_name_repetitions)]
pub fn flex_item_row<Message>(children: Vec<Element<Message>>) -> FlexRow<Message> {
    flex_row(children)
        .spacing(theme::spacing().space_xs)
        .min_item_width(200.0)
        .justify_items(iced::Alignment::Center)
        .justify_content(AlignContent::SpaceBetween)
        .width(Length::Fill)
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
    pub fn control(self, widget: impl Into<Element<'a, Message>>) -> Row<'a, Message> {
        item_row(self.control_(widget.into()))
    }

    /// Assigns a control which flexes.
    pub fn flex_control(self, widget: impl Into<Element<'a, Message>>) -> FlexRow<'a, Message> {
        flex_item_row(self.control_(widget.into()))
    }

    #[inline(never)]
    fn control_(self, widget: Element<'a, Message>) -> Vec<Element<'a, Message>> {
        let mut contents = Vec::with_capacity(4);

        if let Some(icon) = self.icon {
            contents.push(icon);
        }

        if let Some(description) = self.description {
            let column = column::with_capacity(2)
                .spacing(2)
                .push(text::body(self.title).wrapping(Wrapping::Word))
                .push(text::caption(description).wrapping(Wrapping::Word))
                .width(Length::Fill);

            contents.push(column.into());
        } else {
            contents.push(text(self.title).width(Length::Fill).into());
        }

        contents.push(widget);
        contents
    }

    pub fn toggler(
        self,
        is_checked: bool,
        message: impl Fn(bool) -> Message + 'static,
    ) -> Row<'a, Message> {
        self.control(crate::widget::toggler(is_checked).on_toggle(message))
    }
}
