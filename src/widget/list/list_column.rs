// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::widget::container::Catalog;
use crate::widget::space::vertical;
use crate::widget::{button, column, container, divider, row};
use crate::{Apply, Element, theme};
use iced::{Length, Padding};

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

pub enum ListItem<'a, Message> {
    Element(Element<'a, Message>),
    Button(ListButton<'a, Message>),
}

/// A trait for types that can be added to a [`ListColumn`].
pub trait IntoListItem<'a, Message> {
    fn into_list_item(self) -> ListItem<'a, Message>;
}

impl<'a, Message, T> IntoListItem<'a, Message> for T
where
    T: Into<Element<'a, Message>>,
{
    fn into_list_item(self) -> ListItem<'a, Message> {
        ListItem::Element(self.into())
    }
}

impl<'a, Message> IntoListItem<'a, Message> for ListButton<'a, Message> {
    fn into_list_item(self) -> ListItem<'a, Message> {
        ListItem::Button(self)
    }
}

// Snapshots the padding values at the moment an item is added
struct ListEntry<'a, Message> {
    item: ListItem<'a, Message>,
    item_padding: Padding,
    divider_padding: u16,
}

#[must_use]
pub struct ListColumn<'a, Message> {
    list_item_padding: Padding,
    divider_padding: u16,
    style: theme::Container<'a>,
    children: Vec<ListEntry<'a, Message>>,
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
        divider_padding: 0,
        style: theme::Container::List,
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

    /// Adds a [`ListItem`] to the [`ListColumn`].
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, item: impl IntoListItem<'a, Message>) -> Self {
        self.children.push(ListEntry {
            item: item.into_list_item(),
            item_padding: self.list_item_padding,
            divider_padding: self.divider_padding,
        });
        self
    }

    /// Sets the style variant of this [`ListColumn`].
    #[inline]
    pub fn style(mut self, style: <crate::Theme as Catalog>::Class<'a>) -> Self {
        self.style = style;
        self
    }

    pub fn list_item_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.list_item_padding = padding.into();
        self
    }

    #[inline]
    pub fn divider_padding(mut self, padding: u16) -> Self {
        self.divider_padding = padding;
        self
    }

    #[must_use]
    pub fn into_element(self) -> Element<'a, Message> {
        let count = self.children.len();
        let last_index = count.saturating_sub(1);
        let radius_s = theme::active().cosmic().radius_s();
        let mut col = column::with_capacity((2 * count).saturating_sub(1));

        // Ensure minimum height of 32
        let content_row = |content| {
            row![container(content), vertical().height(32)].align_y(iced::Alignment::Center)
        };

        for (
            i,
            ListEntry {
                item,
                item_padding,
                divider_padding,
            },
        ) in self.children.into_iter().enumerate()
        {
            if i > 0 {
                col = col
                    .push(container(divider::horizontal::default()).padding([0, divider_padding]));
            }

            col = match item {
                ListItem::Element(content) => col.push(
                    content_row(content)
                        .padding(item_padding)
                        .width(Length::Fill),
                ),
                ListItem::Button(ListButton {
                    content,
                    on_press,
                    selected,
                }) => col.push(
                    content_row(content)
                        .apply(button::custom)
                        .padding(item_padding)
                        .width(Length::Fill)
                        .on_press_maybe(on_press)
                        .selected(selected)
                        .class(theme::Button::ListItem(get_radius(
                            radius_s,
                            i == 0,
                            i == last_index,
                        ))),
                ),
            };
        }

        col.width(Length::Fill)
            .apply(container)
            .class(self.style)
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
        (true, false) => [radius[0], radius[1], 0.0, 0.0],
        (false, true) => [0.0, 0.0, radius[2], radius[3]],
        (false, false) => [0.0, 0.0, 0.0, 0.0],
    }
}
