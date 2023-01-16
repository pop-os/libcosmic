// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::borrow::Cow;

use iced::{alignment, widget, Alignment, Background, Color, Length};

use crate::{theme, Element, Renderer, Theme};

use super::icon;

#[must_use]
pub fn warning<'a, Message>(message: impl Into<Cow<'a, str>>) -> Warning<'a, Message> {
    Warning {
        message: message.into(),
        on_close: None,
    }
}

pub struct Warning<'a, Message> {
    message: Cow<'a, str>,
    on_close: Option<Message>,
}

impl<'a, Message: 'static + Clone> Warning<'a, Message> {
    /// The message to emit on button press.
    #[must_use]
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// A custom button that has the desired default spacing and padding.
    pub fn into_widget(self) -> widget::Container<'static, Message, Renderer> {
        let close_button =
            widget::button(icon("window-close-symbolic", 16).style(theme::Svg::Default))
                .style(theme::Button::Transparent);

        let close_button = if let Some(message) = self.on_close {
            close_button.on_press(message)
        } else {
            close_button
        };

        widget::container(
            widget::row(vec![
                widget::container(widget::text(self.message))
                    .width(Length::Fill)
                    .into(),
                close_button.into(),
            ])
            .align_items(Alignment::Center),
        )
        .style(theme::Container::Custom(warning_container))
        .padding(10)
        .align_y(alignment::Vertical::Center)
        .width(Length::Fill)
    }
}

impl<'a, Message: 'static + Clone> From<Warning<'a, Message>> for Element<'a, Message> {
    fn from(warning: Warning<'a, Message>) -> Self {
        Self::from(warning.into_widget())
    }
}

pub fn warning_container(theme: &Theme) -> widget::container::Appearance {
    widget::container::Appearance {
        text_color: Some(theme.cosmic().on_warning_color().into()),
        background: Some(Background::Color(theme.cosmic().warning_color().into())),
        border_radius: 0.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }
}
