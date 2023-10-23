// Copyright 2023 System76 <info@system76.com>
// Copyright 2019 Héctor Ramón, Iced contributors
// SPDX-License-Identifier: MPL-2.0 AND MIT

pub mod menu;
pub use menu::Menu;

mod widget;
pub use widget::*;

pub fn dropdown<'a, S: AsRef<str>, Message: 'a>(
    selections: &'a [S],
    selected: Option<usize>,
    on_selected: impl Fn(usize) -> Message + 'a,
) -> Dropdown<'a, S, Message> {
    Dropdown::new(selections, selected, on_selected)
}
