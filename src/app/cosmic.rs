// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use super::{command, Application, ApplicationExt, Core, Subscription};
use crate::theme::{self, Theme, ThemeType, THEME};
use crate::widget::nav_bar;
use crate::{keyboard_nav, Element};
#[cfg(feature = "wayland")]
use cctk::sctk::reexports::csd_frame::{WindowManagerCapabilities, WindowState};
use cosmic_theme::ThemeMode;
#[cfg(feature = "wayland")]
use iced::event::wayland::{self, WindowEvent};
#[cfg(feature = "wayland")]
use iced::event::PlatformSpecific;
#[cfg(all(feature = "winit", feature = "multi-window"))]
use iced::multi_window::Application as IcedApplication;
#[cfg(feature = "wayland")]
use iced::wayland::Application as IcedApplication;
use iced::window;
#[cfg(not(any(feature = "multi-window", feature = "wayland")))]
use iced::Application as IcedApplication;
use iced_futures::event::listen_with;
#[cfg(not(feature = "wayland"))]
use iced_runtime::command::Action;
#[cfg(not(feature = "wayland"))]
use iced_runtime::window::Action as WindowAction;

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
    /// Set scaling factor
    ScaleFactor(f32),
    /// Toggles visibility of the nav bar.
    ToggleNavBar,
    /// Toggles the condensed status of the nav bar.
    ToggleNavBarCondensed,
    /// Notification of system theme changes.
    SystemThemeChange(Theme),
    /// Notification of system theme mode changes.
    SystemThemeModeChange(ThemeMode),
    /// Updates the window maximized state
    WindowMaximized(window::Id, bool),
    /// Updates the tracked window geometry.
    WindowResize(window::Id, u32, u32),
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
}

#[derive(Default)]
pub struct Cosmic<App> {
    pub app: App,
}

