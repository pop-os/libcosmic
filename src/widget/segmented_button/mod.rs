// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A widget providing a conjoined set of linear buttons for choosing between.
//!
//! ## Example
//!
//! Add the state and a message variant in your application for handling selections.
//!
//! ```ignore
//! use iced_core::Length;
//! use cosmic::theme;
//! use cosmic::widget::segmented_button;
//!
//! enum AppMessage {
//!     Selected(segmented_button::Key)
//! }
//!
//! struct App {
//!     ...
//!     state: segmented_button::State<u16>(),
//!     ...
//! }
//! ```
//!
//! Then add choices to the state, while activating the first.
//!
//! ```ignore
//! let first_key = application.state.insert("Choice A", 0);
//! application.state.insert("Choice B", 1);
//! application.state.insert("Choice C", 2);
//! application.state.activate(first_key);
//! ```
//!
//! Then use it in the view method to create segmented button widgets.
//!
//! ```ignore
//! let widget = segmentend_button(&application.state)
//!     .style(theme::SegmentedButton::Selection)
//!     .height(Length::Units(32))
//!     .on_activate(AppMessage::Selected);
//! ```

/// COSMIC configurations of [`SegmentedButton`].
pub mod cosmic;

mod horizontal;
mod state;
mod style;
mod vertical;
mod widget;

pub use self::horizontal::{horizontal_segmented_button, Horizontal, HorizontalSegmentedButton};
pub use self::state::{ButtonContent, Key, SecondaryState, SharedWidgetState, State};
pub use self::style::{Appearance, ButtonAppearance, ButtonStatusAppearance, StyleSheet};
pub use self::vertical::{vertical_segmented_button, Vertical, VerticalSegmentedButton};
pub use self::widget::{SegmentedButton, SegmentedVariant};
