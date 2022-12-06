// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod item;
mod section;

pub use self::item::{item, item_row};
pub use self::section::{Section, view_section};

use crate::{Element, Renderer};
use iced::widget::{Column, column};

/// A column with a predefined style for creating a settings panel
#[must_use]
pub fn view_column<Message: 'static>(children: Vec<Element<Message>>) -> Column<Message, Renderer> {
    column(children).spacing(24).padding(24).max_width(600)
}
