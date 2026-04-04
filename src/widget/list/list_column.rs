// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::theme::{self, Button, Container};
use crate::widget::{button, column, container, row, space::vertical};
use crate::{Apply, Element};
use iced_core::Padding;
use iced_widget::container::Catalog;

/// A button list item for use in a [`ListColumn`].
pub struct ListButton<'a, Message> {
    content: Element<'a, Message>,
    on_press: Option<Message>,
    selected: bool,
}

/// Creates a [`ListButton`] with the given content.
pub fn button<'a, Message>(content: impl Into<Element<'a, Message>>) -> ListButton<'a, Message> {
    ListButton {
        content: content.into(),
        on_press: None,
        selected: false,
    }
}

impl<'a, Message: 'static> ListButton<'a, Message> {
    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(on_press);
        self
    }

    pub fn on_press_maybe(mut self, on_press: Option<Message>) -> Self {
        self.on_press = on_press;
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

enum ListItem<'a, Message> {
    Element(Element<'a, Message>),
    Button(ListButton<'a, Message>),
}

#[must_use]
pub struct ListColumn<'a, Message> {
    list_item_padding: Padding,
    children: Vec<ListItem<'a, Message>>,
}

#[inline]
pub fn list_column<'a, Message: 'static>() -> ListColumn<'a, Message> {
    ListColumn::default()
}

pub fn with_capacity<'a, Message: 'static>(capacity: usize) -> ListColumn<'a, Message> {
    let cosmic_theme::Spacing {
        space_xxs, space_m, ..
    } = theme::spacing();

    ListColumn {
        list_item_padding: [space_xxs, space_m].into(),
        children: Vec::with_capacity(capacity),
    }
}

impl<Message: 'static> Default for ListColumn<'_, Message> {
    fn default() -> Self {
        with_capacity(4)
    }
}

impl<'a, Message: Clone + 'static> ListColumn<'a, Message> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an element to the list column.
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, item: impl Into<Element<'a, Message>>) -> Self {
        self.children.push(ListItem::Element(item.into()));
        self
    }

    /// Adds a [`ListButton`] to the list column.
    pub fn add_button(mut self, item: ListButton<'a, Message>) -> Self {
        self.children.push(ListItem::Button(item));
        self
    }

    pub fn list_item_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.list_item_padding = padding.into();
        self
    }

    #[must_use]
    pub fn into_element(self) -> Element<'a, Message> {
        let padding = self.list_item_padding;
        let count = self.children.len();
        let last_index = count.saturating_sub(1);
        let radius_s = theme::active().cosmic().radius_s();

        self.children
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let is_first = i == 0;
                let is_last = i == last_index;
                let radius = get_radius(radius_s, is_first, is_last);

                match item {
                    ListItem::Element(content) => {
                        // Ensure a minimum height of 32
                        row![container(content), vertical().height(32)]
                            .align_y(iced::Alignment::Center)
                            .padding(padding)
                            .width(iced::Length::Fill)
                            .apply(container)
                            .class(Container::custom(move |theme| {
                                let mut style = theme.style(&Container::List);
                                style.border.radius = radius.into();
                                style
                            }))
                            .apply(Element::from)
                    }
                    ListItem::Button(ListButton {
                        content,
                        on_press,
                        selected,
                    }) => {
                        // Ensure a minimum height of 32
                        row![container(content), vertical().height(32)]
                            .align_y(iced::Alignment::Center)
                            .apply(button::custom)
                            .padding(padding)
                            .width(iced::Length::Fill)
                            .on_press_maybe(on_press)
                            .selected(selected)
                            .class(Button::ListItem(radius))
                            .apply(Element::from)
                    }
                }
            })
            .fold(column::with_capacity(count), |col, item| col.push(item))
            .spacing(2)
            .width(iced::Length::Fill)
            .into()
    }
}

impl<'a, Message: Clone + 'static> From<ListColumn<'a, Message>> for Element<'a, Message> {
    fn from(column: ListColumn<'a, Message>) -> Self {
        column.into_element()
    }
}

fn get_radius(radius: [f32; 4], first: bool, last: bool) -> [f32; 4] {
    match (first, last) {
        (true, true) => radius,
        (true, false) => [radius[0], radius[1], 2.0, 2.0],
        (false, true) => [2.0, 2.0, radius[2], radius[3]],
        (false, false) => [2.0, 2.0, 2.0, 2.0],
    }
}
