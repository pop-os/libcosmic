// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! An async executor that schedules tasks on the same background thread.

use std::future::Future;

#[cfg(feature = "tokio")]
pub struct Executor(tokio::runtime::Runtime);

#[cfg(feature = "tokio")]
impl iced::Executor for Executor {
    fn new() -> Result<Self, iced::futures::io::Error> {
        // Current thread executor requires calling `block_on` to actually run
        // futures. Main thread is busy with things other than running futures,
        // so spawn a single worker thread.
        Ok(Self(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
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
