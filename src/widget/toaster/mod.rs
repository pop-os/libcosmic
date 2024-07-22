// Copyright 2024 wiiznokes
// SPDX-License-Identifier: MPL-2.0

//! A widget that displays toasts.

use std::collections::VecDeque;
use std::time::Duration;

use crate::app::Command;
use crate::widget::container;
use crate::widget::Column;
use iced_core::Element;
use widget::Toaster;

use crate::ext::CollectionWidget;

use super::column;
use super::{button, icon, row, text};

mod widget;

/// Create a new Toaster widget.
pub fn toaster<'a, Message>(
    toasts: &'a Toasts<Message>,
    content: impl Into<Element<'a, Message, crate::Theme, iced::Renderer>>,
) -> Element<'a, Message, crate::Theme, iced::Renderer>
where
    Message: From<ToastMessage> + Clone + 'static,
{
    let theme = crate::theme::active();
    let cosmic_theme::Spacing {
        space_xxxs,
        space_xxs,
        space_xs,
        space_s,
        space_m,
        ..
    } = theme.cosmic().spacing;

    let make_toast = |toast: &'a Toast<Message>| {
        let row = row()
            .push(text(&toast.message))
            .push(
                row()
                    .push_maybe(toast.action.as_ref().map(|action| {
                        button::text(&action.description).on_press(action.message.clone())
                    }))
                    .push(
                        button::icon(icon::from_name("window-close-symbolic"))
                            .on_press(ToastMessage(toast.id).into()),
                    )
                    .align_items(iced::Alignment::Center)
                    .spacing(space_xxs),
            )
            .align_items(iced::Alignment::Center)
            .spacing(space_s);

        container(row)
            .padding([space_xxs, space_s, space_xxs, space_m])
            .style(crate::style::Container::Tooltip)
    };

    let col = toasts
        .toasts
        .iter()
        .rev()
        .map(make_toast)
        .fold(column::with_capacity(toasts.toasts.len()), Column::push)
        .spacing(space_xxxs);

    Toaster::new(col.into(), content.into(), toasts.toasts.is_empty()).into()
}

/// Duration for the [`Toast`]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum ToastDuration {
    #[default]
    Short,
    Long,
    Custom(Duration),
}

impl ToastDuration {
    fn duration(&self) -> Duration {
        match self {
            ToastDuration::Short => Duration::from_millis(2000),
            ToastDuration::Long => Duration::from_millis(3500),
            ToastDuration::Custom(duration) => *duration,
        }
    }
}

impl From<Duration> for ToastDuration {
    fn from(value: Duration) -> Self {
        Self::Custom(value)
    }
}

/// Action that can be triggered by the user.
///
/// Example: `undo`
#[derive(Debug, Clone)]
pub struct ToastAction<Message> {
    pub description: String,
    pub message: Message,
}

/// Represent the data used to display a [`Toast`]
#[derive(Debug, Clone)]
pub struct Toast<Message> {
    message: String,
    action: Option<ToastAction<Message>>,
    duration: ToastDuration,
    id: u32,
}

impl<Message> Toast<Message> {
    /// Construct a new [`Toast`] with the provided message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            action: None,
            duration: ToastDuration::default(),
            id: 0,
        }
    }

    /// Set the [`ToastAction`] of this [`Toast`]
    #[must_use]
    pub fn action(mut self, action: ToastAction<Message>) -> Self {
        self.action.replace(action);
        self
    }

    /// Set the [`ToastDuration`] of this [`Toast`]
    #[must_use]
    pub fn duration(mut self, duration: impl Into<ToastDuration>) -> Self {
        self.duration = duration.into();
        self
    }
}

#[derive(Debug, Clone)]
pub struct Toasts<Message> {
    id_count: u32,
    toasts: VecDeque<Toast<Message>>,
    limit: usize,
}

// need custom impl to not require Message: Clone
impl<M> Default for Toasts<M> {
    fn default() -> Self {
        Self {
            id_count: 0,
            toasts: VecDeque::new(),
            limit: 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ToastMessage(u32);

impl<Message> Toasts<Message> {
    /// Add a new [`Toast`]
    pub fn push(&mut self, mut toast: Toast<Message>) -> Command<Message>
    where
        Message: From<ToastMessage>,
    {
        while self.toasts.len() >= self.limit {
            self.toasts.pop_front();
        }

        toast.id = self.id_count;
        self.id_count += 1;

        let message = ToastMessage(toast.id);
        let duration = toast.duration.duration();

        self.toasts.push_back(toast);

        crate::command::future(async move {
            #[cfg(feature = "tokio")]
            tokio::time::sleep(duration).await;
            crate::app::Message::App(Message::from(message))
        })
    }

    /// Handle the [`ToastMessage`]
    pub fn handle_message(&mut self, message: &ToastMessage) {
        self.toasts
            .iter()
            .position(|e| e.id == message.0)
            .map(|index| self.toasts.remove(index));
    }
}
