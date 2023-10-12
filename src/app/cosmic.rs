// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{command, Application, ApplicationExt, Core, Subscription};
use crate::theme::{self, Theme, ThemeType, THEME};
use crate::widget::nav_bar;
use crate::{keyboard_nav, Element};
#[cfg(feature = "wayland")]
use iced::event::wayland::{self, WindowEvent};
#[cfg(feature = "wayland")]
use iced::event::PlatformSpecific;
use iced::window;
#[cfg(not(feature = "wayland"))]
use iced_runtime::command::Action;
#[cfg(not(feature = "wayland"))]
use iced_runtime::window::Action as WindowAction;
#[cfg(feature = "wayland")]
use sctk::reexports::csd_frame::{WindowManagerCapabilities, WindowState};

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
    /// Updates the tracked window geometry.
    WindowResize(window::Id, u32, u32),
    /// Tracks updates to window state.
    #[cfg(feature = "wayland")]
    WindowState(window::Id, WindowState),
    /// Capabilities the window manager supports
    #[cfg(feature = "wayland")]
    WmCapabilities(window::Id, WindowManagerCapabilities),
}

#[derive(Default)]
pub(crate) struct Cosmic<App> {
    pub(crate) app: App,
    #[cfg(feature = "wayland")]
    pub(crate) should_exit: bool,
}

impl<T: Application> iced::Application for Cosmic<T>
where
    T::Message: Send + 'static,
{
    type Executor = T::Executor;
    type Flags = (Core, T::Flags);
    type Message = super::Message<T::Message>;
    type Theme = Theme;

    fn new((core, flags): Self::Flags) -> (Self, iced::Command<Self::Message>) {
        let (model, command) = T::init(core, flags);

        (Cosmic::new(model), command)
    }

    #[cfg(feature = "wayland")]
    fn close_requested(&self, id: window::Id) -> Self::Message {
        self.app
            .on_close_requested(id)
            .map_or(super::Message::None, super::Message::App)
    }

    fn title(&self) -> String {
        self.app.title().to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            super::Message::App(message) => self.app.update(message),
            super::Message::Cosmic(message) => self.cosmic_update(message),
            super::Message::None => iced::Command::none(),
        }
    }

    fn scale_factor(&self) -> f64 {
        f64::from(self.app.core().scale_factor())
    }

    #[cfg(feature = "wayland")]
    fn should_exit(&self) -> bool {
        self.should_exit
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
        let window_events = iced::subscription::events_with(|event, _| {
            match event {
                iced::Event::Window(id, window::Event::Resized { width, height }) => {
                    return Some(Message::WindowResize(id, width, height));
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
            theme::subscription(0)
                .map(Message::SystemThemeChange)
                .map(super::Message::Cosmic),
            window_events.map(super::Message::Cosmic),
        ])
    }

    fn theme(&self) -> Self::Theme {
        crate::theme::active()
    }

    #[cfg(feature = "wayland")]
    fn view(&self, id: window::Id) -> Element<Self::Message> {
        if id != window::Id(0) {
            return self.app.view_window(id).map(super::Message::App);
        }

        if self.app.core().window.use_template {
            self.app.view_main()
        } else {
            self.app.view().map(super::Message::App)
        }
    }

    #[cfg(not(feature = "wayland"))]
    fn view(&self) -> Element<Self::Message> {
        self.app.view_main()
    }
}

impl<T: Application> Cosmic<T> {
    #[cfg(feature = "wayland")]
    pub fn close(&mut self) -> iced::Command<super::Message<T::Message>> {
        self.should_exit = true;
        iced::Command::none()
    }

    #[cfg(not(feature = "wayland"))]
    #[allow(clippy::unused_self)]
    pub fn close(&mut self) -> iced::Command<super::Message<T::Message>> {
        iced::Command::single(Action::Window(WindowAction::Close))
    }

    fn cosmic_update(&mut self, message: Message) -> iced::Command<super::Message<T::Message>> {
        match message {
            Message::WindowResize(id, width, height) => {
                if window::Id(0) == id {
                    self.app.core_mut().set_window_width(width);
                    self.app.core_mut().set_window_height(height);
                }

                self.app.on_window_resize(id, width, height);
            }

            #[cfg(feature = "wayland")]
            Message::WindowState(id, state) => {
                if window::Id(0) == id {
                    self.app.core_mut().window.sharp_corners =
                        matches!(state, WindowState::ACTIVATED)
                            || state.contains(WindowState::TILED);
                }
            }

            #[cfg(feature = "wayland")]
            Message::WmCapabilities(id, capabilities) => {
                if window::Id(0) == id {
                    self.app.core_mut().window.can_fullscreen =
                        capabilities.contains(WindowManagerCapabilities::FULLSCREEN);
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

                keyboard_nav::Message::Fullscreen => return command::toggle_fullscreen(),
            },

            Message::ContextDrawer(show) => {
                self.app.core_mut().window.show_context = show;
            }

            Message::Drag => return command::drag(),

            Message::Minimize => return command::minimize(),

            Message::Maximize => {
                if self.app.core().window.sharp_corners {
                    self.app.core_mut().window.sharp_corners = false;
                    return command::set_windowed();
                }

                self.app.core_mut().window.sharp_corners = true;
                return command::fullscreen();
            }

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

                    // Anly apply update if the theme is set to load a system theme
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
        }

        iced::Command::none()
    }
}

impl<App: Application> Cosmic<App> {
    pub fn new(app: App) -> Self {
        Self {
            app,
            #[cfg(feature = "wayland")]
            should_exit: false,
        }
    }
}
