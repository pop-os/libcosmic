// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Navigation side panel for switching between views.
//!
//! For details on the model, see the [`segmented_button`] module for more details.

use apply::Apply;
use iced::{
    widget::{container, scrollable},
    Background, Length,
};
use iced_core::Color;

use crate::{theme, widget::segmented_button, Theme};

/// Navigation side panel for switching between views.
///
/// For details on the model, see the [`segmented_button`] module for more details.
pub fn nav_bar<Message>(
    model: &segmented_button::SingleSelectModel,
    on_activate: fn(segmented_button::Entity) -> Message,
) -> iced::widget::Container<Message, crate::Renderer>
where
    Message: Clone + 'static,
{
    segmented_button::vertical(model)
        .button_height(32)
        .button_padding([16, 10, 16, 10])
        .button_spacing(8)
        .icon_size(16)
        .on_activate(on_activate)
        .spacing(8)
        .style(crate::theme::SegmentedButton::ViewSwitcher)
        .apply(scrollable)
        .apply(container)
        .height(Length::Fill)
        .padding(11)
        .style(theme::Container::custom(nav_bar_style))
}

#[must_use]
pub fn nav_bar_style(theme: &Theme) -> iced_style::container::Appearance {
    let cosmic = &theme.cosmic();
    iced_style::container::Appearance {
        text_color: Some(cosmic.on_bg_color().into()),
        background: Some(Background::Color(cosmic.primary.base.into())),
        border_radius: 8.0.into(),
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
    }
}
