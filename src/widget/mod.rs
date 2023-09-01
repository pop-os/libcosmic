// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Cosmic-themed widget implementations.

// Re-exports from Iced
pub use iced::widget::{checkbox, Checkbox};
pub use iced::widget::{container, Container};
pub use iced::widget::{horizontal_space, space, vertical_space, Space};
pub use iced::widget::{image, Image};
pub use iced::widget::{mouse_area, MouseArea};
pub use iced::widget::{pick_list, PickList};
pub use iced::widget::{radio, Radio};
pub use iced::widget::{slider, Slider};
pub use iced::widget::{svg, Svg};

pub mod aspect_ratio;

mod button;
pub use button::*;

pub mod card;
pub use card::*;

pub use column::{column, Column};
pub mod column {
    pub use iced::widget::Column;

    #[must_use]
    pub fn column<'a, Message>() -> Column<'a, Message, crate::Renderer> {
        Column::new()
    }

    #[must_use]
    pub fn with_capacity<'a, Message>(capacity: usize) -> Column<'a, Message, crate::Renderer> {
        Column::with_children(Vec::with_capacity(capacity))
    }

    #[must_use]
    pub fn with_children<Message>(
        children: Vec<crate::Element<Message>>,
    ) -> Column<Message, crate::Renderer> {
        Column::with_children(children)
    }
}

pub mod cosmic_container;
pub use cosmic_container::LayerContainer;

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

pub mod flex_row;
pub use flex_row::{flex_row, FlexRow};

mod header_bar;
pub use header_bar::{header_bar, HeaderBar};

pub mod icon;
pub use icon::{icon, Icon};

#[cfg(feature = "animated-image")]
pub mod frames;

pub mod list;
pub use list::*;

pub mod nav_bar;
pub use nav_bar::nav_bar;

pub mod nav_bar_toggle;
pub use nav_bar_toggle::{nav_bar_toggle, NavBarToggle};

pub mod popover;
pub use popover::{popover, Popover};

pub mod rectangle_tracker;

pub use row::{row, Row};
pub mod row {
    pub use iced::widget::Row;

    #[must_use]
    pub fn row<'a, Message>() -> Row<'a, Message, crate::Renderer> {
        Row::new()
    }

    #[must_use]
    pub fn with_capacity<'a, Message>(capacity: usize) -> Row<'a, Message, crate::Renderer> {
        Row::with_children(Vec::with_capacity(capacity))
    }

    #[must_use]
    pub fn with_children<Message>(
        children: Vec<crate::Element<Message>>,
    ) -> Row<Message, crate::Renderer> {
        Row::with_children(children)
    }
}

mod scrollable;
pub use scrollable::*;

pub mod search;

pub mod segmented_button;
pub mod segmented_selection;

pub mod settings;

pub mod spin_button;
pub use spin_button::{spin_button, SpinButton};

pub mod text;
pub use text::{text, Text};

pub mod text_input;
pub use text_input::*;

mod toggler;
pub use toggler::toggler;

pub mod view_switcher;
pub use view_switcher::horizontal as horiontal_view_switcher;
pub use view_switcher::vertical as vertical_view_switcher;

pub mod warning;
pub use warning::*;
