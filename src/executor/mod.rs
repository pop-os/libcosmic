// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Select the preferred async executor for an application.

#[cfg(feature = "tokio")]
pub mod multi;

#[cfg(feature = "tokio")]
pub mod single;

/// Uses the single thread executor by default.
#[cfg(not(feature = "tokio"))]
pub type Default = iced::executor::Default;

/// Uses the single thread executor by default.
#[cfg(feature = "tokio")]
pub type Default = single::Executor;
