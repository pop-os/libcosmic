// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::borrow::Cow;

use crate::{
    Element, Theme, theme,
    widget::{FlexRow, Row, column, container, flex_row, list, row, text},
};
use derive_setters::Setters;
use iced_core::{Length, text::Wrapping};
use iced_widget::space;
use taffy::AlignContent;

/// A settings item aligned in a row
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn item<'a, Message: 'static>(
    title: impl Into<Cow<'a, str>> + 'a,
    widget: impl Into<Element<'a, Message>> + 'a,
) -> Row<'a, Message, Theme> {
    #[inline(never)]
    fn inner<'a, Message: 'static>(
        title: Cow<'a, str>,
        widget: Element<'a, Message>,
    ) -> Row<'a, Message, Theme> {
        item_row(vec![
            text(title).wrapping(Wrapping::Word).into(),
            space::horizontal().into(),
            widget,
        ])
    }

    inner(title.into(), widget.into())
}

/// A settings item aligned in a row
#[must_use]
#[allow(clippy::module_name_repetitions)]
pub fn item_row<Message>(children: Vec<Element<Message>>) -> Row<Message, Theme> {
    row::with_children(children)
        .spacing(theme::spacing().space_xs)
        .align_y(iced::Alignment::Center)
        .width(Length::Fill)
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
            container(widget).width(Length::Shrink).into(),
        ])
        .width(Length::Fill)
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

impl<'a, Message: Clone + 'static> Item<'a, Message> {
    /// Assigns a control to the item.
    pub fn control(self, widget: impl Into<Element<'a, Message>>) -> Row<'a, Message, Theme> {
        item_row(self.control_(widget.into()))
    }

    /// Assigns a control which flexes.
    pub fn flex_control(self, widget: impl Into<Element<'a, Message>>) -> FlexRow<'a, Message> {
        flex_item_row(self.control_(widget.into()))
    }

    fn label(self) -> Element<'a, Message> {
        if let Some(description) = self.description {
            column::with_capacity(2)
                .spacing(2)
                .push(text::body(self.title).wrapping(Wrapping::Word))
                .push(text::caption(description).wrapping(Wrapping::Word))
                .width(Length::Fill)
                .into()
        } else {
            text(self.title).width(Length::Fill).into()
        }
    }

    #[inline(never)]
    fn control_(mut self, widget: Element<'a, Message>) -> Vec<Element<'a, Message>> {
        let mut contents = Vec::with_capacity(3);
        if let Some(icon) = self.icon.take() {
            contents.push(icon);
        }
        contents.push(self.label());
        contents.push(widget);
        contents
    }

    fn control_start(self, widget: impl Into<Element<'a, Message>>) -> Row<'a, Message, Theme> {
        item_row(vec![widget.into(), self.label()])
    }

    pub fn toggler(
        self,
        is_checked: bool,
        message: impl Fn(bool) -> Message + 'static,
    ) -> list::ListButton<'a, Message> {
        let on_press = message(!is_checked);
        list::button(
            self.control(
                crate::widget::toggler(is_checked)
                    .width(Length::Shrink)
                    .on_toggle(message),
            ),
        )
        .on_press(on_press)
    }

    pub fn toggler_maybe(
        self,
        is_checked: bool,
        message: Option<impl Fn(bool) -> Message + 'static>,
    ) -> list::ListButton<'a, Message> {
        let on_press = message.as_ref().map(|f| f(!is_checked));
        list::button(
            self.control(
                crate::widget::toggler(is_checked)
                    .width(Length::Shrink)
                    .on_toggle_maybe(message),
            ),
        )
        .on_press_maybe(on_press)
    }

    pub fn checkbox(
        self,
        is_checked: bool,
        message: impl Fn(bool) -> Message + 'static,
    ) -> list::ListButton<'a, Message> {
        let on_press = message(!is_checked);
        list::button(
            self.control_start(
                crate::widget::checkbox(is_checked)
                    .width(Length::Shrink)
                    .on_toggle(message),
            ),
        )
        .on_press(on_press)
    }

    pub fn checkbox_maybe(
        self,
        is_checked: bool,
        message: Option<impl Fn(bool) -> Message + 'static>,
    ) -> list::ListButton<'a, Message> {
        let on_press = message.as_ref().map(|f| f(!is_checked));
        list::button(
            self.control_start(
                crate::widget::checkbox(is_checked)
                    .width(Length::Shrink)
                    .on_toggle_maybe(message),
            ),
        )
        .on_press_maybe(on_press)
    }

    pub fn radio<V, F>(self, value: V, selected: Option<V>, f: F) -> list::ListButton<'a, Message>
    where
        V: Eq + Copy + 'static,
        F: Fn(V) -> Message + 'static,
    {
        let on_press = f(value);
        list::button(
            self.control_start(crate::widget::radio::Radio::new_no_label(
                value, selected, f,
            )),
        )
        .on_press(on_press)
    }
}
