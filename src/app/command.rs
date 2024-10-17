// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use iced::window;

/// Asynchronous actions for COSMIC applications.
use super::Message;

/// Commands for COSMIC applications.
pub type Task<M> = iced::Task<Message<M>>;

/// Creates a command which yields a [`crate::app::Message`].
pub fn message<M: Send + 'static>(message: Message<M>) -> Task<M> {
    crate::command::message(message)
}

/// Convenience methods for building message-based commands.
pub mod message {
    /// Creates a command which yields an application message.
    pub fn app<M: Send + 'static>(message: M) -> crate::app::Task<M> {
        super::message(super::Message::App(message))
    }

    /// Creates a command which yields a cosmic message.
    pub fn cosmic<M: Send + 'static>(message: crate::app::cosmic::Message) -> crate::app::Task<M> {
        super::message(super::Message::Cosmic(message))
    }
}

impl crate::app::Core {
    pub fn drag<M: Send + 'static>(&self, id: Option<window::Id>) -> iced::Task<Message<M>> {
        let Some(id) = id.or(self.main_window.get().cloned()) else {
            return iced::Task::none();
        };
        crate::command::drag(id).map(Message::Cosmic)
    }

    pub fn maximize<M: Send + 'static>(
        &self,
        id: Option<window::Id>,
        maximized: bool,
    ) -> iced::Task<Message<M>> {
        let Some(id) = id.or(self.main_window.get().cloned()) else {
            return iced::Task::none();
        };
        crate::command::maximize(id, maximized).map(Message::Cosmic)
    }

    pub fn minimize<M: Send + 'static>(&self, id: Option<window::Id>) -> iced::Task<Message<M>> {
        let Some(id) = id.or(self.main_window.get().cloned()) else {
            return iced::Task::none();
        };
        crate::command::minimize(id).map(Message::Cosmic)
    }

    pub fn set_scaling_factor<M: Send + 'static>(&self, factor: f32) -> iced::Task<Message<M>> {
        message::cosmic(super::cosmic::Message::ScaleFactor(factor))
    }

    pub fn set_title<M: Send + 'static>(
        &self,
        id: Option<window::Id>,
        title: String,
    ) -> iced::Task<Message<M>> {
        let Some(id) = id.or(self.main_window.get().cloned()) else {
            return iced::Task::none();
        };
        crate::command::set_title(id, title).map(Message::Cosmic)
    }

    pub fn set_windowed<M: Send + 'static>(
        &self,
        id: Option<window::Id>,
    ) -> iced::Task<Message<M>> {
        let Some(id) = id.or(self.main_window.get().cloned()) else {
            return iced::Task::none();
        };
        crate::command::set_windowed(id).map(Message::Cosmic)
    }

    pub fn toggle_maximize<M: Send + 'static>(
        &self,
        id: Option<window::Id>,
    ) -> iced::Task<Message<M>> {
        let Some(id) = id.or(self.main_window.get().cloned()) else {
            return iced::Task::none();
        };
        crate::command::toggle_maximize(id).map(Message::Cosmic)
    }
}

pub fn set_theme<M: Send + 'static>(theme: crate::Theme) -> iced::Task<Message<M>> {
    message::cosmic(super::cosmic::Message::AppThemeChange(theme))
}
