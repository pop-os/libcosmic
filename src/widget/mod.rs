// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! The COSMIC widget library
//!
//! This module contains a wide variety of widgets used throughout the COSMIC app ecosystem.
//!
//! # Overview
//!
//! Add widgets to your application view by calling the modules and functions below.
//! Widgets are constructed by chaining their property methods using a functional paradigm.
//! Modules may contain additional functions for constructing different variations of a widget.
//! Each module will typically have one widget with the same name as the module, which will be re-exported here.
//!
//! ```no_run,ignore
//! use cosmic::prelude::*;
//! use cosmic::{cosmic_theme, theme, widget};
//!
//! const REPOSITORY: &str = "https://github.com/pop-os/libcosmic";
//!
//! let cosmic_theme::Spacing { space_xxs, .. } = theme::spacing();
//!
//! let link = widget::button::link(REPOSITORY)
//!     .on_press(Message::LaunchUrl(REPOSITORY))
//!     .padding(0);
//!
//! let content = widget::column()
//!     .push(widget::icon::from_name("my-app-icon"))
//!     .push(widget::text::title3("My App Name"))
//!     .push(link)
//!     .align_items(Alignment::Center)
//!     .spacing(space_xxs);
//! ```
//!
//! Widgets may borrow data from your application struct, and should do so to avoid allocating.
//!
//! ```no_run,ignore
//! let text = widget::text::body(&self.cached_text);
//! ```
//!
//! Use the [`cosmic::Apply`](crate::Apply) trait to embed widgets into other widgets which accept them.
//!
//! ```no_run,ignore
//! let button = widget::icon::from_name("printer-symbolic")
//!     .apply(widget::button::icon)
//!     .on_press(Message::Print);
//! ```

// Re-exports from Iced
#[doc(inline)]
pub use iced::widget::{Canvas, canvas};

#[doc(inline)]
pub use iced::widget::{Checkbox, checkbox};

#[doc(inline)]
pub use iced::widget::{ComboBox, combo_box};

#[doc(inline)]
pub use iced::widget::{Container, container};

#[doc(inline)]
pub use iced::widget::{Space, horizontal_space, vertical_space};

#[doc(inline)]
pub use iced::widget::{Image, image};

#[doc(inline)]
pub use iced::widget::{Lazy, lazy};

#[doc(inline)]
pub use iced::widget::{MouseArea, mouse_area};

#[doc(inline)]
pub use iced::widget::{PaneGrid, pane_grid};

#[doc(inline)]
pub use iced::widget::{ProgressBar, progress_bar};

#[doc(inline)]
pub use iced::widget::{Responsive, responsive};

#[doc(inline)]
pub use iced::widget::{Slider, VerticalSlider, slider, vertical_slider};

#[doc(inline)]
pub use iced::widget::{Svg, svg};

#[doc(inline)]
pub use iced::widget::{TextEditor, text_editor};

#[doc(inline)]
pub use iced_core::widget::{Id, Operation, Widget};

pub mod aspect_ratio;

#[cfg(feature = "autosize")]
pub mod autosize;

pub(crate) mod responsive_container;

#[cfg(feature = "surface-message")]
mod responsive_menu_bar;
#[cfg(feature = "surface-message")]
#[doc(inline)]
pub use responsive_menu_bar::{ResponsiveMenuBar, responsive_menu_bar};

pub mod button;
#[doc(inline)]
pub use button::{Button, IconButton, LinkButton, TextButton};

pub(crate) mod common;

pub mod calendar;
#[doc(inline)]
pub use calendar::{Calendar, calendar};

pub mod card;
#[doc(inline)]
pub use card::*;

pub mod color_picker;
#[doc(inline)]
pub use color_picker::{ColorPicker, ColorPickerModel};

#[cfg(feature = "qr_code")]
#[doc(inline)]
pub use iced::widget::qr_code;

pub mod context_drawer;
#[doc(inline)]
pub use context_drawer::{ContextDrawer, context_drawer};

#[doc(inline)]
pub use column::{Column, column};
pub mod column {
    //! A container which aligns its children in a column.

    pub type Column<'a, Message> = iced::widget::Column<'a, Message, crate::Theme, crate::Renderer>;

    #[must_use]
    /// A container which aligns its children in a column.
    pub fn column<'a, Message>() -> Column<'a, Message> {
        Column::new()
    }

    #[must_use]
    /// A pre-allocated [`column`].
    pub fn with_capacity<'a, Message>(capacity: usize) -> Column<'a, Message> {
        Column::with_capacity(capacity)
    }

    #[must_use]
    /// A [`column`] that will be assigned an [`Iterator`] of children.
    pub fn with_children<'a, Message>(
        children: impl IntoIterator<Item = crate::Element<'a, Message>>,
    ) -> Column<'a, Message> {
        Column::with_children(children)
    }
}

pub mod layer_container;
#[doc(inline)]
pub use layer_container::{LayerContainer, layer_container};

pub mod context_menu;
#[doc(inline)]
pub use context_menu::{ContextMenu, context_menu};

pub mod dialog;
#[doc(inline)]
pub use dialog::{Dialog, dialog};

/// An element to distinguish a boundary between two elements.
pub mod divider {
    /// Horizontal variant of a divider.
    pub mod horizontal {
        use iced::widget::{Rule, horizontal_rule};

        /// Horizontal divider with default thickness
        #[must_use]
        pub fn default<'a>() -> Rule<'a, crate::Theme> {
            horizontal_rule(1).class(crate::theme::Rule::Default)
        }