impl<T: Application> IcedApplication for Cosmic<T>
where
    T::Message: Send + 'static,
{
    type Executor = T::Executor;
    type Flags = (Core, T::Flags);
    type Message = super::Message<T::Message>;
    type Theme = Theme;

    fn new((mut core, flags): Self::Flags) -> (Self, iced::Command<Self::Message>) {
        #[cfg(feature = "dbus-config")]
        {
            use iced_futures::futures::executor::block_on;
            core.settings_daemon = block_on(cosmic_config::dbus::settings_daemon_proxy()).ok();
        }

        let (model, command) = T::init(core, flags);

        (Self::new(model), command)
    }

    #[cfg(not(any(feature = "multi-window", feature = "wayland")))]
    fn title(&self) -> String {
        self.app.title().to_string()
    }

    #[cfg(any(feature = "multi-window", feature = "wayland"))]
    fn title(&self, id: window::Id) -> String {
        self.app.title(id).to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            super::Message::App(message) => self.app.update(message),
            super::Message::Cosmic(message) => self.cosmic_update(message),
            super::Message::None => iced::Command::none(),
            #[cfg(feature = "single-instance")]
            super::Message::DbusActivation(message) => self.app.dbus_activation(message),
        }
    }

    #[cfg(not(any(feature = "multi-window", feature = "wayland")))]
    fn scale_factor(&self) -> f64 {
        f64::from(self.app.core().scale_factor())
    }

    #[cfg(any(feature = "multi-window", feature = "wayland"))]
    fn scale_factor(&self, _id: window::Id) -> f64 {
        f64::from(self.app.core().scale_factor())
    }

    fn style(&self) -> <Self::Theme as iced_style::application::StyleSheet>::Style {
        if let Some(style) = self.app.style() {
            style
        } else if self.app.core().window.sharp_corners {
            theme::Application::default()
        } else {
            theme::Application::Custom(Box::new(|theme| iced_style::application::Appearance {
                background_color: iced_core::Color::TRANSPARENT,
                icon_color: theme.cosmic().on_bg_color().into(),
                text_color: theme.cosmic().on_bg_color().into(),
            }))
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let window_events = listen_with(|event, _| {
            match event {
                iced::Event::Window(id, window::Event::Resized { width, height }) => {
                    return Some(Message::WindowResize(id, width, height));
                }
                iced::Event::Window(id, window::Event::Closed) => {
                    return Some(Message::SurfaceClosed(id))
                }
                #[cfg(feature = "wayland")]
                iced::Event::PlatformSpecific(PlatformSpecific::Wayland(event)) => match event {
                    wayland::Event::Window(WindowEvent::State(state), _surface, id) => {
                        return Some(Message::WindowState(id, state));
                    }
                    wayland::Event::Window(
                        WindowEvent::WmCapabilities(capabilities),
                        _surface,
                        id,
                    ) => {
                        return Some(Message::WmCapabilities(id, capabilities));
                    }
                    wayland::Event::Popup(wayland::PopupEvent::Done, _, id)
                    | wayland::Event::Layer(wayland::LayerEvent::Done, _, id) => {
                        return Some(Message::SurfaceClosed(id));
                    }
                    _ => (),
                },
                _ => (),
            }

            None
        });

        Subscription::batch(vec![
            self.app.subscription().map(super::Message::App),
            keyboard_nav::subscription()
                .map(Message::KeyboardNav)
                .map(super::Message::Cosmic),
            self.app
                .core()
                .watch_config::<cosmic_theme::Theme>(if self.app.core().system_theme_mode.is_dark {
                    cosmic_theme::DARK_THEME_ID
                } else {
                    cosmic_theme::LIGHT_THEME_ID
                })
                .map(|update| {
                    for e in update.errors {
                        tracing::error!("{e}");
                    }
                    Message::SystemThemeChange(crate::theme::Theme::system(Arc::new(update.config)))
                })
                .map(super::Message::Cosmic),
            self.app
                .core()
                .watch_config::<ThemeMode>(cosmic_theme::THEME_MODE_ID)
                .map(|update| {
                    for e in update.errors {
                        tracing::error!("{e}");
                    }
                    Message::SystemThemeModeChange(update.config)
                })
                .map(super::Message::Cosmic),
            window_events.map(super::Message::Cosmic),
            #[cfg(feature = "single-instance")]
            self.app
                .core()
                .single_instance
                .then(|| super::single_instance_subscription::<T>())
                .unwrap_or_else(Subscription::none),
        ])
    }

    #[cfg(not(any(feature = "multi-window", feature = "wayland")))]
    fn theme(&self) -> Self::Theme {
        crate::theme::active()
    }

    #[cfg(any(feature = "multi-window", feature = "wayland"))]
    fn theme(&self, _id: window::Id) -> Self::Theme {
        crate::theme::active()
    }

    #[cfg(any(feature = "multi-window", feature = "wayland"))]
    fn view(&self, id: window::Id) -> Element<Self::Message> {
        if id != self.app.main_window_id() {
            return self.app.view_window(id).map(super::Message::App);
        }

        if self.app.core().window.use_template {
            self.app.view_main()
        } else {
            self.app.view().map(super::Message::App)
        }
    }

    #[cfg(not(any(feature = "multi-window", feature = "wayland")))]
    fn view(&self) -> Element<Self::Message> {
        self.app.view_main()
    }
}

impl<T: Application> Cosmic<T> {
    #[cfg(feature = "wayland")]
    pub fn close(&mut self) -> iced::Command<super::Message<T::Message>> {
        iced_sctk::commands::window::close_window(self.app.main_window_id())
    }

    #[cfg(not(feature = "wayland"))]
    #[allow(clippy::unused_self)]
    pub fn close(&mut self) -> iced::Command<super::Message<T::Message>> {
        iced::Command::single(Action::Window(WindowAction::Close(
            self.app.main_window_id(),
        )))
    }

