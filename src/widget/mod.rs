// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Cosmic-themed widget implementations.

// Re-exports from Iced
#[doc(inline)]
pub use iced::widget::{checkbox, Checkbox};

#[doc(inline)]
pub use iced::widget::{combo_box, ComboBox};

#[doc(inline)]
pub use iced::widget::{container, Container};

#[doc(inline)]
pub use iced::widget::{horizontal_space, space, vertical_space, Space};

#[doc(inline)]
pub use iced::widget::{image, Image};

#[doc(inline)]
pub use iced::widget::{lazy, Lazy};

#[doc(inline)]
pub use iced::widget::{mouse_area, MouseArea};

#[doc(inline)]
pub use iced::widget::{pane_grid, PaneGrid};

#[doc(inline)]
pub use iced::widget::{progress_bar, ProgressBar};

#[doc(inline)]
pub use iced::widget::{radio, Radio};

#[doc(inline)]
pub use iced::widget::{responsive, Responsive};

#[doc(inline)]
pub use iced::widget::{slider, vertical_slider, Slider, VerticalSlider};

#[doc(inline)]
pub use iced::widget::{svg, Svg};

#[doc(inline)]
pub use iced_core::widget::{Id, Operation, Widget};

pub mod aspect_ratio;

pub mod button;
#[doc(inline)]
pub use button::{button, Button, IconButton, LinkButton, TextButton};

pub(crate) mod common;

pub mod calendar;
#[doc(inline)]
pub use calendar::{calendar, Calendar};

pub mod card;
#[doc(inline)]
pub use card::*;

pub mod color_picker;
#[doc(inline)]
pub use color_picker::{ColorPicker, ColorPickerModel};

pub mod context_drawer;
#[doc(inline)]
pub use context_drawer::{context_drawer, ContextDrawer};

#[doc(inline)]
pub use column::{column, Column};
pub mod column {
    pub type Column<'a, Message> = iced::widget::Column<'a, Message, crate::Theme, crate::Renderer>;

    #[must_use]
    pub fn column<'a, Message>() -> Column<'a, Message> {
        Column::new()
    }

    #[must_use]
    pub fn with_capacity<'a, Message>(capacity: usize) -> Column<'a, Message> {
        Column::with_children(Vec::with_capacity(capacity))
    }

    #[must_use]
    pub fn with_children<Message>(children: Vec<crate::Element<Message>>) -> Column<Message> {
        Column::with_children(children)
    }
}

pub mod layer_container;
#[doc(inline)]
pub use layer_container::{layer_container, LayerContainer};

pub mod context_menu;
#[doc(inline)]
pub use context_menu::{context_menu, ContextMenu};

pub mod dialog;
#[doc(inline)]
pub use dialog::{dialog, Dialog};

/// An element to distinguish a boundary between two elements.
pub mod divider {
    /// Horizontal variant of a divider.
    pub mod horizontal {
        use iced::widget::{horizontal_rule, Rule};

        /// Horizontal divider with default thickness
        #[must_use]
        pub fn default() -> Rule<crate::Theme> {
            horizontal_rule(1).style(crate::theme::Rule::Default)
        }

        /// Horizontal divider with light thickness
        #[must_use]
        pub fn light() -> Rule<crate::Theme> {
            horizontal_rule(4).style(crate::theme::Rule::LightDivider)
        }

        /// Horizontal divider with heavy thickness.
        #[must_use]
        pub fn heavy() -> Rule<crate::Theme> {
            horizontal_rule(10).style(crate::theme::Rule::HeavyDivider)
        }
    }

    /// Vertical variant of a divider.
    pub mod vertical {
        use iced::widget::{vertical_rule, Rule};

        /// Vertical divider with default thickness
        #[must_use]
        pub fn default() -> Rule<crate::Theme> {
            vertical_rule(1).style(crate::theme::Rule::Default)
        }

        /// Vertical divider with light thickness
        #[must_use]
        pub fn light() -> Rule<crate::Theme> {
            vertical_rule(4).style(crate::theme::Rule::LightDivider)
        }

        /// Vertical divider with heavy thickness.
        #[must_use]
        pub fn heavy() -> Rule<crate::Theme> {
            vertical_rule(10).style(crate::theme::Rule::HeavyDivider)
        }
    }
}

pub mod dnd_destination;
#[doc(inline)]
pub use dnd_destination::{dnd_destination, DndDestination};

pub mod dnd_source;
#[doc(inline)]
pub use dnd_source::{dnd_source, DndSource};

pub mod dropdown;
#[doc(inline)]
pub use dropdown::{dropdown, Dropdown};

pub mod flex_row;
#[doc(inline)]
pub use flex_row::{flex_row, FlexRow};

pub mod grid;
#[doc(inline)]
pub use grid::{grid, Grid};

mod header_bar;
#[doc(inline)]
pub use header_bar::{header_bar, HeaderBar};

pub mod icon;
#[doc(inline)]
pub use icon::{icon, Icon};

#[cfg(feature = "animated-image")]
pub mod frames;

pub mod list;
#[doc(inline)]
pub use list::{list_column, ListColumn};

pub mod menu;

pub mod nav_bar;
#[doc(inline)]
pub use nav_bar::{nav_bar, nav_bar_dnd};

pub mod nav_bar_toggle;
#[doc(inline)]
pub use nav_bar_toggle::{nav_bar_toggle, NavBarToggle};

pub mod popover;
#[doc(inline)]
pub use popover::{popover, Popover};

pub mod rectangle_tracker;
#[doc(inline)]
pub use rectangle_tracker::{rectangle_tracker, RectangleTracker};

#[doc(inline)]
pub use row::{row, Row};
pub mod row {
    pub type Row<'a, Message> = iced::widget::Row<'a, Message, crate::Theme, crate::Renderer>;

    #[must_use]
    pub fn row<'a, Message>() -> Row<'a, Message> {
        Row::new()
    }

    #[must_use]
    pub fn with_capacity<'a, Message>(capacity: usize) -> Row<'a, Message> {
        Row::with_children(Vec::with_capacity(capacity))
    }

    #[must_use]
    pub fn with_children<Message>(children: Vec<crate::Element<Message>>) -> Row<Message> {
        Row::with_children(children)
    }
}

mod scrollable;
#[doc(inline)]
pub use scrollable::*;

pub mod segmented_button;
pub mod segmented_control;

pub mod settings;

pub mod spin_button;
#[doc(inline)]
pub use spin_button::{spin_button, SpinButton};

pub mod tab_bar;

pub mod text;
#[doc(inline)]
pub use text::{text, Text};

pub mod text_input;
#[doc(inline)]
pub use text_input::{
    editable_input, inline_input, search_input, secure_input, text_input, TextInput,
};

mod toggler;
#[doc(inline)]
pub use toggler::toggler;

#[doc(inline)]
pub use tooltip::{tooltip, Tooltip};
pub mod tooltip {
    use crate::Element;
    use std::borrow::Cow;

    pub use iced::widget::tooltip::Position;

    pub type Tooltip<'a, Message> =
        iced::widget::Tooltip<'a, Message, crate::Theme, crate::Renderer>;

    pub fn tooltip<'a, Message>(
        content: impl Into<Element<'a, Message>>,
        tooltip: impl Into<Cow<'a, str>>,
        position: Position,
    ) -> Tooltip<'a, Message> {
        let xxs = crate::theme::active().cosmic().space_xxs();

        Tooltip::new(content, tooltip, position)
            .style(crate::theme::Container::Tooltip)
            .padding(xxs)
            .gap(1)
    }
}

pub mod warning;
#[doc(inline)]
pub use warning::*;
