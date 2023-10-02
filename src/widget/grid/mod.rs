// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod layout;
pub mod widget;

pub use widget::{Grid, Item};

/// Responsively generates rows and columns of widgets based on its dimmensions.
pub const fn grid<'a, Message>() -> Grid<'a, Message> {
    Grid::new()
}
