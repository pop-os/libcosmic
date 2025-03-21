// Copyright 2023 System76 <info@system76.com>
// Copyright 2019 Héctor Ramón, Iced contributors
// SPDX-License-Identifier: MPL-2.0 AND MIT

mod model;
pub use model::{List, Model, list, model};

pub mod menu;
pub use menu::Menu;

mod widget;
pub use widget::{Catalog, Dropdown, Style};

pub fn dropdown<'a, S: AsRef<str>, Message: 'a, Item: Clone + PartialEq + 'static>(
    model: &'a Model<S, Item>,
    on_selected: impl Fn(Item) -> Message + 'a,
) -> Dropdown<'a, S, Message, Item> {
    Dropdown::new(model, on_selected)
}
