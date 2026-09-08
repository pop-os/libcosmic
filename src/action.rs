// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#[cfg(feature = "winit")]
use crate::app;
#[cfg(feature = "single-instance")]
use crate::dbus_activation;

pub const fn app<M>(message: M) -> Action<M> {
    Action::App(message)
}
#[cfg(feature = "winit")]
pub const fn cosmic<M>(message: app::Action) -> Action<M> {
    Action::Cosmic(message)
}

pub const fn none<M>() -> Action<M> {
    Action::None
}

/// Wrap a surface action, typically produced by a widget, to be handled by libcosmic.
pub const fn surface<M>(action: crate::surface::Action<M>) -> Action<M> {
    Action::Surface(action)
}

#[derive(Clone, Debug)]
#[must_use]
pub enum Action<M> {
    /// Messages from the application, for the application.
    App(M),
    #[cfg(feature = "winit")]
    /// Internal messages to be handled by libcosmic.
    Cosmic(app::Action),
    #[cfg(feature = "single-instance")]
    /// Dbus activation messages
    DbusActivation(dbus_activation::Message),
    /// Surface (popup, subsurface, window, layer shell) requests, handled by libcosmic.
    Surface(crate::surface::Action<M>),
    /// Do nothing
    None,
}

impl<M: 'static> Action<M> {
    /// Map the application message inside, leaving libcosmic's own variants untouched.
    #[must_use]
    pub fn map<N: 'static>(self, f: impl Fn(M) -> N + Clone + Send + Sync + 'static) -> Action<N> {
        match self {
            Action::App(message) => Action::App(f(message)),
            #[cfg(feature = "winit")]
            Action::Cosmic(action) => Action::Cosmic(action),
            #[cfg(feature = "single-instance")]
            Action::DbusActivation(message) => Action::DbusActivation(message),
            Action::Surface(action) => Action::Surface(action.map(f)),
            Action::None => Action::None,
        }
    }
}

impl<M: 'static> Action<Action<M>> {
    /// Collapse a doubly wrapped action, as produced by widgets whose message type is already
    /// an [`Action`], into a single one.
    #[must_use]
    pub fn flatten(self) -> Action<M> {
        match self {
            Action::App(action) => action,
            #[cfg(feature = "winit")]
            Action::Cosmic(action) => Action::Cosmic(action),
            #[cfg(feature = "single-instance")]
            Action::DbusActivation(message) => Action::DbusActivation(message),
            Action::Surface(action) => Action::Surface(action.flatten()),
            Action::None => Action::None,
        }
    }
}

impl<M> From<M> for Action<M> {
    fn from(value: M) -> Self {
        Self::App(value)
    }
}
