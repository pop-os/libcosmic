// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::app;

pub const fn app<M>(message: M) -> Action<M> {
    Action::App(message)
}

pub const fn cosmic<M>(message: app::Action) -> Action<M> {
    Action::Cosmic(message)
}

pub const fn none<M>() -> Action<M> {
    Action::None
}

#[derive(Clone, Debug)]
#[must_use]
pub enum Action<M> {
    /// Messages from the application, for the application.
    App(M),
    /// Internal messages to be handled by libcosmic.
    Cosmic(app::Action),
    #[cfg(feature = "single-instance")]
    /// Dbus activation messages
    DbusActivation(app::DbusActivationMessage),
    /// Do nothing
    None,
}

impl<M> From<M> for Action<M> {
    fn from(value: M) -> Self {
        Self::App(value)
    }
}
