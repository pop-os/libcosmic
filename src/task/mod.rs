// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Create asynchronous actions to be performed in the background.

use iced::window;
use iced::Task;
use iced_core::window::Mode;
use std::future::Future;

/// Yields a task which contains a batch of tasks.
pub fn batch<X: Send + 'static + Into<Y>, Y: Send + 'static>(
    tasks: impl IntoIterator<Item = Task<X>>,
) -> Task<Y> {
    Task::batch(tasks).map(Into::into)
}

/// Yields a task which will run the future on the runtime executor.
pub fn future<X: Into<Y>, Y: 'static>(future: impl Future<Output = X> + Send + 'static) -> Task<Y> {
    Task::future(async move { future.await.into() })
}

/// Yields a task which will return a message.
pub fn message<X: Send + 'static + Into<Y>, Y: 'static>(message: X) -> Task<Y> {
    future(async move { message.into() })
}

/// Initiates a window drag.
pub fn drag<M>(id: window::Id) -> Task<M> {
    iced_runtime::window::drag(id)
}

/// Maximizes the window.
pub fn maximize<M>(id: window::Id, maximized: bool) -> Task<M> {
    iced_runtime::window::maximize(id, maximized)
}

/// Minimizes the window.
pub fn minimize<M>(id: window::Id) -> Task<M> {
    iced_runtime::window::minimize(id, true)
}

/// Sets the title of a window.
#[allow(unused_variables, clippy::needless_pass_by_value)]
pub fn set_title<M>(id: window::Id, title: String) -> Task<M> {
    Task::none()
}

/// Sets the window mode to windowed.
pub fn set_windowed<M>(id: window::Id) -> Task<M> {
    iced_runtime::window::change_mode(id, Mode::Windowed)
}

/// Toggles the windows' maximize state.
pub fn toggle_maximize<M>(id: window::Id) -> Task<M> {
    iced_runtime::window::toggle_maximize(id)
}
