// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::borrow::Borrow;
use std::sync::Arc;

use super::{Application, ApplicationExt, Core, Subscription};
use crate::config::CosmicTk;
use crate::theme::{Theme, ThemeType, THEME};
use crate::widget::nav_bar;
use crate::{keyboard_nav, Element};
#[cfg(feature = "wayland")]
use cctk::sctk::reexports::csd_frame::{WindowManagerCapabilities, WindowState};
use cosmic_theme::ThemeMode;
#[cfg(feature = "wayland")]
use iced::event::wayland;
#[cfg(not(any(feature = "multi-window", feature = "wayland")))]
use iced::Application as IcedApplication;
use iced::{window, Task};
use iced_futures::event::listen_with;
use palette::color_difference::EuclideanDistance;

/// A message managed internally by COSMIC.
#[derive(Clone, Debug)]
pub enum Message {
    /// Application requests theme change.
    AppThemeChange(Theme),
    /// Requests to close the window.
    Close,
    /// Closes or shows the context drawer.
    ContextDrawer(bool),
    /// Requests to drag the window.
    Drag,
    /// Keyboard shortcuts managed by libcosmic.
    KeyboardNav(keyboard_nav::Message),
    /// Requests to maximize the window.
    Maximize,
    /// Requests to minimize the window.
    Minimize,
    /// Activates a navigation element from the nav bar.
    NavBar(nav_bar::Id),
    /// Activates a context menu for an item from the nav bar.
    NavBarContext(nav_bar::Id),
    /// Set scaling factor
    ScaleFactor(f32),
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
    /// Updates the window maximized state
    WindowMaximized(window::Id, bool),
    /// Updates the tracked window geometry.
    WindowResize(window::Id, f32, f32),
    /// Tracks updates to window state.
    #[cfg(feature = "wayland")]
    WindowState(window::Id, WindowState),
    /// Capabilities the window manager supports
    #[cfg(feature = "wayland")]
    WmCapabilities(window::Id, WindowManagerCapabilities),
    /// Notifies that a surface was closed.
    /// Any data relating to the surface should be cleaned up.
    SurfaceClosed(window::Id),
    /// Activate the application
    Activate(String),
    ShowWindowMenu,
    #[cfg(feature = "xdg-portal")]
    DesktopSettings(crate::theme::portal::Desktop),
    /// Window focus changed
    Focus(window::Id),
    /// Window focus lost
    Unfocus(window::Id),
    /// Tracks updates to window suggested size.
    #[cfg(feature = "applet")]
    SuggestedBounds(Option<iced::Size>),
    /// Window Created
    WindowCreated(window::Id),
}

#[derive(Default)]
pub struct Cosmic<App> {
    pub app: App,
}

