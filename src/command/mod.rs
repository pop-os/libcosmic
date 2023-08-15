// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Create asynchronous actions to be performed in the background.

#[cfg(feature = "wayland")]
use iced::window;
use iced::Command;
use iced_core::window::Mode;
#[cfg(feature = "wayland")]
use iced_runtime::command::platform_specific::wayland::window::Action as WindowAction;
#[cfg(feature = "wayland")]
use iced_runtime::command::platform_specific::wayland::Action as WaylandAction;
#[cfg(feature = "wayland")]
use iced_runtime::command::platform_specific::Action as PlatformAction;
use iced_runtime::command::Action;
#[cfg(not(feature = "wayland"))]
use iced_runtime::window::Action as WindowAction;
use std::future::Future;

/// Yields a command which contains a batch of commands.
pub fn batch<M>(commands: impl IntoIterator<Item = Command<M>>) -> Command<M> {
    Command::batch(commands)
}

/// Yields a command which will run the future on thet runtime executor.
pub fn future<M: Send + 'static>(future: impl Future<Output = M> + Send + 'static) -> Command<M> {
    Command::single(Action::Future(Box::pin(future)))
}

/// Yields a command which will return a message.
pub fn message<M: Send + 'static>(message: M) -> Command<M> {
    future(async move { message })
}

/// Initiates a window drag.
#[cfg(feature = "wayland")]
pub fn drag<M>() -> Command<M> {
    iced_sctk::commands::window::start_drag_window(window::Id(0))
}

/// Initiates a window drag.
#[cfg(not(feature = "wayland"))]
pub fn drag<M>() -> Command<M> {
    iced::Command::none()
}

/// Fullscreens the window.
#[cfg(feature = "wayland")]
pub fn fullscreen<M>() -> Command<M> {
    iced_sctk::commands::window::set_mode_window(window::Id(0), Mode::Fullscreen)
}

/// Fullscreens the window.
#[cfg(not(feature = "wayland"))]
pub fn fullscreen<M>() -> Command<M> {
    iced::Command::single(Action::Window(WindowAction::ChangeMode(Mode::Fullscreen)))
}

/// Minimizes the window.
#[cfg(feature = "wayland")]
pub fn minimize<M>() -> Command<M> {
    iced_sctk::commands::window::set_mode_window(window::Id(0), Mode::Hidden)
}

/// Minimizes the window.
#[cfg(not(feature = "wayland"))]
pub fn minimize<M>() -> Command<M> {
    iced::Command::single(Action::Window(WindowAction::ChangeMode(Mode::Hidden)))
}

/// Sets the title of a window.
#[cfg(feature = "wayland")]
pub fn set_title<M>(title: String) -> Command<M> {
    window_action(WindowAction::Title {
        id: window::Id(0),
        title,
    })
}

/// Sets the title of a window.
#[cfg(not(feature = "wayland"))]
#[allow(unused_variables, clippy::needless_pass_by_value)]
pub fn set_title<M>(title: String) -> Command<M> {
    Command::none()
}

/// Sets the window mode to windowed.
#[cfg(feature = "wayland")]
pub fn set_windowed<M>() -> Command<M> {
    iced_sctk::commands::window::set_mode_window(window::Id(0), Mode::Windowed)
}

/// Sets the window mode to windowed.
#[cfg(not(feature = "wayland"))]
pub fn set_windowed<M>() -> Command<M> {
    iced::Command::single(Action::Window(WindowAction::ChangeMode(Mode::Windowed)))
}

/// Toggles the windows' maximization state.
#[cfg(feature = "wayland")]
pub fn toggle_fullscreen<M>() -> Command<M> {
    window_action(WindowAction::ToggleFullscreen { id: window::Id(0) })
}

/// Toggles the windows' maximization state.
#[cfg(not(feature = "wayland"))]
pub fn toggle_fullscreen<M>() -> Command<M> {
    iced::Command::single(Action::Window(WindowAction::ToggleMaximize))
}

/// Creates a command to apply an action to a window.
#[cfg(feature = "wayland")]
pub fn window_action<M>(action: WindowAction<M>) -> Command<M> {
    Command::single(Action::PlatformSpecific(PlatformAction::Wayland(
        WaylandAction::Window(action),
    )))
}
