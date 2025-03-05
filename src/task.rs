// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Create asynchronous actions to be performed in the background.

use std::future::Future;

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
