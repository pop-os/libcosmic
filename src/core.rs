// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashMap;

use crate::widget::nav_bar;
use cosmic_config::CosmicConfigEntry;
use cosmic_theme::ThemeMode;
use iced::{Limits, Size, window};
use iced_core::window::Id;
use palette::Srgba;
use slotmap::Key;

use crate::Theme;

/// Status of the nav bar and its panels.
#[derive(Clone)]
pub struct NavBar {
    active: bool,
    context_id: crate::widget::nav_bar::Id,
    toggled: bool,
    toggled_condensed: bool,
}

/// COSMIC-specific settings for windows.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone)]
pub struct Window {
    /// Label to display as header bar title.
    pub header_title: String,
    pub use_template: bool,
    pub content_container: bool,
    pub context_is_overlay: bool,
    pub sharp_corners: bool,
    pub show_context: bool,
    pub show_headerbar: bool,
    pub show_window_menu: bool,
    pub show_close: bool,
    pub show_maximize: bool,
    pub show_minimize: bool,
    pub is_maximized: bool,
    height: f32,
    width: f32,
}

/// COSMIC-specific application settings
#[derive(Clone)]
pub struct Core {
    /// Enables debug features in cosmic/iced.
    pub debug: bool,

    /// Disables loading the icon theme from cosmic-config.
    pub(super) icon_theme_override: bool,

    /// Whether the window is too small for the nav bar + main content.
    is_condensed: bool,

    /// Enables built in keyboard navigation
    pub(super) keyboard_nav: bool,

    /// Current status of the nav bar panel.
    nav_bar: NavBar,

    /// Scaling factor used by the application
    scale_factor: f32,

    /// Window focus state
    pub(super) focused_window: Vec<window::Id>,

    pub(super) theme_sub_counter: u64,
    /// Last known system theme
    pub(super) system_theme: Theme,

    /// Configured theme mode
    pub(super) system_theme_mode: ThemeMode,

    pub(super) portal_is_dark: Option<bool>,

    pub(super) portal_accent: Option<Srgba>,

    pub(super) portal_is_high_contrast: Option<bool>,

    pub(super) title: HashMap<Id, String>,

    pub window: Window,

    #[cfg(feature = "applet")]
    pub applet: crate::applet::Context,

    #[cfg(feature = "single-instance")]
    pub(crate) single_instance: bool,

    #[cfg(all(feature = "dbus-config", target_os = "linux"))]
    pub(crate) settings_daemon: Option<cosmic_settings_daemon::CosmicSettingsDaemonProxy<'static>>,

    pub(crate) main_window: Option<window::Id>,

    pub(crate) exit_on_main_window_closed: bool,

    pub(crate) menu_bars: HashMap<crate::widget::Id, (Limits, Size)>,

    #[cfg(feature = "wayland")]
    pub(crate) sync_window_border_radii_to_theme: bool,
}

impl Default for Core {
    fn default() -> Self {
        Self {
            debug: false,
            icon_theme_override: false,
            is_condensed: false,
            keyboard_nav: true,
            nav_bar: NavBar {
                active: true,
                context_id: crate::widget::nav_bar::Id::null(),
                toggled: true,
                toggled_condensed: false,
            },
            scale_factor: 1.0,
            title: HashMap::new(),
            theme_sub_counter: 0,
            system_theme: crate::theme::active(),
            system_theme_mode: ThemeMode::config()
                .map(|c| {
                    ThemeMode::get_entry(&c).unwrap_or_else(|(errors, mode)| {
                        for why in errors.into_iter().filter(cosmic_config::Error::is_err) {
                            tracing::error!(?why, "ThemeMode config entry error");
                        }
                        mode
                    })
                })
                .unwrap_or_default(),
            window: Window {
                header_title: String::new(),
                use_template: true,
                content_container: true,
                context_is_overlay: true,
                sharp_corners: false,
                show_context: false,
                show_headerbar: true,
                show_close: true,
                show_maximize: true,
                show_minimize: true,
                show_window_menu: false,
                is_maximized: false,
                height: 0.,
                width: 0.,
            },
            focused_window: Vec::new(),
            #[cfg(feature = "applet")]
            applet: crate::applet::Context::default(),
            #[cfg(feature = "single-instance")]
            single_instance: false,
            #[cfg(all(feature = "dbus-config", target_os = "linux"))]
            settings_daemon: None,
            portal_is_dark: None,
            portal_accent: None,
            portal_is_high_contrast: None,
            main_window: None,
            exit_on_main_window_closed: true,
            menu_bars: HashMap::new(),
            #[cfg(feature = "wayland")]
            sync_window_border_radii_to_theme: true,
        }
    }
}

impl Core {
    /// Whether the window is too small for the nav bar + main content.
    #[must_use]
    #[inline]
    pub const fn is_condensed(&self) -> bool {
        self.is_condensed
    }

    /// The scaling factor used by the application.
    #[must_use]
    #[inline]
    pub const fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Enable or disable keyboard navigation
    #[inline]
    pub const fn set_keyboard_nav(&mut self, enabled: bool) {
        self.keyboard_nav = enabled;
    }

