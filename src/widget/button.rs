// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{theme, Element, Renderer};
use iced::widget;

/// A button widget with COSMIC styling
#[must_use]
pub const fn button<Message>(style: theme::Button) -> Button<Message> {
    Button { style, message: None }
}

/// A button widget with COSMIC styling
pub struct Button<Message> {
    style: theme::Button,
    message: Option<Message>,
}

impl<Message: 'static> Button<Message> {
    /// The message to emit on button press.
    #[must_use]
    pub fn on_press(mut self, message: Message) -> Self {
        self.message = Some(message);
        self
    }

    /// A button with an icon.
    pub fn icon(self, style: theme::Svg, icon: &str, size: u16) -> widget::Button<Message, Renderer> {
        self.custom(vec![super::icon(icon, size).style(style).into()])
    }

    /// A button with text.
    pub fn text(self, text: &str) -> widget::Button<Message, Renderer> {
        self.custom(vec![text.into()])
    }

    /// A custom button that has the desired default spacing and padding.
    pub fn custom(self, children: Vec<Element<Message>>) -> widget::Button<Message, Renderer> {
        let button = widget::button(widget::row(children).spacing(8))
            .style(self.style)
            .padding([8, 16]);

        if let Some(message) = self.message {
            button.on_press(message)
        } else {
            button
        }
    }
}