impl<T: Application> Cosmic<T>
where
    T::Message: Send + 'static,
{
    pub fn init(
        (mut core, flags, window_settings): (Core, T::Flags, iced::window::Settings),
    ) -> (Self, iced::Task<super::Message<T::Message>>) {
        #[cfg(feature = "dbus-config")]
        {
            use iced_futures::futures::executor::block_on;
            core.settings_daemon = block_on(cosmic_config::dbus::settings_daemon_proxy()).ok();
        }

        let (model, mut command) = T::init(core, flags);

        (Self::new(model), command)
    }

    #[cfg(not(feature = "multi-window"))]
    pub fn title(&self) -> String {
        self.app.title().to_string()
    }

    #[cfg(feature = "multi-window")]
    pub fn title(&self, id: window::Id) -> String {
        self.app.title(id).to_string()
    }

    pub fn update(
        &mut self,
        message: super::Message<T::Message>,
    ) -> iced::Task<super::Message<T::Message>> {
        match message {
            super::Message::App(message) => self.app.update(message),
            super::Message::Cosmic(message) => self.cosmic_update(message),
            super::Message::None => iced::Task::none(),
            #[cfg(feature = "single-instance")]
            super::Message::DbusActivation(message) => self.app.dbus_activation(message),
        }
    }

    #[cfg(not(feature = "multi-window"))]
    pub fn scale_factor(&self) -> f64 {
        f64::from(self.app.core().scale_factor())
    }

    #[cfg(feature = "multi-window")]
    pub fn scale_factor(&self, _id: window::Id) -> f64 {
        f64::from(self.app.core().scale_factor())
    }

    pub fn style(&self, theme: &Theme) -> iced_runtime::Appearance {
        if let Some(style) = self.app.style() {
            style
        } else if self.app.core().window.sharp_corners {
            let theme = THEME.lock().unwrap();
            crate::style::iced::application::appearance(theme.borrow())
        } else {
            let theme = THEME.lock().unwrap();
            iced_runtime::Appearance {
                background_color: iced_core::Color::TRANSPARENT,
                icon_color: theme.cosmic().on_bg_color().into(),
                text_color: theme.cosmic().on_bg_color().into(),
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn subscription(&self) -> Subscription<super::Message<T::Message>> {
        let window_events = listen_with(|event, _, id| {
            match event {
                iced::Event::Window(window::Event::Resized(iced::Size { width, height })) => {
                    return Some(Message::WindowResize(id, width, height));
                }
                iced::Event::Window(window::Event::Closed) => {
                    return Some(Message::SurfaceClosed(id))
                }
                iced::Event::Window(window::Event::Focused) => return Some(Message::Focus(id)),
                iced::Event::Window(window::Event::Unfocused) => return Some(Message::Unfocus(id)),
                iced::Event::Window(window::Event::Opened { .. }) => {
                    return Some(Message::WindowCreated(id));
                }
                #[cfg(feature = "wayland")]
                iced::Event::PlatformSpecific(iced::event::PlatformSpecific::Wayland(event)) => {
                    match event {
                        wayland::Event::Popup(wayland::PopupEvent::Done, _, id)
                        | wayland::Event::Layer(wayland::LayerEvent::Done, _, id) => {
                            return Some(Message::SurfaceClosed(id));
                        }
                        #[cfg(feature = "applet")]
                        wayland::Event::Window(
                            iced::event::wayland::WindowEvent::SuggestedBounds(b),
                        ) => {
                            return Some(Message::SuggestedBounds(b));
                        }
                        _ => (),
                    }
                }
                _ => (),
            }

            None
        });

        let mut subscriptions = vec![
            self.app.subscription().map(super::Message::App),
            self.app
                .core()
                .watch_config::<crate::config::CosmicTk>(crate::config::ID)
                .map(|update| {
                    for why in update.errors {
                        tracing::error!(?why, "cosmic toolkit config update error");
                    }

                    super::Message::Cosmic(Message::ToolkitConfig(update.config))
                }),
            self.app
                .core()
                .watch_config::<cosmic_theme::Theme>(
                    if if let ThemeType::System { prefer_dark, .. } =
                        THEME.lock().unwrap().theme_type
                    {
                        prefer_dark
                    } else {
                        None
                    }
                    .unwrap_or_else(|| self.app.core().system_theme_mode.is_dark)
                    {
                        cosmic_theme::DARK_THEME_ID
                    } else {
                        cosmic_theme::LIGHT_THEME_ID
                    },
                )
                .map(|update| {
                    for why in update.errors {
                        tracing::error!(?why, "cosmic theme config update error");
                    }
                    Message::SystemThemeChange(
                        update.keys,
                        crate::theme::Theme::system(Arc::new(update.config)),
                    )
                })
                .map(super::Message::Cosmic),
            self.app
                .core()
                .watch_config::<ThemeMode>(cosmic_theme::THEME_MODE_ID)
                .map(|update| {
                    for e in update.errors {
                        tracing::error!("{e}");
                    }
                    Message::SystemThemeModeChange(update.keys, update.config)
                })
                .map(super::Message::Cosmic),
            window_events.map(super::Message::Cosmic),
            #[cfg(feature = "xdg-portal")]
            crate::theme::portal::desktop_settings()
                .map(Message::DesktopSettings)
                .map(super::Message::Cosmic),
        ];

        if self.app.core().keyboard_nav {
            subscriptions.push(
                keyboard_nav::subscription()
                    .map(Message::KeyboardNav)
                    .map(super::Message::Cosmic),
            );
        }

        #[cfg(feature = "single-instance")]
        if self.app.core().single_instance {
            subscriptions.push(super::single_instance_subscription::<T>());
        }

        Subscription::batch(subscriptions)
    }

    #[cfg(not(feature = "multi-window"))]
    pub fn theme(&self) -> Theme {
        crate::theme::active()
    }

    #[cfg(feature = "multi-window")]
    pub fn theme(&self, _id: window::Id) -> Theme {
        crate::theme::active()
    }

    #[cfg(feature = "multi-window")]
    pub fn view(&self, id: window::Id) -> Element<super::Message<T::Message>> {
        if !self
            .app
            .core()
            .main_window_id()
            .is_some_and(|main_id| main_id == id)
        {
            return self.app.view_window(id).map(super::Message::App);
        }

        if self.app.core().window.use_template {
            self.app.view_main()
        } else {
            self.app.view().map(super::Message::App)
        }
    }

    #[cfg(not(feature = "multi-window"))]
    pub fn view(&self) -> Element<super::Message<T::Message>> {
        self.app.view_main()
    }
}

impl<T: Application> Cosmic<T> {
    #[allow(clippy::unused_self)]
    pub fn close(&mut self) -> iced::Task<super::Message<T::Message>> {
        if let Some(id) = self.app.core().main_window_id() {
            iced::window::close(id)
        } else {
            iced::Task::none()
        }
    }

    #[allow(clippy::too_many_lines)]
    fn cosmic_update(&mut self, message: Message) -> iced::Task<super::Message<T::Message>> {
        match message {
            Message::WindowMaximized(id, maximized) => {
                if self
                    .app
                    .core()
                    .main_window_id()
                    .is_some_and(|main_id| main_id == id)
                {
                    self.app.core_mut().window.sharp_corners = maximized;
                }
            }

            Message::WindowResize(id, width, height) => {
                if self
                    .app
                    .core()
                    .main_window_id()
                    .is_some_and(|main_id| main_id == id)
                {
                    self.app.core_mut().set_window_width(width);
                    self.app.core_mut().set_window_height(height);
                }

                self.app.on_window_resize(id, width, height);

                //TODO: more efficient test of maximized (winit has no event for maximize if set by the OS)
                return iced::window::get_maximized(id).map(move |maximized| {
                    super::Message::Cosmic(Message::WindowMaximized(id, maximized))
                });
            }

            #[cfg(feature = "wayland")]
            Message::WindowState(id, state) => {
                if self
                    .app
                    .core()
                    .main_window_id()
                    .is_some_and(|main_id| main_id == id)
                {
                    self.app.core_mut().window.sharp_corners = state.intersects(
                        WindowState::MAXIMIZED
                            | WindowState::FULLSCREEN
                            | WindowState::TILED
                            | WindowState::TILED_RIGHT
                            | WindowState::TILED_LEFT
                            | WindowState::TILED_TOP
                            | WindowState::TILED_BOTTOM,
                    );
                }
            }

            #[cfg(feature = "wayland")]
            Message::WmCapabilities(id, capabilities) => {
                if self
                    .app
                    .core()
                    .main_window_id()
                    .is_some_and(|main_id| main_id == id)
                {
                    self.app.core_mut().window.show_maximize =
                        capabilities.contains(WindowManagerCapabilities::MAXIMIZE);
                    self.app.core_mut().window.show_minimize =
                        capabilities.contains(WindowManagerCapabilities::MINIMIZE);
                    self.app.core_mut().window.show_window_menu =
                        capabilities.contains(WindowManagerCapabilities::WINDOW_MENU);
                }
            }

            Message::KeyboardNav(message) => match message {
                keyboard_nav::Message::FocusNext => {
                    return iced::widget::focus_next().map(super::Message::Cosmic)
                }
                keyboard_nav::Message::FocusPrevious => {
                    return iced::widget::focus_previous().map(super::Message::Cosmic)
                }
                keyboard_nav::Message::Escape => return self.app.on_escape(),
                keyboard_nav::Message::Search => return self.app.on_search(),

                keyboard_nav::Message::Fullscreen => return self.app.core().toggle_maximize(None),
            },

            Message::ContextDrawer(show) => {
                self.app.core_mut().set_show_context(show);
                return self.app.on_context_drawer();
            }

            Message::Drag => return self.app.core().drag(None),

            Message::Minimize => return self.app.core().minimize(None),

            Message::Maximize => return self.app.core().toggle_maximize(None),

            Message::NavBar(key) => {
                self.app.core_mut().nav_bar_set_toggled_condensed(false);
                return self.app.on_nav_select(key);
            }

            Message::NavBarContext(key) => {
                self.app.core_mut().nav_bar_set_context(key);
                return self.app.on_nav_context(key);
            }

            Message::ToggleNavBar => {
                self.app.core_mut().nav_bar_toggle();
            }

            Message::ToggleNavBarCondensed => {
                self.app.core_mut().nav_bar_toggle_condensed();
            }

            Message::AppThemeChange(mut theme) => {
                if let ThemeType::System { theme: _, .. } = theme.theme_type {
                    self.app.core_mut().theme_sub_counter += 1;

                    let portal_accent = self.app.core().portal_accent;
                    if let Some(a) = portal_accent {
                        let t_inner = theme.cosmic();
                        if a.distance_squared(*t_inner.accent_color()) > 0.00001 {
                            theme = Theme::system(Arc::new(t_inner.with_accent(a)));
                        }
                    };
                }

                THEME.lock().unwrap().set_theme(theme.theme_type);
            }

            Message::SystemThemeChange(keys, theme) => {
                let cur_is_dark = THEME.lock().unwrap().theme_type.is_dark();
                // Ignore updates if the current theme mode does not match.
                if cur_is_dark != theme.cosmic().is_dark {
                    return iced::Task::none();
                }
                let cmd = self.app.system_theme_update(&keys, theme.cosmic());
                // Record the last-known system theme in event that the current theme is custom.
                self.app.core_mut().system_theme = theme.clone();
                let portal_accent = self.app.core().portal_accent;
                {
                    let mut cosmic_theme = THEME.lock().unwrap();

                    // Only apply update if the theme is set to load a system theme
                    if let ThemeType::System {
                        theme: _,
                        prefer_dark,
                    } = cosmic_theme.theme_type
                    {
                        let mut new_theme = if let Some(a) = portal_accent {
                            let t_inner = theme.cosmic();
                            if a.distance_squared(*t_inner.accent_color()) > 0.00001 {
                                Theme::system(Arc::new(t_inner.with_accent(a)))
                            } else {
                                theme
                            }
                        } else {
                            theme
                        };
                        new_theme.theme_type.prefer_dark(prefer_dark);

                        cosmic_theme.set_theme(new_theme.theme_type);
                    }
                }

                return cmd;
            }

            Message::ScaleFactor(factor) => {
                self.app.core_mut().set_scale_factor(factor);
            }

            Message::Close => {
                return match self.app.on_app_exit() {
                    Some(message) => self.app.update(message),
                    None => self.close(),
                };
            }
            Message::SystemThemeModeChange(keys, mode) => {
                if !keys.contains(&"is_dark") {
                    return iced::Task::none();
                }
                if match THEME.lock().unwrap().theme_type {
                    ThemeType::System {
                        theme: _,
                        prefer_dark,
                    } => prefer_dark.is_some(),
                    _ => false,
                } {
                    return iced::Task::none();
                }
                let mut cmds = vec![self.app.system_theme_mode_update(&keys, &mode)];

                let core = self.app.core_mut();
                let prev_is_dark = core.system_is_dark();
                core.system_theme_mode = mode;
                let is_dark = core.system_is_dark();
                let changed = prev_is_dark != is_dark;
                if changed {
                    core.theme_sub_counter += 1;
                    let mut new_theme = if is_dark {
                        crate::theme::system_dark()
                    } else {
                        crate::theme::system_light()
                    };
                    cmds.push(self.app.system_theme_update(&[], new_theme.cosmic()));

                    let core = self.app.core_mut();
                    new_theme = if let Some(a) = core.portal_accent {
                        let t_inner = new_theme.cosmic();
                        if a.distance_squared(*t_inner.accent_color()) > 0.00001 {
                            Theme::system(Arc::new(t_inner.with_accent(a)))
                        } else {
                            new_theme
                        }
                    } else {
                        new_theme
                    };

                    core.system_theme = new_theme.clone();
                    {
                        let mut cosmic_theme = THEME.lock().unwrap();
                        // Only apply update if the theme is set to load a system theme
                        if let ThemeType::System { theme: _, .. } = cosmic_theme.theme_type {
                            cosmic_theme.set_theme(new_theme.theme_type);
                        }
                    }
                }
                return Task::batch(cmds);
            }
            Message::Activate(_token) =>
            {
                #[cfg(feature = "wayland")]
                if let Some(id) = self.app.core().main_window_id() {
                    return iced_winit::platform_specific::commands::activation::activate(
                        id,
                        #[allow(clippy::used_underscore_binding)]
                        _token,
                    );
                }
            }
            Message::SurfaceClosed(id) => {
                let mut ret = if let Some(msg) = self.app.on_close_requested(id) {
                    self.app.update(msg)
                } else {
                    Task::none()
                };
                let core = self.app.core();
                if core.exit_on_main_window_closed
                    && core.main_window_id().is_some_and(|m_id| id == m_id)
                {
                    ret = Task::batch(vec![iced::exit::<super::Message<T::Message>>()]);
                }
                return ret;
            }
            Message::ShowWindowMenu => {
                if let Some(id) = self.app.core().main_window_id() {
                    return iced::window::show_system_menu(id);
                }
            }
            #[cfg(feature = "xdg-portal")]
            Message::DesktopSettings(crate::theme::portal::Desktop::ColorScheme(s)) => {
                use ashpd::desktop::settings::ColorScheme;
                if match THEME.lock().unwrap().theme_type {
                    ThemeType::System {
                        theme: _,
                        prefer_dark,
                    } => prefer_dark.is_some(),
                    _ => false,
                } {
                    return iced::Task::none();
                }
                let is_dark = match s {
                    ColorScheme::NoPreference => None,
                    ColorScheme::PreferDark => Some(true),
                    ColorScheme::PreferLight => Some(false),
                };
                let core = self.app.core_mut();
                let prev_is_dark = core.system_is_dark();
                core.portal_is_dark = is_dark;
                let is_dark = core.system_is_dark();
                let changed = prev_is_dark != is_dark;
                if changed {
                    core.theme_sub_counter += 1;
                    let new_theme = if is_dark {
                        crate::theme::system_dark()
                    } else {
                        crate::theme::system_light()
                    };
                    core.system_theme = new_theme.clone();
                    {
                        let mut cosmic_theme = THEME.lock().unwrap();

                        // Only apply update if the theme is set to load a system theme
                        if let ThemeType::System { theme: _, .. } = cosmic_theme.theme_type {
                            cosmic_theme.set_theme(new_theme.theme_type);
                        }
                    }
                }
            }
            #[cfg(feature = "xdg-portal")]
            Message::DesktopSettings(crate::theme::portal::Desktop::Accent(c)) => {
                use palette::Srgba;
                let c = Srgba::new(c.red() as f32, c.green() as f32, c.blue() as f32, 1.0);
                let core = self.app.core_mut();
                core.portal_accent = Some(c);
                let cur_accent = core.system_theme.cosmic().accent_color();

                if cur_accent.distance_squared(*c) < 0.00001 {
                    // skip calculations if we already have the same color
                    return iced::Task::none();
                }

                {
                    let mut cosmic_theme = THEME.lock().unwrap();

                    // Only apply update if the theme is set to load a system theme
                    if let ThemeType::System {
                        theme: t,
                        prefer_dark,
                    } = cosmic_theme.theme_type.clone()
                    {
                        cosmic_theme.set_theme(ThemeType::System {
                            theme: Arc::new(t.with_accent(c)),
                            prefer_dark,
                        });
                    }
                }
            }
            #[cfg(feature = "xdg-portal")]
            Message::DesktopSettings(crate::theme::portal::Desktop::Contrast(_)) => {
                // TODO when high contrast is integrated in settings and all custom themes
            }

            Message::ToolkitConfig(config) => {
                // Change the icon theme if not defined by the application.
                if !self.app.core().icon_theme_override
                    && crate::icon_theme::default() != config.icon_theme
                {
                    crate::icon_theme::set_default(config.icon_theme.clone());
                }

                *crate::config::COSMIC_TK.write().unwrap() = config;
            }

            Message::Focus(f) => {
                self.app.core_mut().focused_window = Some(f);
            }

            Message::Unfocus(id) => {
                let core = self.app.core_mut();
                if core.focused_window.as_ref().is_some_and(|cur| *cur == id) {
                    core.focused_window = None;
                }
            }
            Message::WindowCreated(id) => {
                let core = self.app.core_mut();
                // Set the main window if it was not set to something else
                if core.main_window.is_none() {
                    core.main_window = Some(id);
                }
            }
            #[cfg(feature = "applet")]
            Message::SuggestedBounds(b) => {
                tracing::info!("Suggested bounds: {b:?}");
                let core = self.app.core_mut();
                core.applet.suggested_bounds = b;
            }
            _ => {}
        }

        iced::Task::none()
    }
}

impl<App: Application> Cosmic<App> {
    pub fn new(app: App) -> Self {
        Self { app }
    }
}
