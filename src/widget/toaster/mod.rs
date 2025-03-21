// Copyright 2024 wiiznokes
// SPDX-License-Identifier: MPL-2.0

//! A widget that displays toasts.

use std::collections::VecDeque;
use std::rc::Rc;

use crate::widget::Column;
use crate::widget::container;
use iced::Task;
use iced_core::Element;
use slotmap::SlotMap;
use slotmap::new_key_type;
use widget::Toaster;

use super::column;
use super::{button, icon, row, text};

mod widget;

/// Create a new Toaster widget.
pub fn toaster<'a, Message: Clone + 'static>(
    toasts: &'a Toasts<Message>,
    content: impl Into<Element<'a, Message, crate::Theme, iced::Renderer>>,
) -> Element<'a, Message, crate::Theme, iced::Renderer> {
    let theme = crate::theme::active();
    let cosmic_theme::Spacing {
        space_xxxs,
        space_xxs,
        space_s,
        space_m,
        ..
    } = theme.cosmic().spacing;

    let make_toast = move |(id, toast): (ToastId, &'a Toast<Message>)| {
        let row = row()
            .push(text(&toast.message))
            .push(
                row()
                    .push_maybe(toast.action.as_ref().map(|action| {
                        button::text(&action.description).on_press((action.message)(id))
                    }))
                    .push(
                        button::icon(icon::from_name("window-close-symbolic"))
                            .on_press((toasts.on_close)(id)),
                    )
                    .align_y(iced::Alignment::Center)
                    .spacing(space_xxs),
            )
            .align_y(iced::Alignment::Center)
            .spacing(space_s);

        container(row)
            .padding([space_xxs, space_s, space_xxs, space_m])
            .class(crate::style::Container::Tooltip)
    };

    let col = toasts
        .queue
        .iter()
        .filter_map(|id| Some((*id, toasts.toasts.get(*id)?)))
        .rev()
        .map(make_toast)
        .fold(column::with_capacity(toasts.toasts.len()), Column::push)
        .spacing(space_xxxs);

    Toaster::new(col.into(), content.into(), toasts.toasts.is_empty()).into()
}

/// Duration for the [`Toast`]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Duration {
    #[default]
    Short,
    Long,
    Custom(std::time::Duration),
}

impl Duration {
    #[cfg(feature = "tokio")]
    fn duration(&self) -> std::time::Duration {
        match self {
            Duration::Short => std::time::Duration::from_millis(5000),
            Duration::Long => std::time::Duration::from_millis(15000),
            Duration::Custom(duration) => *duration,
        }
    }
}

impl From<std::time::Duration> for Duration {
    fn from(value: std::time::Duration) -> Self {
        Self::Custom(value)
    }
}

/// Action that can be triggered by the user.
///
/// Example: `undo`
#[derive(Clone)]
pub struct Action<Message> {
    pub description: String,
    pub message: Rc<dyn Fn(ToastId) -> Message>,
}

impl<Message> std::fmt::Debug for Action<Message> {
    #[cold]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Action")
            .field("description", &self.description)
            .finish()
    }
}

/// Represent the data used to display a [`Toast`]
#[derive(Debug, Clone)]
pub struct Toast<Message> {
    message: String,
    action: Option<Action<Message>>,
    duration: Duration,
}

impl<Message> Toast<Message> {
    /// Construct a new [`Toast`] with the provided message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            action: None,
            duration: Duration::default(),
        }
    }

    /// Set the [`Action`] of this [`Toast`]
    #[must_use]
    pub fn action(
        mut self,
        description: String,
        message: impl Fn(ToastId) -> Message + 'static,
    ) -> Self {
        self.action.replace(Action {
            description,
            message: Rc::new(message),
        });
        self
    }

    /// Set the [`Duration`] of this [`Toast`]
    #[must_use]
    pub fn duration(mut self, duration: impl Into<Duration>) -> Self {
        self.duration = duration.into();
        self
    }
}

new_key_type! { pub struct ToastId; }

#[derive(Debug, Clone)]
pub struct Toasts<Message> {
    toasts: SlotMap<ToastId, Toast<Message>>,
    queue: VecDeque<ToastId>,
    on_close: fn(ToastId) -> Message,
    limit: usize,
}

impl<Message: Clone + Send + 'static> Toasts<Message> {
    pub fn new(on_close: fn(ToastId) -> Message) -> Self {
        let limit = 5;
        Self {
            toasts: SlotMap::with_capacity_and_key(limit),
            queue: VecDeque::new(),
            on_close,
            limit,
        }
    }

    /// Add a new [`Toast`]
    pub fn push(&mut self, toast: Toast<Message>) -> Task<Message> {
        while self.toasts.len() >= self.limit {
            self.toasts.remove(
                self.queue
                    .pop_front()
                    .expect("Queue must contain all toast ids"),
            );
        }

        #[cfg(feature = "tokio")]
        let duration = toast.duration.duration();

        let id = self.toasts.insert(toast);
        self.queue.push_back(id);

        #[cfg(feature = "tokio")]
        {
            let on_close = self.on_close;
            crate::task::future(async move {
                tokio::time::sleep(duration).await;
                on_close(id)
            })
        }
        #[cfg(not(feature = "tokio"))]
        {
            Task::none()
        }
    }

    /// Remove a [`Toast`]
    pub fn remove(&mut self, id: ToastId) {
        self.toasts.remove(id);
        if let Some(pos) = self.queue.iter().position(|key| *key == id) {
            self.queue.remove(pos);
        }
    }
}
