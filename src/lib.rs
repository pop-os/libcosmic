// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

#![allow(clippy::module_name_repetitions)]
#![cfg_attr(target_os = "redox", feature(lazy_cell))]
#![cfg_attr(docsrs, feature(doc_cfg))]

//! # The COSMIC Toolkit
//!
//! Quickly code your project with modular elements created with customization in mind.
//! Panels, applets, theming, tiling, launcher, app library, keyboard shortcuts and
//! dynamic or pinned workspaces are all made flexible to bend to your users needs.
//!
//! [COSMIC](https://system76.com/cosmic) empowers frictionless development by being
//! approachable, easy to maintain, and modular. Use the same language and toolkit to
//! build apps and applets, with helpful templates provided. Even shell components and
//! the compositor use the same toolkit. Learn once and use your knowledge anywhere in
//! the desktop.
//!
//! ## Architecture
//!
//! Based on [iced](https://iced.rs/), COSMIC apps and applets are modeled with the
//! MVU (Model-View-Update) pattern from [The Elm Architecture](https://guide.elm-lang.org/architecture/).
//! For more details, see the [architecture page of the iced book](https://book.iced.rs/architecture.html).
//!
//! An application will consist of:
//!
//! * An application **model** for holding persistent application state which implements
//!   the [Application] trait.
//! * A [view](Application::view) function which borrows data from the model to construct
//!   a view with stateless widgets
//! * An [update](Application::update) function which receives application messages from
//!   widgets, tasks, and subscriptions.
//!
//! Messages handled by the update function are used to update the
//! application model and spawn background tasks that can emit messages back to the
//! app's update function.
//!
//! ### Tasks
//!
//! Tasks returned by the update function are scheduled for concurrent execution on a
//! background thread managed by the application's async executor, which is tokio by default.
//! They can be constructed from futures and may also stream events back to the application
//! asynchronously.
//!
//! ### Subscriptions
//!
//! Applications may also use a [subscription](Application::subscription) function to subscribe to external
//! asynchronous event streams. These can run perpetually from application start; or
//! optionally started, stopped, and restarted based on changes to the application model
//! between updates. Such as:
//!
//! * Conditionally starting and stopping a subscription based on the state of a boolean
//!   value or enum
//! * Restarting a subscription when the hash of its borrowed data changes
//! * Dynamically spawning a subscription for each item in a list.
//!
//! ## Templates
//!
//! Get started using [cargo-generate](https://github.com/cargo-generate/cargo-generate)
//! with one of the following templates. The app template is for developing desktop
//! applications and the applet template is for developing COSMIC applets.
//!
//! - [App Template](https://github.com/pop-os/cosmic-app-template/)
//! - [Applet Template](https://github.com/pop-os/cosmic-applet-template/)
//!
//! ## Widgets
//!
//! Reference the [`widget`] module for available widgets for use in the view function.
//! Widgets are composable and can be configured through chainable builder methods.
//! Compose widgets together to create complex interfaces and higher level widgets.
//!
//! Composed widgets may be managed by their own custom type with its own view and
//! update functions. It is a common pattern to use these functions within the
//! application's own view and update functions. If using a custom type, implement
//! [From] for the [Element] type to have API treat a composed widget the same as a
//! native custom widget.
//!
//! If a widget does not exist for a specific use case, use the [Widget](iced::advanced::Widget)
//! trait to create an advanced custom widget. This can then be used to composed higher
//! level widgets by chaining composable widgets together.
//!
//! ## Core
//!
//! Every application model requires a [cosmic::Core](app::Core). This contains
//! application state which is managed by libcosmic's runtime for its generated
//! interfaces. Such as the context drawner, nav bar, and the headerbar. This can be
//! used by the app to subscribe to configuration changes and to emit events to the
//! libcosmic-managed portion of the application's state and view.

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

pub mod anim;

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

pub mod icon_theme;
pub mod keyboard_nav;

mod localize;

#[cfg(all(target_env = "gnu", not(target_os = "windows")))]
pub(crate) mod malloc;

#[cfg(all(feature = "process", not(windows)))]
pub mod process;

#[doc(inline)]
#[cfg(wayland_platform)]
pub use cctk;

pub mod surface;

pub use iced::Task;
pub mod task;

pub mod theme;

pub mod scroll;

#[doc(inline)]
pub use theme::{Theme, style};

pub mod widget;
type Plain = iced_core::text::paragraph::Plain<<Renderer as iced_core::text::Renderer>::Paragraph>;
type Paragraph = <Renderer as iced_core::text::Renderer>::Paragraph;
pub type Renderer = iced::Renderer;
pub type Element<'a, Message> = iced::Element<'a, Message, crate::Theme, Renderer>;
