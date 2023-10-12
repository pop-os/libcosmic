// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::Theme;

/// Status of the nav bar and its panels.
#[derive(Clone)]
pub struct NavBar {
    active: bool,
    toggled: bool,
    toggled_condensed: bool,
}

/// COSMIC-specific settings for windows.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone)]
pub struct Window {
    /// Label to display as context drawer title.
    pub context_title: String,
    /// Label to display as header bar title.
    pub header_title: String,
    pub use_template: bool,
    pub can_fullscreen: bool,
    pub sharp_corners: bool,
    pub show_context: bool,
    pub show_headerbar: bool,
    pub show_window_menu: bool,
    pub show_maximize: bool,
    pub show_minimize: bool,
    height: u32,
    width: u32,
}

/// COSMIC-specific application settings
#[derive(Clone)]
pub struct Core {
    /// Enables debug features in cosmic/iced.
    pub debug: bool,

    /// Whether the window is too small for the nav bar + main content.
    is_condensed: bool,

    /// Current status of the nav bar panel.
    nav_bar: NavBar,

    /// Scaling factor used by the application
    scale_factor: f32,

    /// Last known system theme
    pub(super) system_theme: Theme,

    pub(super) title: String,

    pub window: Window,

    #[cfg(feature = "applet")]
    pub applet: crate::applet::Context,
}

impl Default for Core {
    fn default() -> Self {
        Self {
            debug: false,
            is_condensed: false,
            nav_bar: NavBar {
                active: true,
                toggled: true,
                toggled_condensed: true,
            },
            scale_factor: 1.0,
            title: String::new(),
            system_theme: crate::theme::active(),
            window: Window {
                context_title: String::new(),
                header_title: String::new(),
                use_template: true,
                can_fullscreen: false,
                sharp_corners: false,
                show_context: false,
                show_headerbar: true,
                show_maximize: true,
                show_minimize: true,
                show_window_menu: false,
                height: 0,
                width: 0,
            },
            #[cfg(feature = "applet")]
            applet: crate::applet::Context::default(),
        }
    }
}

impl Core {
    /// Whether the window is too small for the nav bar + main content.
    #[must_use]
    pub fn is_condensed(&self) -> bool {
        self.is_condensed
    }

    /// The scaling factor used by the application.
    #[must_use]
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Changes the scaling factor used by the application.
    pub(crate) fn set_scale_factor(&mut self, factor: f32) {
        self.scale_factor = factor;
        self.is_condensed_update();
    }

    /// Set context drawer header title
    pub fn set_context_title(&mut self, title: String) {
        self.window.context_title = title;
    }

    /// Set header bar title
    pub fn set_header_title(&mut self, title: String) {
        self.window.header_title = title;
    }

    /// Whether to show or hide the main window's content.
    pub(crate) fn show_content(&self) -> bool {
        !self.is_condensed || !self.nav_bar.toggled_condensed
    }

    /// Call this whenever the scaling factor or window width has changed.
    #[allow(clippy::cast_precision_loss)]
    fn is_condensed_update(&mut self) {
        self.is_condensed = (600.0 * self.scale_factor) > self.window.width as f32;
        self.nav_bar_update();
    }

    /// Whether the nav panel is visible or not
    #[must_use]
    pub fn nav_bar_active(&self) -> bool {
        self.nav_bar.active
    }

    pub fn nav_bar_toggle(&mut self) {
        self.nav_bar.toggled = !self.nav_bar.toggled;
        self.nav_bar_set_toggled_condensed(self.nav_bar.toggled);
    }

    pub fn nav_bar_toggle_condensed(&mut self) {
        self.nav_bar_set_toggled_condensed(!self.nav_bar.toggled_condensed);
    }

    pub(crate) fn nav_bar_set_toggled_condensed(&mut self, toggled: bool) {
        self.nav_bar.toggled_condensed = toggled;
        self.nav_bar_update();
    }

    pub(crate) fn nav_bar_update(&mut self) {
        self.nav_bar.active = if self.is_condensed {
            self.nav_bar.toggled_condensed
        } else {
            self.nav_bar.toggled
        };
    }

    /// Set the height of the main window.
    pub(crate) fn set_window_height(&mut self, new_height: u32) {
        self.window.height = new_height;
    }

    /// Set the width of the main window.
    pub(crate) fn set_window_width(&mut self, new_width: u32) {
        self.window.width = new_width;
        self.is_condensed_update();
    }
}
