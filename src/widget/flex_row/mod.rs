// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod layout;
pub mod widget;

pub use widget::FlexRow;

use crate::Element;

/// Responsively generates rows and columns of widgets based on its dimmensions.
pub const fn flex_row<Message>(children: Vec<Element<Message>>) -> FlexRow<Message> {
    FlexRow::new(children)
}
