// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#![allow(clippy::module_name_repetitions)]
#![cfg_attr(target_os = "redox", feature(lazy_cell))]

/// Recommended default imports.
pub mod prelude {
    #[cfg(feature = "winit")]
    pub use crate::ApplicationExt;
    pub use crate::ext::*;
    pub use crate::{Also, Apply, Element, Renderer, Task, Theme};
}

pub use apply::{Also, Apply};

/// Actions are managed internally by the cosmic runtime.
pub mod action;
pub use action::Action;

#[cfg(feature = "winit")]
pub mod app;
#[cfg(feature = "winit")]
#[doc(inline)]
pub use app::{Application, ApplicationExt};

#[cfg(feature = "applet")]
pub mod applet;

pub mod command;

/// State which is managed by the cosmic runtime.
pub mod core;
#[doc(inline)]
pub use core::Core;

pub mod config;

#[doc(inline)]
pub use cosmic_config;

#[doc(inline)]
pub use cosmic_theme;

#[cfg(feature = "single-instance")]
pub mod dbus_activation;
#[cfg(feature = "single-instance")]
pub use dbus_activation::DbusActivation;

#[cfg(feature = "desktop")]
pub mod desktop;

#[cfg(any(feature = "xdg-portal", feature = "rfd"))]
pub mod dialog;

pub mod executor;
#[cfg(feature = "tokio")]
pub use executor::single::Executor as SingleThreadExecutor;

mod ext;

pub mod font;

#[doc(inline)]
pub use iced;

#[doc(inline)]
pub use iced_core;

#[doc(inline)]
pub use iced_futures;

#[doc(inline)]
pub use iced_renderer;

#[doc(inline)]
pub use iced_runtime;

#[doc(inline)]
pub use iced_widget;

#[doc(inline)]
#[cfg(feature = "winit")]
pub use iced_winit;

#[doc(inline)]
#[cfg(feature = "wgpu")]
pub use iced_wgpu;

pub mod icon_theme;
pub mod keyboard_nav;

mod localize;

#[cfg(all(target_env = "gnu", not(target_os = "windows")))]
pub(crate) mod malloc;

#[cfg(all(feature = "process", not(windows)))]
pub mod process;

#[cfg(feature = "wayland")]
pub use cctk;

pub mod surface;

pub use iced::Task;
pub mod task;

pub mod theme;

#[doc(inline)]
pub use theme::{Theme, style};

pub mod widget;
type Plain = iced_core::text::paragraph::Plain<<Renderer as iced_core::text::Renderer>::Paragraph>;
type Paragraph = <Renderer as iced_core::text::Renderer>::Paragraph;
pub type Renderer = iced::Renderer;
pub type Element<'a, Message> = iced::Element<'a, Message, crate::Theme, Renderer>;
