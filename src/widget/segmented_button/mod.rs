// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A widget providing a conjoined set of linear items that function in conjunction as a single button.
//!
//! ## Example
//!
//! Add the model and a message variant in your application for handling selections.
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
//!     model: segmented_button::SingleSelectModel,
//! }
//! ```
//!
//! Then add choices to the model, while activating the first.
//!
//! ```ignore
//! application.model = segmented_button::Model::builder()
//!     .insert(|b| b.text("Choice A").data(0u16))
//!     .insert(|b| b.text("Choice B").data(1u16))
//!     .insert(|b| b.text("Choice C").data(2u16))
//!     .build();
//! ```
//!
//! Or incrementally insert items with
//!
//! ```ignore
//! let id = application.model.insert()
//!     .text("Choice C")
//!     .icon("custom-icon")
//!     .data(3u16)
//!     .data("custom-meta")
//!     .id();
//! ```
//!
//! Then use it in the view method to create segmented button widgets.
//!
//! ```ignore
//! let widget = segmented_button::horizontal(&application.model)
//!     .style(theme::SegmentedButton::ViewSeitcher)
//!     .button_height(32)
//!     .button_padding([16, 10, 16, 10])
//!     .button_spacing(8)
//!     .icon_size(16)
//!     .spacing(8)
//!     .on_activate(AppMessage::Selected);
//! ```
//!
//! And respond to events like so:
//!
//! ```ignore
//! match message {
//!     AppMessage::Selected(id) => {
//!         application.model.activate(id);
//!
//!         if let Some(number) = application.model.data::<u16>(id) {
//!             println!("activated item with number {number}");
//!         }
//!
//!         if let Some(text) = application.text(id) {
//!             println!("activated button with text {text}");
//!         }
//!     }
//! }
//! ```

mod horizontal;
mod model;
mod style;
mod vertical;
mod widget;

pub use self::horizontal::{HorizontalSegmentedButton, horizontal};
pub use self::model::{
    BuilderEntity, Entity, EntityMut, Model, ModelBuilder, MultiSelect, MultiSelectEntityMut,
    MultiSelectModel, Selectable, SingleSelect, SingleSelectEntityMut, SingleSelectModel,
};
pub use self::style::{Appearance, ItemAppearance, ItemStatusAppearance, StyleSheet};
pub use self::vertical::{VerticalSegmentedButton, vertical};
pub use self::widget::{Id, SegmentedButton, SegmentedVariant, focus};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsertPosition {
    Before,
    After,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReorderEvent {
    pub dragged: Entity,
    pub target: Entity,
    pub position: InsertPosition,
}

/// Associates extra data with an external secondary map.
///
/// The secondary map internally uses a `Vec`, so should only be used for data that
pub type SecondaryMap<T> = slotmap::SecondaryMap<Entity, T>;

/// Associates extra data with an external sparse secondary map.
///
/// Sparse maps internally use a `HashMap`, for data that is sparsely associated.
pub type SparseSecondaryMap<T> = slotmap::SparseSecondaryMap<Entity, T>;
