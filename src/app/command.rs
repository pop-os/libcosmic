// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::future::Future;

use super::{Command, Message};
use iced_runtime::command::Action;

/// Yields a command which contains a batch of commands.
pub fn batch<M: Send + 'static>(commands: impl IntoIterator<Item = Command<M>>) -> Command<M> {
    Command::batch(commands)
}

/// Yields a command which will run the future on the runtime executor.
pub fn future<M: Send + 'static>(
    future: impl Future<Output = Message<M>> + Send + 'static,
) -> Command<M> {
    Command::single(Action::Future(Box::pin(future)))
}

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

pub fn drag<M: Send + 'static>() -> iced::Command<Message<M>> {
    crate::command::drag().map(Message::Cosmic)
}

pub fn fullscreen<M: Send + 'static>() -> iced::Command<Message<M>> {
    crate::command::fullscreen().map(Message::Cosmic)
}

pub fn minimize<M: Send + 'static>() -> iced::Command<Message<M>> {
    crate::command::minimize().map(Message::Cosmic)
}

pub fn set_title<M: Send + 'static>(title: String) -> iced::Command<Message<M>> {
    crate::command::set_title(title).map(Message::Cosmic)
}

pub fn set_windowed<M: Send + 'static>() -> iced::Command<Message<M>> {
    crate::command::set_windowed().map(Message::Cosmic)
}

pub fn toggle_fullscreen<M: Send + 'static>() -> iced::Command<Message<M>> {
    crate::command::toggle_fullscreen().map(Message::Cosmic)
}
