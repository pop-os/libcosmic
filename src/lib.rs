// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#![allow(clippy::module_name_repetitions)]

/// Recommended default imports.
pub mod prelude {
    pub use crate::ext::*;
    pub use crate::{Element, Renderer, Theme};
}

pub use apply::{Also, Apply};

pub mod app;
pub use app::{Application, ApplicationExt};

pub use iced::Command;
pub mod command;
pub use cosmic_config;
pub use cosmic_theme;

#[cfg(feature = "xdg-portal")]
pub mod dialog;

pub mod executor;
#[cfg(feature = "tokio")]
pub use executor::single::Executor as SingleThreadExecutor;

mod ext;

pub mod font;

pub use iced;
pub use iced_core;
pub use iced_futures;
pub use iced_renderer;
pub use iced_runtime;
#[cfg(feature = "wayland")]
pub use iced_sctk;
pub use iced_style;
pub use iced_widget;
#[cfg(feature = "winit")]
pub use iced_winit;

pub mod icon_theme;
pub mod keyboard_nav;

#[cfg(feature = "wayland")]
pub use sctk;

pub mod theme;
pub use theme::Theme;

pub mod widget;

pub type Renderer = iced::Renderer<Theme>;
pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
