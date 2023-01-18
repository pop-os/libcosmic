// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#![allow(clippy::module_name_repetitions)]

pub use iced;
pub use iced_lazy;
pub use iced_native;
#[cfg(feature = "wayland")]
pub use iced_sctk;
pub use iced_style;
#[cfg(feature = "winit")]
pub use iced_winit;

#[cfg(feature = "applet")]
pub mod applet;
pub mod font;
pub mod keyboard_nav;
pub mod theme;
pub mod widget;

#[cfg(feature = "tokio")]
mod single_thread_executor;
#[cfg(feature = "tokio")]
pub use single_thread_executor::SingleThreadExecutor;

pub mod settings;
pub use settings::settings;

mod ext;
pub use ext::ElementExt;

pub use theme::Theme;
pub type Renderer = iced::Renderer<Theme>;
pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
