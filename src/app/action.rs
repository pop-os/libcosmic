// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::surface;
use crate::theme::Theme;
use crate::widget::nav_bar;
use crate::{config::CosmicTk, keyboard_nav};
#[cfg(feature = "wayland")]
use cctk::sctk::reexports::csd_frame::{WindowManagerCapabilities, WindowState};
use cosmic_theme::ThemeMode;
#[cfg(not(any(feature = "multi-window", feature = "wayland")))]
use iced::Application as IcedApplication;

/// A message managed internally by COSMIC.
#[derive(Clone, Debug)]
pub enum Action {
    /// Activate the application
    Activate(String),
    /// Application requests theme change.
    AppThemeChange(Theme),
    /// Requests to close the window.
    Close,
    /// Closes or shows the context drawer.
    ContextDrawer(bool),
    #[cfg(feature = "single-instance")]
    DbusConnection(zbus::Connection),
    /// Requests to drag the window.
    Drag,
    /// Window focus changed
    Focus(iced::window::Id),
    /// Keyboard shortcuts managed by libcosmic.
    KeyboardNav(keyboard_nav::Action),
    /// Requests to maximize the window.
    Maximize,
    /// Requests to minimize the window.
    Minimize,
    /// Activates a navigation element from the nav bar.
    NavBar(nav_bar::Id),
    /// Activates a context menu for an item from the nav bar.
    NavBarContext(nav_bar::Id),
    /// A new window was opened.
    Opened(iced::window::Id),
    /// Set scaling factor
    ScaleFactor(f32),
    /// Show the window menu
    ShowWindowMenu,
    /// Tracks updates to window suggested size.
    #[cfg(feature = "applet")]
    SuggestedBounds(Option<iced::Size>),
    /// Internal surface message
    Surface(surface::Action),
    /// Notifies that a surface was closed.
    /// Any data relating to the surface should be cleaned up.
    SurfaceClosed(iced::window::Id),
    /// Notification of system theme changes.
    SystemThemeChange(Vec<&'static str>, Theme),
    /// Notification of system theme mode changes.
    SystemThemeModeChange(Vec<&'static str>, ThemeMode),
    /// Toggles visibility of the nav bar.
    ToggleNavBar,
    /// Toggles the condensed status of the nav bar.
    ToggleNavBarCondensed,
    /// Toolkit configuration update
    ToolkitConfig(CosmicTk),
    /// Window focus lost
    Unfocus(iced::window::Id),
    /// Windowing system initialized
    WindowingSystemInitialized,
    /// Updates the window maximized state
    WindowMaximized(iced::window::Id, bool),
    /// Updates the tracked window geometry.
    WindowResize(iced::window::Id, f32, f32),
    /// Tracks updates to window state.
    #[cfg(feature = "wayland")]
    WindowState(iced::window::Id, WindowState),
    /// Capabilities the window manager supports
    #[cfg(feature = "wayland")]
    WmCapabilities(iced::window::Id, WindowManagerCapabilities),
    #[cfg(feature = "xdg-portal")]
    DesktopSettings(crate::theme::portal::Desktop),
}
