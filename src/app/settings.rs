// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Configure a new COSMIC application.

use crate::{font, Theme};
#[cfg(feature = "wayland")]
use iced_core::layout::Limits;
use iced_core::Font;

/// Configure a new COSMIC application.
#[allow(clippy::struct_excessive_bools)]
#[must_use]
#[derive(derive_setters::Setters)]
pub struct Settings {
    /// Produces a smoother result in some widgets, at a performance cost.
    pub(crate) antialiasing: bool,

    /// Autosize the window to fit its contents
    #[cfg(feature = "wayland")]
    pub(crate) autosize: bool,

    /// Set the application to not create a main window
    #[cfg(feature = "wayland")]
    pub(crate) no_main_window: bool,

    /// Whether the window should have a border, a title bar, etc. or not.
    pub(crate) client_decorations: bool,

    /// Enables debug features in cosmic/iced.
    pub(crate) debug: bool,

    /// The default [`Font`] to be used.
    pub(crate) default_font: Font,

    /// Name of the icon theme to search by default.
    #[setters(skip)]
    pub(crate) default_icon_theme: Option<String>,

    /// Default size of fonts.
    pub(crate) default_text_size: f32,

    /// Whether the window should be resizable or not.
    /// and the size of the window border which can be dragged for a resize
    pub(crate) resizable: Option<f64>,

    /// Scale factor to use by default.
    pub(crate) scale_factor: f32,

    /// Initial size of the window.
    pub(crate) size: iced::Size,

    /// Limitations of the window size
    #[cfg(feature = "wayland")]
    pub(crate) size_limits: Limits,

    /// The theme to apply to the application.
    pub(crate) theme: Theme,

    /// Whether the window should be transparent.
    pub(crate) transparent: bool,

    /// Whether the application should exit when there are no open windows
    pub(crate) exit_on_close: bool,
}

impl Settings {
    /// Sets the default icon theme, passing an empty string will unset the theme.
    pub fn default_icon_theme(mut self, value: impl Into<String>) -> Self {
        let value: String = value.into();
        self.default_icon_theme = if value.is_empty() { None } else { Some(value) };
        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            antialiasing: true,
            #[cfg(feature = "wayland")]
            autosize: false,
            #[cfg(feature = "wayland")]
            no_main_window: false,
            client_decorations: true,
            debug: false,
            default_font: font::FONT,
            default_icon_theme: Some(String::from("Cosmic")),
            default_text_size: 14.0,
            resizable: Some(8.0),
            scale_factor: std::env::var("COSMIC_SCALE")
                .ok()
                .and_then(|scale| scale.parse::<f32>().ok())
                .unwrap_or(1.0),
            size: iced::Size::new(1024.0, 768.0),
            #[cfg(feature = "wayland")]
            size_limits: Limits::NONE.min_height(1.0).min_width(1.0),
            theme: crate::theme::system_preference(),
            transparent: false,
            exit_on_close: true,
        }
    }
}
