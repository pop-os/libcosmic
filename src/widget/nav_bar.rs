// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Navigation side panel for switching between views.
//!
//! For details on the model, see the [`segmented_button`] module for more details.

use apply::Apply;
use iced::{
    clipboard::{dnd::DndAction, mime::AllowedMimeTypes},
    widget::{container, scrollable},
    Background, Length,
};
use iced_core::{Border, Color, Shadow};

use crate::{theme, widget::segmented_button, Theme};

use super::dnd_destination::DragId;

pub type Id = segmented_button::Entity;
pub type Model = segmented_button::SingleSelectModel;

/// Navigation side panel for switching between views.
///
/// For details on the model, see the [`segmented_button`] module for more details.
pub fn nav_bar<Message>(
    model: &segmented_button::SingleSelectModel,
    on_activate: fn(segmented_button::Entity) -> Message,
) -> iced::widget::Container<Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'static,
{
    let theme = crate::theme::active();
    let space_s = theme.cosmic().space_s();
    let space_xxs = theme.cosmic().space_xxs();

    segmented_button::vertical(model)
        .button_height(32)
        .button_padding([space_s, space_xxs, space_s, space_xxs])
        .button_spacing(space_xxs)
        .spacing(space_xxs)
        .on_activate(on_activate)
        .style(crate::theme::SegmentedButton::TabBar)
        .apply(scrollable)
        .height(Length::Fill)
        .apply(container)
        .padding(space_xxs)
        .height(Length::Fill)
        .style(theme::Container::custom(nav_bar_style))
}

/// Navigation side panel for switching between views.
/// Can receive drag and drop events.
pub fn nav_bar_dnd<Message, D: AllowedMimeTypes>(
    model: &segmented_button::SingleSelectModel,
    on_activate: fn(segmented_button::Entity) -> Message,
    on_dnd_enter: impl Fn(segmented_button::Entity, Vec<String>) -> Message + 'static,
    on_dnd_leave: impl Fn(segmented_button::Entity) -> Message + 'static,
    on_dnd_drop: impl Fn(segmented_button::Entity, Option<D>, DndAction) -> Message + 'static,
    id: DragId,
) -> iced::widget::Container<Message, crate::Theme, crate::Renderer>
where
    Message: Clone + 'static,
{
    let theme = crate::theme::active();
    let space_s = theme.cosmic().space_s();
    let space_xxs = theme.cosmic().space_xxs();

    let nav_buttons = segmented_button::vertical(model)
        .button_height(32)
        .button_padding([space_s, space_xxs, space_s, space_xxs])
        .button_spacing(space_xxs)
        .spacing(space_xxs)
        .on_activate(on_activate)
        .style(crate::theme::SegmentedButton::TabBar)
        .on_dnd_enter(on_dnd_enter)
        .on_dnd_leave(on_dnd_leave)
        .on_dnd_drop(on_dnd_drop)
        .drag_id(id);

    nav_buttons
        .apply(scrollable)
        .height(Length::Fill)
        .apply(container)
        .padding(space_xxs)
        .height(Length::Fill)
        .style(theme::Container::custom(nav_bar_style))
}

#[must_use]
pub fn nav_bar_style(theme: &Theme) -> iced_style::container::Appearance {
    let cosmic = &theme.cosmic();
    iced_style::container::Appearance {
        icon_color: Some(cosmic.on_bg_color().into()),
        text_color: Some(cosmic.on_bg_color().into()),
        background: Some(Background::Color(cosmic.primary.base.into())),
        border: Border {
            width: 0.0,
            color: Color::TRANSPARENT,
            radius: cosmic.corner_radii.radius_s.into(),
        },
        shadow: Shadow::default(),
    }
}
