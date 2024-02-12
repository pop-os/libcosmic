// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Create asynchronous actions to be performed in the background.

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
use std::future::Future;

/// Yields a command which contains a batch of commands.
pub fn batch<M>(commands: impl IntoIterator<Item = Command<M>>) -> Command<M> {
    Command::batch(commands)
}

/// Yields a command which will run the future on the runtime executor.
pub fn future<M>(future: impl Future<Output = M> + Send + 'static) -> Command<M> {
    Command::single(Action::Future(Box::pin(future)))
}

/// Yields a command which will return a message.
pub fn message<M: Send + 'static>(message: M) -> Command<M> {
    future(async move { message })
}

/// Initiates a window drag.
#[cfg(feature = "wayland")]
pub fn drag<M>(id: Option<window::Id>) -> Command<M> {
    iced_sctk::commands::window::start_drag_window(id.unwrap_or(window::Id::MAIN))
}

/// Initiates a window drag.
#[cfg(not(feature = "wayland"))]
pub fn drag<M>(id: Option<window::Id>) -> Command<M> {
    iced_runtime::window::drag(id.unwrap_or(window::Id::MAIN))
}

/// Maximizes the window.
#[cfg(feature = "wayland")]
pub fn maximize<M>(id: Option<window::Id>, maximized: bool) -> Command<M> {
    iced_sctk::commands::window::maximize(id.unwrap_or(window::Id::MAIN), maximized)
}

/// Maximizes the window.
#[cfg(not(feature = "wayland"))]
pub fn maximize<M>(id: Option<window::Id>, maximized: bool) -> Command<M> {
    iced_runtime::window::maximize(id.unwrap_or(window::Id::MAIN), maximized)
}

/// Minimizes the window.
#[cfg(feature = "wayland")]
pub fn minimize<M>(id: Option<window::Id>) -> Command<M> {
    iced_sctk::commands::window::set_mode_window(id.unwrap_or(window::Id::MAIN), Mode::Hidden)
}

/// Minimizes the window.
#[cfg(not(feature = "wayland"))]
pub fn minimize<M>(id: Option<window::Id>) -> Command<M> {
    iced_runtime::window::minimize(id.unwrap_or(window::Id::MAIN), true)
}

/// Sets the title of a window.
#[cfg(feature = "wayland")]
pub fn set_title<M>(id: Option<window::Id>, title: String) -> Command<M> {
    window_action(WindowAction::Title {
        id: id.unwrap_or(window::Id::MAIN),
        title,
    })
}

/// Sets the title of a window.
#[cfg(not(feature = "wayland"))]
#[allow(unused_variables, clippy::needless_pass_by_value)]
pub fn set_title<M>(id: Option<window::Id>, title: String) -> Command<M> {
    Command::none()
}

/// Sets the window mode to windowed.
#[cfg(feature = "wayland")]
pub fn set_windowed<M>(id: Option<window::Id>) -> Command<M> {
    iced_sctk::commands::window::set_mode_window(id.unwrap_or(window::Id::MAIN), Mode::Windowed)
}

/// Sets the window mode to windowed.
#[cfg(not(feature = "wayland"))]
pub fn set_windowed<M>(id: Option<window::Id>) -> Command<M> {
    iced_runtime::window::change_mode(id.unwrap_or(window::Id::MAIN), Mode::Windowed)
}

/// Toggles the windows' maximize state.
#[cfg(feature = "wayland")]
pub fn toggle_maximize<M>(id: Option<window::Id>) -> Command<M> {
    iced_sctk::commands::window::toggle_maximize(id.unwrap_or(window::Id::MAIN))
}

/// Toggles the windows' maximize state.
#[cfg(not(feature = "wayland"))]
pub fn toggle_maximize<M>(id: Option<window::Id>) -> Command<M> {
    iced_runtime::window::toggle_maximize(id.unwrap_or(window::Id::MAIN))
}

/// Creates a command to apply an action to a window.
#[cfg(feature = "wayland")]
pub fn window_action<M>(action: WindowAction<M>) -> Command<M> {
    Command::single(Action::PlatformSpecific(PlatformAction::Wayland(
        WaylandAction::Window(action),
    )))
}
