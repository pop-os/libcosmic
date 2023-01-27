// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#[cfg(feature = "tokio")]
pub mod multi;

#[cfg(feature = "tokio")]
pub mod single;

#[cfg(not(feature = "tokio"))]
pub type Default = iced::executor::Default;

#[cfg(feature = "tokio")]
pub type Default = single::Executor;
