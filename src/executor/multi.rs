// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! An async executor that schedules tasks across a pol ofbackground thread.

use std::future::Future;

#[cfg(feature = "tokio")]
pub struct Executor(tokio::runtime::Runtime);

#[cfg(feature = "tokio")]
impl iced::Executor for Executor {
    fn new() -> Result<Self, iced::futures::io::Error> {
        Ok(Self(
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?,
        ))
    }

    fn spawn(&self, future: impl Future<Output = ()> + Send + 'static) {
        let _res = self.0.spawn(future);
    }

    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        let _guard = self.0.enter();
        f()
    }
}
