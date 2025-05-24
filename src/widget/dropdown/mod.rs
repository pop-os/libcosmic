// Copyright 2023 System76 <info@system76.com>
// Copyright 2019 Héctor Ramón, Iced contributors
// SPDX-License-Identifier: MPL-2.0 AND MIT

//! Displays a list of options in a popover menu on select.

use std::borrow::Cow;

pub mod menu;
use iced_core::window;
pub use menu::Menu;

pub mod multi;

mod widget;
pub use widget::*;

use crate::surface;

/// Displays a list of options in a popover menu on select.
pub fn dropdown<
    'a,
    S: AsRef<str> + std::clone::Clone + Send + Sync + 'static,
    Message: 'static + Clone,
>(
    selections: impl Into<Cow<'a, [S]>>,
    selected: Option<usize>,
    on_selected: impl Fn(usize) -> Message + Send + Sync + 'static,
) -> Dropdown<'a, S, Message, Message> {
    Dropdown::new(selections.into(), selected, on_selected)
}

/// Displays a list of options in a popover menu on select.
/// AppMessage must be the App's toplevel message.
pub fn popup_dropdown<
    'a,
    S: AsRef<str> + std::clone::Clone + Send + Sync + 'static,
    Message: 'static + Clone,
    AppMessage: 'static + Clone,
>(
    selections: impl Into<Cow<'a, [S]>>,
    selected: Option<usize>,
    on_selected: impl Fn(usize) -> Message + Send + Sync + 'static,
    _parent_id: window::Id,
    _on_surface_action: impl Fn(surface::Action) -> Message + Send + Sync + 'static,
    _map_action: impl Fn(Message) -> AppMessage + Send + Sync + 'static,
) -> Dropdown<'a, S, Message, AppMessage> {
    let dropdown: Dropdown<'_, S, Message, AppMessage> =
        Dropdown::new(selections.into(), selected, on_selected);

    #[cfg(all(feature = "winit", feature = "wayland"))]
    let dropdown = dropdown.with_popup(_parent_id, _on_surface_action, _map_action);

    dropdown
}
