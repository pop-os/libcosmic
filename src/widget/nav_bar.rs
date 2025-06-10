// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Navigation side panel for switching between views.
//!
//! For details on the model, see the [`segmented_button`] module for more details.

use apply::Apply;
use iced::{
    Background, Length,
    clipboard::{dnd::DndAction, mime::AllowedMimeTypes},
};
use iced_core::{Border, Color, Shadow};

use crate::widget::{Container, Icon, container, menu, scrollable, segmented_button};
use crate::{Theme, theme};

use super::dnd_destination::DragId;

pub type Id = segmented_button::Entity;
pub type Model = segmented_button::SingleSelectModel;

/// Navigation side panel for switching between views.
///
/// For details on the model, see the [`segmented_button`] module for more details.
pub fn nav_bar<Message: Clone + 'static>(
    model: &segmented_button::SingleSelectModel,
    on_activate: fn(segmented_button::Entity) -> Message,
) -> NavBar<Message> {
    NavBar {
        segmented_button: segmented_button::vertical(model).on_activate(on_activate),
    }
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
) -> NavBar<Message>
where
    Message: Clone + 'static,
{
    NavBar {
        segmented_button: segmented_button::vertical(model)
            .on_activate(on_activate)
            .on_dnd_enter(on_dnd_enter)
            .on_dnd_leave(on_dnd_leave)
            .on_dnd_drop(on_dnd_drop)
            .drag_id(id),
    }
}

#[must_use]
pub struct NavBar<'a, Message> {
    segmented_button:
        segmented_button::VerticalSegmentedButton<'a, segmented_button::SingleSelect, Message>,
}

impl<'a, Message: Clone + 'static> NavBar<'a, Message> {
    #[inline]
    pub fn close_icon(mut self, close_icon: Icon) -> Self {
        self.segmented_button = self.segmented_button.close_icon(close_icon);
        self
    }

    #[inline]
    pub fn context_menu(mut self, context_menu: Option<Vec<menu::Tree<Message>>>) -> Self {
        self.segmented_button = self.segmented_button.context_menu(context_menu);
        self
    }

    #[inline]
    pub fn drag_id(mut self, id: DragId) -> Self {
        self.segmented_button = self.segmented_button.drag_id(id);
        self
    }

    /// Pre-convert this widget into the [`Container`] widget that it becomes.
    #[must_use]
    #[inline]
    pub fn into_container(self) -> Container<'a, Message, crate::Theme, crate::Renderer> {
        Container::from(self)
    }

    /// Emitted when a tab close button is pressed.
    pub fn on_close<T>(mut self, on_close: T) -> Self
    where
        T: Fn(Id) -> Message + 'static,
    {
        self.segmented_button = self.segmented_button.on_close(on_close);
        self
    }

    /// Emitted when a button is right-clicked.
    pub fn on_context<T>(mut self, on_context: T) -> Self
    where
        T: Fn(Id) -> Message + 'static,
    {
        self.segmented_button = self.segmented_button.on_context(on_context);
        self
    }

    /// Emitted when the middle mouse button is pressed on a button.
    pub fn on_middle_press<T>(mut self, on_middle_press: T) -> Self
    where
        T: Fn(Id) -> Message + 'static,
    {
        self.segmented_button = self.segmented_button.on_middle_press(on_middle_press);
        self
    }

    /// Handle the dnd drop event.
    pub fn on_dnd_drop<D: AllowedMimeTypes>(
        mut self,
        handler: impl Fn(Id, Option<D>, DndAction) -> Message + 'static,
    ) -> Self {
        self.segmented_button = self.segmented_button.on_dnd_drop(handler);
        self
    }

    /// Handle the dnd enter event.
    pub fn on_dnd_enter(mut self, handler: impl Fn(Id, Vec<String>) -> Message + 'static) -> Self {
        self.segmented_button = self.segmented_button.on_dnd_enter(handler);
        self
    }

    /// Handle the dnd leave event.
    pub fn on_dnd_leave(mut self, handler: impl Fn(Id) -> Message + 'static) -> Self {
        self.segmented_button = self.segmented_button.on_dnd_leave(handler);
        self
    }
}

impl<'a, Message: Clone + 'static> From<NavBar<'a, Message>>
    for Container<'a, Message, crate::Theme, crate::Renderer>
{
    fn from(this: NavBar<'a, Message>) -> Self {
        let theme = crate::theme::active();
        let space_s = theme.cosmic().space_s();
        let space_xxs = theme.cosmic().space_xxs();

        this.segmented_button
            .button_height(32)
            .button_padding([space_s, space_xxs, space_s, space_xxs])
            .button_spacing(space_xxs)
            .spacing(space_xxs)
            .style(crate::theme::SegmentedButton::TabBar)
            .apply(container)
            .padding(space_xxs)
            .apply(scrollable)
            .class(crate::style::iced::Scrollable::Minimal)
            .height(Length::Fill)
            .apply(container)
            .height(Length::Fill)
            .class(theme::Container::custom(nav_bar_style))
    }
}

impl<'a, Message: Clone + 'static> From<NavBar<'a, Message>> for crate::Element<'a, Message> {
    fn from(this: NavBar<'a, Message>) -> Self {
        Container::from(this).into()
    }
}

#[must_use]
pub fn nav_bar_style(theme: &Theme) -> iced_widget::container::Style {
    let cosmic = &theme.cosmic();
    iced_widget::container::Style {
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
