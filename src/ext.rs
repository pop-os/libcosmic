// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced::Color;
use iced_core::Widget;

pub trait ElementExt {
    #[must_use]
    fn debug(self, debug: bool) -> Self;
}

impl<'a, Message: 'static> ElementExt for crate::Element<'a, Message> {
    fn debug(self, debug: bool) -> Self {
        if debug {
            self.explain(Color::WHITE)
        } else {
            self
        }
    }
}

/// Additional methods for the [`Column`] and [`Row`] widgets.
pub trait CollectionWidget<'a, Message: 'a>: Widget<Message, crate::Renderer>
where
    Self: Sized,
{
    /// Moves all the elements of `other` into `self`, leaving `other` empty.
    #[must_use]
    fn append<E>(self, other: &mut Vec<E>) -> Self
    where
        E: Into<crate::Element<'a, Message>>;

    /// Appends all elements in an iterator to the widget.
    #[must_use]
    fn extend<E>(mut self, iterator: impl Iterator<Item = E>) -> Self
    where
        E: Into<crate::Element<'a, Message>>,
    {
        for item in iterator {
            self = self.push(item.into());
        }

        self
    }

    /// Pushes an element into the widget.
    #[must_use]
    fn push(self, element: impl Into<crate::Element<'a, Message>>) -> Self;

    /// Conditionally pushes an element to the widget.
    #[must_use]
    fn push_maybe(self, element: Option<impl Into<crate::Element<'a, Message>>>) -> Self {
        if let Some(element) = element {
            self.push(element.into())
        } else {
            self
        }
    }
}

impl<'a, Message: 'a> CollectionWidget<'a, Message> for crate::widget::Column<'a, Message> {
    fn append<E>(self, other: &mut Vec<E>) -> Self
    where
        E: Into<crate::Element<'a, Message>>,
    {
        self.extend(other.drain(..))
    }

    fn push(self, element: impl Into<crate::Element<'a, Message>>) -> Self {
        self.push(element)
    }
}

impl<'a, Message: 'a> CollectionWidget<'a, Message> for crate::widget::Row<'a, Message> {
    fn append<E>(self, other: &mut Vec<E>) -> Self
    where
        E: Into<crate::Element<'a, Message>>,
    {
        self.extend(other.drain(..))
    }

    fn push(self, element: impl Into<crate::Element<'a, Message>>) -> Self {
        self.push(element)
    }
}
