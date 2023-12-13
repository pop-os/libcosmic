// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::icon;
use crate::{theme, widget, Element, Renderer, Theme};
use apply::Apply;
use iced::{alignment, Alignment, Background, Color, Length};
use std::borrow::Cow;

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
    pub fn into_widget(self) -> widget::Container<'a, Message, Renderer> {
        let label = widget::container(crate::widget::text(self.message)).width(Length::Fill);

        let close_button = icon::from_name("window-close-symbolic")
            .size(16)
            .apply(widget::button::icon)
            .on_press_maybe(self.on_close);

        widget::row::with_capacity(2)
            .push(label)
            .push(close_button)
            .align_items(Alignment::Center)
            .apply(widget::container)
            .style(theme::Container::custom(warning_container))
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

#[must_use]
pub fn warning_container(theme: &Theme) -> widget::container::Appearance {
    let cosmic = theme.cosmic();
    widget::container::Appearance {
        icon_color: Some(theme.cosmic().warning.on.into()),
        text_color: Some(theme.cosmic().warning.on.into()),
        background: Some(Background::Color(theme.cosmic().warning_color().into())),
        border_radius: cosmic.corner_radii.radius_0.into(),
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }
}