        /// Horizontal divider with light thickness
        #[must_use]
        pub fn light<'a>() -> Rule<'a, crate::Theme> {
            horizontal_rule(1).class(crate::theme::Rule::LightDivider)
        }

        /// Horizontal divider with heavy thickness.
        #[must_use]
        pub fn heavy<'a>() -> Rule<'a, crate::Theme> {
            horizontal_rule(4).class(crate::theme::Rule::HeavyDivider)
        }
    }

    /// Vertical variant of a divider.
    pub mod vertical {
        use iced::widget::{Rule, vertical_rule};

        /// Vertical divider with default thickness
        #[must_use]
        pub fn default<'a>() -> Rule<'a, crate::Theme> {
            vertical_rule(1).class(crate::theme::Rule::Default)
        }

        /// Vertical divider with light thickness
        #[must_use]
        pub fn light<'a>() -> Rule<'a, crate::Theme> {
            vertical_rule(4).class(crate::theme::Rule::LightDivider)
        }

        /// Vertical divider with heavy thickness.
        #[must_use]
        pub fn heavy<'a>() -> Rule<'a, crate::Theme> {
            vertical_rule(10).class(crate::theme::Rule::HeavyDivider)
        }
    }
}

pub mod dnd_destination;
#[doc(inline)]
pub use dnd_destination::{DndDestination, dnd_destination};

pub mod dnd_source;
#[doc(inline)]
pub use dnd_source::{DndSource, dnd_source};

pub mod dropdown;
#[doc(inline)]
pub use dropdown::{Dropdown, dropdown};

pub mod flex_row;
#[doc(inline)]
pub use flex_row::{FlexRow, flex_row};

pub mod grid;
#[doc(inline)]
pub use grid::{Grid, grid};

mod header_bar;
#[doc(inline)]
pub use header_bar::{HeaderBar, header_bar};

pub mod icon;
#[doc(inline)]
pub use icon::{Icon, icon};

pub mod id_container;
#[doc(inline)]
pub use id_container::{IdContainer, id_container};

#[cfg(feature = "animated-image")]
pub mod frames;

pub use taffy::JustifyContent;

pub mod list;
#[doc(inline)]
pub use list::{ListColumn, list_column};

pub mod menu;

pub mod nav_bar;
#[doc(inline)]
pub use nav_bar::{nav_bar, nav_bar_dnd};

pub mod nav_bar_toggle;
#[doc(inline)]
pub use nav_bar_toggle::{NavBarToggle, nav_bar_toggle};

pub mod popover;
#[doc(inline)]
pub use popover::{Popover, popover};

pub mod radio;
#[doc(inline)]
pub use radio::{Radio, radio};

pub mod rectangle_tracker;
#[doc(inline)]
pub use rectangle_tracker::{RectangleTracker, rectangle_tracking_container};

#[doc(inline)]
pub use row::{Row, row};

pub mod row {
    //! A container which aligns its children in a row.

    pub type Row<'a, Message> = iced::widget::Row<'a, Message, crate::Theme, crate::Renderer>;

    #[must_use]
    /// A container which aligns its children in a row.
    pub fn row<'a, Message>() -> Row<'a, Message> {
        Row::new()
    }

    #[must_use]
    /// A pre-allocated [`row`].
    pub fn with_capacity<'a, Message>(capacity: usize) -> Row<'a, Message> {
        Row::with_capacity(capacity)
    }

    #[must_use]
    /// A [`row`] that will be assigned an [`Iterator`] of children.
    pub fn with_children<'a, Message>(
        children: impl IntoIterator<Item = crate::Element<'a, Message>>,
    ) -> Row<'a, Message> {
        Row::with_children(children)
    }
}

pub mod scrollable;
#[doc(inline)]
pub use scrollable::scrollable;
pub mod segmented_button;
pub mod segmented_control;

pub mod settings;

pub mod spin_button;
#[doc(inline)]
pub use spin_button::{SpinButton, spin_button, vertical as vertical_spin_button};

pub mod tab_bar;

pub mod table;
#[doc(inline)]
pub use table::{compact_table, table};

pub mod text;
#[doc(inline)]
pub use text::{Text, text};

pub mod text_input;
#[doc(inline)]
pub use text_input::{
    TextInput, editable_input, inline_input, search_input, secure_input, text_input,
};

pub mod toaster;
#[doc(inline)]
pub use toaster::{Toast, ToastId, Toasts, toaster};

mod toggler;
#[doc(inline)]
pub use toggler::toggler;

#[doc(inline)]
pub use tooltip::{Tooltip, tooltip};

#[cfg(all(feature = "wayland", feature = "winit"))]
pub mod wayland;

pub mod tooltip {
    use crate::Element;

    pub use iced::widget::tooltip::Position;

    pub type Tooltip<'a, Message> =
        iced::widget::Tooltip<'a, Message, crate::Theme, crate::Renderer>;

    pub fn tooltip<'a, Message>(
        content: impl Into<Element<'a, Message>>,
        tooltip: impl Into<Element<'a, Message>>,
        position: Position,
    ) -> Tooltip<'a, Message> {
        let xxs = crate::theme::spacing().space_xxs;

        Tooltip::new(content, tooltip, position)
            .class(crate::theme::Container::Tooltip)
            .padding(xxs)
            .gap(1)
    }
}

pub mod warning;
#[doc(inline)]
pub use warning::*;

pub mod wrapper;
#[doc(inline)]
pub use wrapper::*;

#[cfg(feature = "markdown")]
#[doc(inline)]
pub use iced::widget::markdown;

#[cfg(feature = "about")]
pub mod about;
#[cfg(feature = "about")]
#[doc(inline)]
pub use about::about;
