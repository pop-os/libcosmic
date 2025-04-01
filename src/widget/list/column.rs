// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced_core::Padding;
use iced_widget::container::Catalog;

use crate::{
    Apply, Element, theme,
    widget::{container, divider, vertical_space},
};

#[inline]
pub fn list_column<'a, Message: 'static>() -> ListColumn<'a, Message> {
    ListColumn::default()
}

#[must_use]
pub struct ListColumn<'a, Message> {
    spacing: u16,
    padding: Padding,
    list_item_padding: Padding,
    divider_padding: u16,
    style: theme::Container<'a>,
    children: Vec<Element<'a, Message>>,
}

impl<Message: 'static> Default for ListColumn<'_, Message> {
    fn default() -> Self {
        let cosmic_theme::Spacing {
            space_xxs, space_m, ..
        } = theme::spacing();

        Self {
            spacing: 0,
            padding: Padding::from(0),
            divider_padding: 16,
            list_item_padding: [space_xxs, space_m].into(),
            style: theme::Container::List,
            children: Vec::with_capacity(4),
        }
    }
}

impl<'a, Message: 'static> ListColumn<'a, Message> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(clippy::should_implement_trait)]
    pub fn add(self, item: impl Into<Element<'a, Message>>) -> Self {
        #[inline(never)]
        fn inner<'a, Message: 'static>(
            mut this: ListColumn<'a, Message>,
            item: Element<'a, Message>,
        ) -> ListColumn<'a, Message> {
            if !this.children.is_empty() {
                this.children.push(
                    container(divider::horizontal::default())
                        .padding([0, this.divider_padding])
                        .into(),
                );
            }

            // Ensure a minimum height of 32.
            let list_item = iced::widget::row![
                container(item).align_y(iced::Alignment::Center),
                vertical_space().height(iced::Length::Fixed(32.))
            ]
            .padding(this.list_item_padding)
            .align_y(iced::Alignment::Center);

            this.children.push(list_item.into());
            this
        }

        inner(self, item.into())
    }

    #[inline]
    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the style variant of this [`Circular`].
    #[inline]
    pub fn style(mut self, style: <crate::Theme as Catalog>::Class<'a>) -> Self {
        self.style = style;
        self
    }

    #[inline]
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    #[inline]
    pub fn divider_padding(mut self, padding: u16) -> Self {
        self.divider_padding = padding;
        self
    }

    pub fn list_item_padding(mut self, padding: impl Into<Padding>) -> Self {
        self.list_item_padding = padding.into();
        self
    }

    #[must_use]
    pub fn into_element(self) -> Element<'a, Message> {
        crate::widget::column::with_children(self.children)
            .spacing(self.spacing)
            .padding(self.padding)
            .apply(container)
            .padding([self.spacing, 0])
            .class(self.style)
            .width(iced::Length::Fill)
            .into()
    }
}

impl<'a, Message: 'static> From<ListColumn<'a, Message>> for Element<'a, Message> {
    fn from(column: ListColumn<'a, Message>) -> Self {
        column.into_element()
    }
}
