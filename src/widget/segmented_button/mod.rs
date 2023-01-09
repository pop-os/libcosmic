// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A widget providing a conjoined set of linear buttons that function in conjunction with each other.
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
//!     state: segmented_button::SingleSelectModel<u16>(),
//! }
//! ```
//!
//! Then add choices to the state, while activating the first.
//!
//! ```ignore
//! application.model = SingleSelectModel::builder()
//!     .insert_activate("Choice A", 0)
//!     .insert("Choice B", 1)
//!     .insert("Choice C", 2)
//!     .build();
//! ```
//!
//! Then use it in the view method to create segmented button widgets.
//!
//! ```ignore
//! let widget = horizontal_segmented_button(&application.model)
//!     .style(theme::SegmentedButton::ViewSeitcher)
//!     .button_height(32)
//!     .button_padding([16, 10, 16, 10])
//!     .button_spacing(8)
//!     .icon_size(16)
//!     .spacing(8)
//!     .on_activate(AppMessage::Selected);
//! ```

/// COSMIC configurations of [`SegmentedButton`].
pub mod cosmic;

mod horizontal;
mod item;
mod model;
mod selection_modes;
mod style;
mod vertical;
mod widget;

pub use self::horizontal::{horizontal_segmented_button, HorizontalSegmentedButton};
pub use self::item::{item, SegmentedItem};
pub use self::model::{Batch, Key, Model, ModelBuilder, MultiSelectModel, SingleSelectModel};
pub use self::selection_modes::Selectable;
pub use self::style::{Appearance, ButtonAppearance, ButtonStatusAppearance, StyleSheet};
pub use self::vertical::{vertical_segmented_button, VerticalSegmentedButton};
pub use self::widget::{focus, Id, SegmentedButton, SegmentedVariant};
