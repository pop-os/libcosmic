// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Create asynchronous actions to be performed in the background.

use futures::stream::{Stream, StreamExt};
use iced_core::window;
use iced_runtime::task;
use std::future::Future;

use crate::direction::Direction;

/// Yields a task which contains a batch of tasks.
pub fn batch<X: Send + 'static + Into<Y>, Y: Send + 'static>(
    tasks: impl IntoIterator<Item = iced::Task<X>>,
) -> iced::Task<Y> {
    iced::Task::batch(tasks).map(Into::into)
}

/// Yields a task which will run the future on the runtime executor.
pub fn future<X: Into<Y>, Y: 'static>(
    future: impl Future<Output = X> + Send + 'static,
) -> iced::Task<Y> {
    iced::Task::future(async move { future.await.into() })
}

/// Yields a task which will return a message.
pub fn message<X: Send + 'static + Into<Y>, Y: 'static>(message: X) -> iced::Task<Y> {
    future(async move { message.into() })
}

/// Yields a task which will run a stream on the runtime executor.
pub fn stream<X: Into<Y> + 'static, Y: 'static>(
    stream: impl Stream<Item = X> + Send + 'static,
) -> iced::Task<Y> {
    iced::Task::stream(stream.map(Into::into))
}

pub fn none<Y: 'static>() -> iced::Task<Y> {
    iced::Task::none()
}

/// Focuses the next focusable widget.
pub fn focus_next<T>(window: Vec<window::Id>) -> iced::Task<T> {
    task::effect(iced_runtime::Action::widget(crate::focus::focus_next(
        window,
    )))
}

/// Focuses the previous focusable widget.
pub fn focus_previous<T>(window: Vec<window::Id>) -> iced::Task<T> {
    task::effect(iced_runtime::Action::widget(crate::focus::focus_previous(
        window,
    )))
}

#[cfg(feature = "winit")]
pub fn dir_focus<T: Send + 'static>(w_id: window::Id, d: Direction) -> iced::Task<T> {
    crate::app::cosmic::dir_focus_task(w_id, d, false)
}
