// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::theme::{self, Button, Container};
use crate::widget::button::Catalog as ButtonCatalog;
use crate::widget::{button, column, container, row, space::vertical};
use crate::{Apply, Element};
use iced_core::Padding;
use iced_widget::container::Catalog;

#[inline]
pub fn list_column<'a, Message: 'static>() -> ListColumn<'a, Message> {
    ListColumn::default()
}

enum ListItem<'a, Message> {
    Element(Element<'a, Message>),
    Button {
        content: Element<'a, Message>,
        on_press: Option<Message>,
    },
}

#[must_use]
pub struct ListColumn<'a, Message> {
    list_item_padding: Padding,
    children: Vec<ListItem<'a, Message>>,
}

impl<Message: 'static> Default for ListColumn<'_, Message> {
    fn default() -> Self {
        let cosmic_theme::Spacing {
            space_xxs, space_m, ..
        } = theme::spacing();

        Self {
            list_item_padding: [space_xxs, space_m].into(),
            children: Vec::with_capacity(4),
        }
    }
}

impl<'a, Message: Clone + 'static> ListColumn<'a, Message> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a generic element to the list column.
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, item: impl Into<Element<'a, Message>>) -> Self {
        self.children.push(ListItem::Element(item.into()));
        self
    }

    /// Adds a button element to the list column.
    /// Only use for elements which have a single on_press/toggle-like action.
    pub fn add_button(
        mut self,
        item: impl Into<Element<'a, Message>>,
        on_press: impl Into<Option<Message>>,
    ) -> Self {
        self.children.push(ListItem::Button {
            content: item.into(),
            on_press: on_press.into(),
        });
        self
    }

    pub fn list_item_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.list_item_padding = padding.into();
        self
    }

    #[must_use]
    pub fn into_element(self) -> Element<'a, Message> {
        let padding = self.list_item_padding;
        let last_index = self.children.len().saturating_sub(1);
        let radius_s = theme::active().cosmic().radius_s();

        let content: Vec<Element<Message>> = self
            .children
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let is_first = i == 0;
                let is_last = i == last_index;
                let radius = get_radius(radius_s, is_first, is_last).into();

                match item {
                    ListItem::Element(element) => {
                        // Ensure a minimum height of 32
                        row![
                            container(element),
                            vertical().height(iced::Length::Fixed(32.))
                        ]
                        .align_y(iced::Alignment::Center)
                        .padding(padding)
                        .width(iced::Length::Fill)
                        .apply(container)
                        .class(Container::custom(move |theme| {
                            let mut style = theme.style(&Container::List);
                            style.border.radius = radius;
                            style
                        }))
                        .into()
                    }
                    ListItem::Button {
                        content: element,
                        on_press,
                    } => {
                        // Ensure a minimum height of 32
                        row![
                            container(element),
                            vertical().height(iced::Length::Fixed(32.))
                        ]
                        .align_y(iced::Alignment::Center)
                        .apply(button::custom)
                        .padding(padding)
                        .width(iced::Length::Fill)
                        .on_press_maybe(on_press)
                        .class(Button::Custom {
                            active: Box::new(move |focused, theme| {
                                let mut s = theme.active(focused, false, &Button::ListItem);
                                s.border_radius = radius;
                                s
                            }),
                            hovered: Box::new(move |focused, theme| {
                                let mut s = theme.hovered(focused, false, &Button::ListItem);
                                s.border_radius = radius;
                                s
                            }),
                            pressed: Box::new(move |focused, theme| {
                                let mut s = theme.pressed(focused, false, &Button::ListItem);
                                s.border_radius = radius;
                                s
                            }),
                            disabled: Box::new(move |theme| {
                                let mut s = theme.disabled(&Button::ListItem);
                                s.border_radius = radius;
                                s
                            }),
                        })
                        .into()
                    }
                }
            })
            .collect();

        column::with_children(content)
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
