// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Cosmic-themed widget implementations.

pub mod aspect_ratio;

mod button;
pub use button::*;

pub mod card;
pub use card::*;

pub mod flex_row;
pub use flex_row::{flex_row, FlexRow};

mod header_bar;
pub use header_bar::{header_bar, HeaderBar};

pub mod icon;
pub use icon::{icon, Icon, IconSource};

pub mod list;
pub use list::*;

pub mod nav_bar;
pub use nav_bar::nav_bar;

pub mod nav_bar_toggle;
pub use nav_bar_toggle::{nav_bar_toggle, NavBarToggle};

pub mod popover;
pub use popover::{popover, Popover};

pub mod rectangle_tracker;

pub mod search;

pub mod segmented_button;
pub use segmented_button::horizontal as horizontal_segmented_button;
pub use segmented_button::vertical as vertical_segmented_button;

pub mod segmented_selection;
pub use segmented_selection::horizontal as horizontal_segmented_selection;
pub use segmented_selection::vertical as vertical_segmented_selection;

pub mod settings;

mod scrollable;
pub use scrollable::*;

pub mod spin_button;
pub use spin_button::{spin_button, SpinButton};

mod text;
pub use text::{text, Text};

mod toggler;
pub use toggler::toggler;

pub mod view_switcher;
pub use view_switcher::horizontal as horiontal_view_switcher;
pub use view_switcher::vertical as vertical_view_switcher;

pub mod warning;
pub use warning::*;

pub mod cosmic_container;
pub use cosmic_container::*;

/// An element to distinguish a boundary between two elements.
pub mod divider {
    /// Horizontal variant of a divider.
    pub mod horizontal {
        use iced::widget::{horizontal_rule, Rule};

        /// Horizontal divider with light thickness
        #[must_use]
        pub fn light() -> Rule<crate::Renderer> {
            horizontal_rule(4).style(crate::theme::Rule::LightDivider)
        }

        /// Horizontal divider with heavy thickness.
        #[must_use]
        pub fn heavy() -> Rule<crate::Renderer> {
            horizontal_rule(10).style(crate::theme::Rule::HeavyDivider)
        }
    }
}
