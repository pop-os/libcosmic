// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Cosmic-themed widget implementations.

mod button;
pub use button::*;

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

mod toggler;
pub use toggler::toggler;

pub mod segmented_button;
pub use segmented_button::horizontal as horizontal_segmented_button;
pub use segmented_button::vertical as vertical_segmented_button;

pub mod segmented_selection;
pub use segmented_selection::horizontal as horizontal_segmented_selection;
pub use segmented_selection::vertical as vertical_segmented_selection;

pub mod settings;

mod scrollable;
pub use scrollable::*;

pub mod separator;
pub use separator::{horizontal_rule, vertical_rule};

pub mod spin_button;
pub use spin_button::{spin_button, SpinButton};

pub mod rectangle_tracker;

pub mod aspect_ratio;

pub mod view_switcher;
pub use view_switcher::horizontal as horiontal_view_switcher;
pub use view_switcher::vertical as vertical_view_switcher;

pub mod warning;
pub use warning::*;
