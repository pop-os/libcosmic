pub mod single;

#[cfg(not(feature = "tokio"))]
pub type Default = iced::executor::Default;

#[cfg(feature = "tokio")]
pub type Default = single::Executor;
