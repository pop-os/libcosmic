// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use apply::Apply;
use iced::{
    widget::{container, scrollable},
    Background, Length,
};
use iced_core::Color;

use crate::{theme, Theme};

use super::segmented_button::{self, cosmic::vertical_view_switcher};

/// A container holding a vertical view switcher with the n style
pub fn nav_bar<Data, Message>(
    state: &segmented_button::State<Data>,
    on_activate: impl Fn(segmented_button::Key) -> Message + 'static,
) -> iced::widget::Container<Message, crate::Renderer>
where
    Message: Clone + 'static,
{
    vertical_view_switcher(state)
        .on_activate(on_activate)
        .button_height(32)
        .button_padding([16, 10, 16, 10])
        .button_spacing(8)
        .icon_size(16)
        .spacing(14)
        .apply(scrollable)
        .apply(container)
        .height(Length::Fill)
        .padding(11)
        .style(theme::Container::Custom(nav_bar_style))
}

#[must_use]
pub fn nav_bar_style(theme: &Theme) -> iced_style::container::Appearance {
    let cosmic = &theme.cosmic().primary;
    iced_style::container::Appearance {
        text_color: Some(cosmic.on.into()),
        background: Some(Background::Color(cosmic.base.into())),
        border_radius: 8.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }
}