    #[allow(clippy::too_many_lines)]
    fn cosmic_update(&mut self, message: Message) -> iced::Command<super::Message<T::Message>> {
        match message {
            Message::WindowMaximized(id, maximized) => {
                if self.app.main_window_id() == id {
                    self.app.core_mut().window.sharp_corners = maximized;
                }
            }

            Message::WindowResize(id, width, height) => {
                if self.app.main_window_id() == id {
                    self.app.core_mut().set_window_width(width);
                    self.app.core_mut().set_window_height(height);
                }

                self.app.on_window_resize(id, width, height);

                //TODO: more efficient test of maximized (winit has no event for maximize if set by the OS)
                #[cfg(not(feature = "wayland"))]
                return iced::window::fetch_maximized(id, move |maximized| {
                    super::Message::Cosmic(Message::WindowMaximized(id, maximized))
                });
            }

            #[cfg(feature = "wayland")]
            Message::WindowState(id, state) => {
                if self.app.main_window_id() == id {
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
                if self.app.main_window_id() == id {
                    self.app.core_mut().window.show_maximize =
                        capabilities.contains(WindowManagerCapabilities::MAXIMIZE);
                    self.app.core_mut().window.show_minimize =
                        capabilities.contains(WindowManagerCapabilities::MINIMIZE);
                    self.app.core_mut().window.show_window_menu =
                        capabilities.contains(WindowManagerCapabilities::WINDOW_MENU);
                }
            }

            Message::KeyboardNav(message) => match message {
                keyboard_nav::Message::Unfocus => {
                    return keyboard_nav::unfocus().map(super::Message::Cosmic)
                }
                keyboard_nav::Message::FocusNext => {
                    return iced::widget::focus_next().map(super::Message::Cosmic)
                }
                keyboard_nav::Message::FocusPrevious => {
                    return iced::widget::focus_previous().map(super::Message::Cosmic)
                }
                keyboard_nav::Message::Escape => return self.app.on_escape(),
                keyboard_nav::Message::Search => return self.app.on_search(),

                keyboard_nav::Message::Fullscreen => {
                    return command::toggle_maximize(Some(self.app.main_window_id()))
                }
            },

            Message::ContextDrawer(show) => {
                self.app.core_mut().window.show_context = show;
                return self.app.on_context_drawer();
            }

            Message::Drag => return command::drag(Some(self.app.main_window_id())),

            Message::Minimize => return command::minimize(Some(self.app.main_window_id())),

            Message::Maximize => return command::toggle_maximize(Some(self.app.main_window_id())),

            Message::NavBar(key) => {
                self.app.core_mut().nav_bar_set_toggled_condensed(false);
                return self.app.on_nav_select(key);
            }

            Message::ToggleNavBar => {
                self.app.core_mut().nav_bar_toggle();
            }

            Message::ToggleNavBarCondensed => {
                self.app.core_mut().nav_bar_toggle_condensed();
            }

            Message::AppThemeChange(mut theme) => {
                // Apply last-known system theme if the system theme is preferred.
                if let ThemeType::System(_) = theme.theme_type {
                    self.app.core_mut().theme_sub_counter += 1;
                    theme = self.app.core().system_theme.clone();
                }

                THEME.with(move |t| {
                    let mut cosmic_theme = t.borrow_mut();
                    cosmic_theme.set_theme(theme.theme_type);
                });
            }

            Message::SystemThemeChange(theme) => {
                // Record the last-known system theme in event that the current theme is custom.
                self.app.core_mut().system_theme = theme.clone();
                THEME.with(move |t| {
                    let mut cosmic_theme = t.borrow_mut();

                    // Only apply update if the theme is set to load a system theme
                    if let ThemeType::System(_) = cosmic_theme.theme_type {
                        cosmic_theme.set_theme(theme.theme_type);
                    }
                });
            }

            Message::ScaleFactor(factor) => {
                self.app.core_mut().set_scale_factor(factor);
            }

            Message::Close => {
                self.app.on_app_exit();
                return self.close();
            }
            Message::SystemThemeModeChange(mode) => {
                let core = self.app.core_mut();
                let changed = core.system_theme_mode.is_dark != mode.is_dark;
                core.system_theme_mode = mode;
                core.theme_sub_counter += 1;
                if changed {
                    let new_theme = crate::theme::system_preference();
                    core.system_theme = new_theme.clone();
                    THEME.with(move |t| {
                        let mut cosmic_theme = t.borrow_mut();

                        // Only apply update if the theme is set to load a system theme
                        if let ThemeType::System(_) = cosmic_theme.theme_type {
                            cosmic_theme.set_theme(new_theme.theme_type);
                        }
                    });
                }
            }
            Message::Activate(_token) => {
                #[cfg(feature = "wayland")]
                return iced_sctk::commands::activation::activate(
                    self.app.main_window_id(),
                    #[allow(clippy::used_underscore_binding)]
                    _token,
                );
            }
            Message::SurfaceClosed(id) => {
                if let Some(msg) = self.app.on_close_requested(id) {
                    return self.app.update(msg);
                }
            }
            Message::ShowWindowMenu => {
                #[cfg(not(feature = "wayland"))]
                return window::show_window_menu(window::Id::MAIN);
            }
        }

        iced::Command::none()
    }
}

impl<App: Application> Cosmic<App> {
    pub fn new(app: App) -> Self {
        Self { app }
    }
}