    /// Enable or disable keyboard navigation
    #[must_use]
    #[inline]
    pub const fn keyboard_nav(&self) -> bool {
        self.keyboard_nav
    }

    /// Changes the scaling factor used by the application.
    #[cold]
    pub(crate) fn set_scale_factor(&mut self, factor: f32) {
        self.scale_factor = factor;
        self.is_condensed_update();
    }

    /// Set header bar title
    #[inline]
    pub fn set_header_title(&mut self, title: String) {
        self.window.header_title = title;
    }

    #[inline]
    /// Whether to show or hide the main window's content.
    pub(crate) fn show_content(&self) -> bool {
        !self.is_condensed || !self.nav_bar.toggled_condensed
    }

    #[allow(clippy::cast_precision_loss)]
    /// Call this whenever the scaling factor or window width has changed.
    fn is_condensed_update(&mut self) {
        // Nav bar (280px) + padding (8px) + content (360px)
        let mut breakpoint = 280.0 + 8.0 + 360.0;
        //TODO: the app may return None from the context_drawer function even if show_context is true
        if self.window.show_context && !self.window.context_is_overlay {
            // Context drawer min width (344px) + padding (8px)
            breakpoint += 344.0 + 8.0;
        };
        self.is_condensed = (breakpoint * self.scale_factor) > self.window.width;
        self.nav_bar_update();
    }

    #[inline]
    fn condensed_conflict(&self) -> bool {
        // There is a conflict if the view is condensed and both the nav bar and context drawer are open on the same layer
        self.is_condensed
            && self.nav_bar.toggled_condensed
            && self.window.show_context
            && !self.window.context_is_overlay
    }

    #[inline]
    pub(crate) fn context_width(&self, has_nav: bool) -> f32 {
        let window_width = self.window.width / self.scale_factor;

        // Content width (360px) + padding (8px)
        let mut reserved_width = 360.0 + 8.0;
        if has_nav {
            // Navbar width (280px) + padding (8px)
            reserved_width += 280.0 + 8.0;
        }

        #[allow(clippy::manual_clamp)]
        // This logic is to ensure the context drawer does not take up too much of the content's space
        // The minimum width is 344px and the maximum with is 480px
        // We want to keep the content at least 360px until going down to the minimum width
        (window_width - reserved_width).min(480.0).max(344.0)
    }

    #[cold]
    pub fn set_show_context(&mut self, show: bool) {
        self.window.show_context = show;
        self.is_condensed_update();
        // Ensure nav bar is closed if condensed view and context drawer is opened
        if self.condensed_conflict() {
            self.nav_bar.toggled_condensed = false;
            self.is_condensed_update();
        }
    }

    #[inline]
    pub fn main_window_is(&self, id: iced::window::Id) -> bool {
        self.main_window_id().is_some_and(|main_id| main_id == id)
    }

    /// Whether the nav panel is visible or not
    #[must_use]
    #[inline]
    pub const fn nav_bar_active(&self) -> bool {
        self.nav_bar.active
    }

    #[inline]
    pub fn nav_bar_toggle(&mut self) {
        self.nav_bar.toggled = !self.nav_bar.toggled;
        self.nav_bar_set_toggled_condensed(self.nav_bar.toggled);
    }

    #[inline]
    pub fn nav_bar_toggle_condensed(&mut self) {
        self.nav_bar_set_toggled_condensed(!self.nav_bar.toggled_condensed);
    }

    #[inline]
    pub(crate) const fn nav_bar_context(&self) -> nav_bar::Id {
        self.nav_bar.context_id
    }

    #[inline]
    pub(crate) fn nav_bar_set_context(&mut self, id: nav_bar::Id) {
        self.nav_bar.context_id = id;
    }

    #[inline]
    pub fn nav_bar_set_toggled(&mut self, toggled: bool) {
        self.nav_bar.toggled = toggled;
        self.nav_bar_set_toggled_condensed(self.nav_bar.toggled);
    }

    #[cold]
    pub(crate) fn nav_bar_set_toggled_condensed(&mut self, toggled: bool) {
        self.nav_bar.toggled_condensed = toggled;
        self.nav_bar_update();
        // Ensure context drawer is closed if condensed view and nav bar is opened
        if self.condensed_conflict() {
            self.window.show_context = false;
            self.is_condensed_update();
            // Sync nav bar state if the view is no longer condensed after closing the context drawer
            if !self.is_condensed {
                self.nav_bar.toggled = toggled;
                self.nav_bar_update();
            }
        }
    }

    #[inline]
    pub(crate) fn nav_bar_update(&mut self) {
        self.nav_bar.active = if self.is_condensed {
            self.nav_bar.toggled_condensed
        } else {
            self.nav_bar.toggled
        };
    }

    #[inline]
    /// Set the height of the main window.
    pub(crate) const fn set_window_height(&mut self, new_height: f32) {
        self.window.height = new_height;
    }

    #[inline]
    /// Set the width of the main window.
    pub(crate) fn set_window_width(&mut self, new_width: f32) {
        self.window.width = new_width;
        self.is_condensed_update();
    }

