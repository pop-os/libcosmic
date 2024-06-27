// Copyright 2024 wiiznokes
// SPDX-License-Identifier: MIT

use std::time::Duration;

use crate::widget::container;
use crate::widget::Column;
use iced::{Command, Length};
use iced_core::Element;
use widget::Toaster;

use crate::ext::CollectionWidget;

use super::{button, row, text};

mod widget;

pub fn toaster<'a, Message>(
    toasts: &'a Toasts<Message>,
    content: impl Into<Element<'a, Message, crate::Theme, iced::Renderer>>,
) -> Element<'a, Message, crate::Theme, iced::Renderer>
where
    Message: Clone + 'a,
{
    let make_toast = |toast: &'a Toast<Message>| {
        let row = row().push(text(&toast.message)).push_maybe(
            toast
                .action
                .as_ref()
                .map(|t| button(text(t.0.clone())).on_press(t.1.clone())),
        );

        container(row).into()
    };

    let col = Column::with_children(toasts.toasts.iter().map(make_toast));

    Toaster::new(col.into(), content.into(), toasts.toasts.is_empty()).into()
}

pub enum ToastDuration {
    Short,
    Medium,
    Long,
}

impl ToastDuration {
    fn duration(&self) -> Duration {
        match self {
            ToastDuration::Short => Duration::from_millis(200),
            ToastDuration::Medium => Duration::from_millis(1000),
            ToastDuration::Long => Duration::from_millis(5000),
        }
    }
}

pub struct Toast<Message> {
    message: String,
    /// (Description, message)
    action: Option<(String, Message)>,

    duration: ToastDuration,

    id: u32,
}

pub struct Toasts<Message> {
    id_count: u32,
    toasts: Vec<Toast<Message>>,
}

pub struct ToastMessage(u32);

impl<Message> Toasts<Message> {
    pub fn push(&mut self, mut toast: Toast<Message>) -> Command<ToastMessage> {
        toast.id = self.id_count;
        self.id_count += 1;

        let message = ToastMessage(toast.id);
        let duration = toast.duration.duration();

        self.toasts.push(toast);

        Command::perform(
            async move {
                tokio::time::sleep(duration).await;
            },
            |_| message,
        )
    }

    pub fn handle_message(&mut self, message: ToastMessage) {
        self.toasts
            .iter()
            .position(|e| e.id == message.0)
            .map(|index| self.toasts.remove(index));
    }
}
