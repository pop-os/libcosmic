use std::{future::Future, thread};

#[cfg(feature = "tokio")]
pub struct SingleThreadExecutor(tokio::runtime::Runtime);

#[cfg(feature = "tokio")]
impl iced_native::Executor for SingleThreadExecutor {
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
        let _ = self.0.spawn(future);
    }

    fn enter<R>(&self, f: impl FnOnce() -> R) -> R {
        let _guard = self.0.enter();
        f()
    }
}