    #[inline]
    /// Get the current system theme
    pub const fn system_theme(&self) -> &Theme {
        &self.system_theme
    }

    #[inline]
    #[must_use]
    /// Get the current system theme mode
    pub const fn system_theme_mode(&self) -> ThemeMode {
        self.system_theme_mode
    }

    pub fn watch_config<
        T: CosmicConfigEntry + Send + Sync + Default + 'static + Clone + PartialEq,
    >(
        &self,
        config_id: &'static str,
    ) -> iced::Subscription<cosmic_config::Update<T>> {
        #[cfg(all(feature = "dbus-config", target_os = "linux"))]
        if let Some(settings_daemon) = self.settings_daemon.as_ref() {
            return cosmic_config::dbus::watcher_subscription(
                settings_daemon.clone(),
                config_id,
                false,
            );
        }
        cosmic_config::config_subscription(
            std::any::TypeId::of::<T>(),
            std::borrow::Cow::Borrowed(config_id),
            T::VERSION,
        )
    }

    pub fn watch_state<
        T: CosmicConfigEntry + Send + Sync + Default + 'static + Clone + PartialEq,
    >(
        &self,
        state_id: &'static str,
    ) -> iced::Subscription<cosmic_config::Update<T>> {
        #[cfg(all(feature = "dbus-config", target_os = "linux"))]
        if let Some(settings_daemon) = self.settings_daemon.as_ref() {
            return cosmic_config::dbus::watcher_subscription(
                settings_daemon.clone(),
                state_id,
                true,
            );
        }
        cosmic_config::config_subscription(
            std::any::TypeId::of::<T>(),
            std::borrow::Cow::Borrowed(state_id),
            T::VERSION,
        )
    }

    /// Get the current focused window if it exists
    #[must_use]
    #[inline]
    pub fn focused_window(&self) -> Option<window::Id> {
        self.focused_window.last().copied()
    }

    /// Get the current focus chain of windows
    #[must_use]
    #[inline]
    pub fn focus_chain(&self) -> &[window::Id] {
        &self.focused_window
    }

    /// Whether the application should use a dark theme, according to the system
    #[must_use]
    #[inline]
    pub fn system_is_dark(&self) -> bool {
        self.portal_is_dark
            .unwrap_or(self.system_theme_mode.is_dark)
    }

    /// The [`Id`] of the main window
    #[must_use]
    #[inline]
    pub fn main_window_id(&self) -> Option<window::Id> {
        self.main_window.filter(|id| iced::window::Id::NONE != *id)
    }

    /// Reset the tracked main window to a new value
    #[inline]
    pub fn set_main_window_id(&mut self, mut id: Option<window::Id>) -> Option<window::Id> {
        std::mem::swap(&mut self.main_window, &mut id);
        id
    }

    #[cfg(feature = "winit")]
    pub fn drag<M: Send + 'static>(&self, id: Option<window::Id>) -> crate::app::Task<M> {
        let Some(id) = id.or(self.main_window) else {
            return iced::Task::none();
        };
        crate::command::drag(id)
    }

    #[cfg(feature = "winit")]
    pub fn maximize<M: Send + 'static>(
        &self,
        id: Option<window::Id>,
        maximized: bool,
    ) -> crate::app::Task<M> {
        let Some(id) = id.or(self.main_window) else {
            return iced::Task::none();
        };
        crate::command::maximize(id, maximized)
    }

    #[cfg(feature = "winit")]
    pub fn minimize<M: Send + 'static>(&self, id: Option<window::Id>) -> crate::app::Task<M> {
        let Some(id) = id.or(self.main_window) else {
            return iced::Task::none();
        };
        crate::command::minimize(id)
    }

    #[cfg(feature = "winit")]
    pub fn set_title<M: Send + 'static>(
        &self,
        id: Option<window::Id>,
        title: String,
    ) -> crate::app::Task<M> {
        let Some(id) = id.or(self.main_window) else {
            return iced::Task::none();
        };
        crate::command::set_title(id, title)
    }

    #[cfg(feature = "winit")]
    pub fn set_windowed<M: Send + 'static>(&self, id: Option<window::Id>) -> crate::app::Task<M> {
        let Some(id) = id.or(self.main_window) else {
            return iced::Task::none();
        };
        crate::command::set_windowed(id)
    }

    #[cfg(feature = "winit")]
    pub fn toggle_maximize<M: Send + 'static>(
        &self,
        id: Option<window::Id>,
    ) -> crate::app::Task<M> {
        let Some(id) = id.or(self.main_window) else {
            return iced::Task::none();
        };

        crate::command::toggle_maximize(id)
    }

    // TODO should we emit tasks setting the corner radius or unsetting it if this is changed?
    #[cfg(feature = "wayland")]
    pub fn set_sync_window_border_radii_to_theme(&mut self, sync: bool) {
        self.sync_window_border_radii_to_theme = sync;
    }

    #[cfg(feature = "wayland")]
    pub fn sync_window_border_radii_to_theme(&self) -> bool {
        self.sync_window_border_radii_to_theme
    }
}
