// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced::window;

/// Asynchronous actions for COSMIC applications.
use super::Message;

/// Commands for COSMIC applications.
pub type Command<M> = iced::Command<Message<M>>;

/// Creates a command which yields a [`crate::app::Message`].
pub fn message<M: Send + 'static>(message: Message<M>) -> Command<M> {
    crate::command::message(message)
}

/// Convenience methods for building message-based commands.
pub mod message {
    /// Creates a command which yields an application message.
    pub fn app<M: Send + 'static>(message: M) -> crate::app::Command<M> {
        super::message(super::Message::App(message))
    }

    /// Creates a command which yields a cosmic message.
    pub fn cosmic<M: Send + 'static>(
        message: crate::app::cosmic::Message,
    ) -> crate::app::Command<M> {
        super::message(super::Message::Cosmic(message))
    }
}

pub fn drag<M: Send + 'static>(id: Option<window::Id>) -> iced::Command<Message<M>> {
    crate::command::drag(id).map(Message::Cosmic)
}

pub fn maximize<M: Send + 'static>(
    id: Option<window::Id>,
    maximized: bool,
) -> iced::Command<Message<M>> {
    crate::command::maximize(id, maximized).map(Message::Cosmic)
}

pub fn minimize<M: Send + 'static>(id: Option<window::Id>) -> iced::Command<Message<M>> {
    crate::command::minimize(id).map(Message::Cosmic)
}

pub fn set_scaling_factor<M: Send + 'static>(factor: f32) -> iced::Command<Message<M>> {
    message::cosmic(super::cosmic::Message::ScaleFactor(factor))
}

pub fn set_theme<M: Send + 'static>(theme: crate::Theme) -> iced::Command<Message<M>> {
    message::cosmic(super::cosmic::Message::AppThemeChange(theme))
}

pub fn set_title<M: Send + 'static>(
    id: Option<window::Id>,
    title: String,
) -> iced::Command<Message<M>> {
    crate::command::set_title(id, title).map(Message::Cosmic)
}

pub fn set_windowed<M: Send + 'static>(id: Option<window::Id>) -> iced::Command<Message<M>> {
    crate::command::set_windowed(id).map(Message::Cosmic)
}

pub fn toggle_maximize<M: Send + 'static>(id: Option<window::Id>) -> iced::Command<Message<M>> {
    crate::command::toggle_maximize(id).map(Message::Cosmic)
}
