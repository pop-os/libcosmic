// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#![allow(clippy::module_name_repetitions)]

#[cfg(all(feature = "wayland", feature = "winit"))]
compile_error!("cannot use `wayland` feature with `winit`");

/// Recommended default imports.
pub mod prelude {
    pub use crate::ext::*;
    #[cfg(any(feature = "winit", feature = "wayland"))]
    pub use crate::ApplicationExt;
    pub use crate::{Also, Apply, Element, Renderer, Theme};
}

pub use apply::{Also, Apply};

#[cfg(any(feature = "winit", feature = "wayland"))]
pub mod app;
#[cfg(any(feature = "winit", feature = "wayland"))]
pub use app::{Application, ApplicationExt};

#[cfg(feature = "applet")]
pub mod applet;

pub use iced::Command;
pub mod command;

pub mod config;

#[doc(inline)]
pub use cosmic_config;

#[doc(inline)]
pub use cosmic_theme;

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

#[cfg(feature = "wayland")]
pub use iced_sctk;

#[doc(inline)]
pub use iced_style;

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

#[cfg(feature = "desktop")]
pub mod desktop;
#[cfg(feature = "process")]
pub mod process;

#[cfg(feature = "wayland")]
pub use cctk;

pub mod theme;

#[doc(inline)]
pub use theme::{style, Theme};

pub mod widget;

type Paragraph = <Renderer as iced_core::text::Renderer>::Paragraph;
pub type Renderer = iced::Renderer;
pub type Element<'a, Message> = iced::Element<'a, Message, crate::Theme, Renderer>;
