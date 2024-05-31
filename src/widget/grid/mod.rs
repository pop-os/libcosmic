// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Arrange widgets with a grid layout.

pub mod layout;
pub mod widget;

pub use widget::Grid;

/// Arrange widgets with a grid layout.
pub const fn grid<'a, Message>() -> Grid<'a, Message> {
    Grid::new()
}
