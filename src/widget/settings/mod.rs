// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

pub mod item;
pub mod section;

pub use self::item::{flex_item, flex_item_row, item, item_row};
pub use self::section::{section, view_section, Section};

use crate::widget::{column, Column};
use crate::{theme, Element};

/// A column with a predefined style for creating a settings panel
#[must_use]
pub fn view_column<Message: 'static>(children: Vec<Element<Message>>) -> Column<Message> {
    let spacing = theme::THEME.lock().unwrap().cosmic().spacing;
    column::with_children(children)
        .spacing(spacing.space_m)
        .padding([0, spacing.space_m])
}
