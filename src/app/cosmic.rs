// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use super::{Action, Application, ApplicationExt, Subscription};
use crate::theme::{THEME, Theme, ThemeType};
use crate::{Core, Element, keyboard_nav};
#[cfg(feature = "wayland")]
use cctk::sctk::reexports::csd_frame::{WindowManagerCapabilities, WindowState};
use cosmic_theme::ThemeMode;
#[cfg(not(any(feature = "multi-window", feature = "wayland")))]
use iced::Application as IcedApplication;
#[cfg(feature = "wayland")]
use iced::event::wayland;
use iced::{Task, window};
use iced_futures::event::listen_with;
#[cfg(feature = "wayland")]
use iced_winit::SurfaceIdWrapper;
use palette::color_difference::EuclideanDistance;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[non_exhaustive]
pub enum WindowingSystem {
    UiKit,
    AppKit,
    Orbital,
    OhosNdk,
    Xlib,
    Xcb,
    Wayland,
    Drm,
    Gbm,
    Win32,
    WinRt,
    Web,
    WebCanvas,
    WebOffscreenCanvas,
    AndroidNdk,
    Haiku,
}

pub(crate) static WINDOWING_SYSTEM: std::sync::OnceLock<WindowingSystem> =
    std::sync::OnceLock::new();

pub fn windowing_system() -> Option<WindowingSystem> {
    WINDOWING_SYSTEM.get().copied()
}

fn init_windowing_system<M>(handle: raw_window_handle::WindowHandle) -> crate::Action<M> {
    let raw: &raw_window_handle::RawWindowHandle = handle.as_ref();
    let system = match raw {
        window::raw_window_handle::RawWindowHandle::UiKit(_) => WindowingSystem::UiKit,
        window::raw_window_handle::RawWindowHandle::AppKit(_) => WindowingSystem::AppKit,
        window::raw_window_handle::RawWindowHandle::Orbital(_) => WindowingSystem::Orbital,
        window::raw_window_handle::RawWindowHandle::OhosNdk(_) => WindowingSystem::OhosNdk,
        window::raw_window_handle::RawWindowHandle::Xlib(_) => WindowingSystem::Xlib,
        window::raw_window_handle::RawWindowHandle::Xcb(_) => WindowingSystem::Xcb,
        window::raw_window_handle::RawWindowHandle::Wayland(_) => WindowingSystem::Wayland,
        window::raw_window_handle::RawWindowHandle::Web(_) => WindowingSystem::Web,
        window::raw_window_handle::RawWindowHandle::WebCanvas(_) => WindowingSystem::WebCanvas,
        window::raw_window_handle::RawWindowHandle::WebOffscreenCanvas(_) => {
            WindowingSystem::WebOffscreenCanvas
        }
        window::raw_window_handle::RawWindowHandle::AndroidNdk(_) => WindowingSystem::AndroidNdk,
        window::raw_window_handle::RawWindowHandle::Haiku(_) => WindowingSystem::Haiku,
        window::raw_window_handle::RawWindowHandle::Drm(_) => WindowingSystem::Drm,
        window::raw_window_handle::RawWindowHandle::Gbm(_) => WindowingSystem::Gbm,
        window::raw_window_handle::RawWindowHandle::Win32(_) => WindowingSystem::Win32,
        window::raw_window_handle::RawWindowHandle::WinRt(_) => WindowingSystem::WinRt,
        _ => {
            tracing::warn!("Unknown windowing system: {raw:?}");
            return crate::Action::Cosmic(Action::WindowingSystemInitialized);
        }
    };

    _ = WINDOWING_SYSTEM.set(system);
    crate::Action::Cosmic(Action::WindowingSystemInitialized)
}

#[derive(Default)]
pub struct Cosmic<App: Application> {
    pub app: App,
    #[cfg(feature = "wayland")]
    pub surface_views: HashMap<
        window::Id,
        (
            Option<window::Id>,
            SurfaceIdWrapper,
            Box<dyn for<'a> Fn(&'a App) -> Element<'a, crate::Action<App::Message>>>,
        ),
    >,
    pub tracked_windows: HashSet<window::Id>,
    pub opened_surfaces: HashMap<window::Id, u32>,
}

impl<T: Application> Cosmic<T>
where
    T::Message: Send + 'static,
{
    pub fn init(
        (mut core, flags): (Core, T::Flags),
    ) -> (Self, iced::Task<crate::Action<T::Message>>) {
        #[cfg(all(feature = "dbus-config", target_os = "linux"))]
        {
            use iced_futures::futures::executor::block_on;
            core.settings_daemon = block_on(cosmic_config::dbus::settings_daemon_proxy()).ok();
        }
        let id = core.main_window_id().unwrap_or(window::Id::RESERVED);

        let (model, command) = T::init(core, flags);

        (
            Self::new(model),
            Task::batch([
                command,
                iced_runtime::window::run_with_handle(id, init_windowing_system),
            ]),
        )
    }

    #[cfg(not(feature = "multi-window"))]
    pub fn title(&self) -> String {
        self.app.title().to_string()
    }

    #[cfg(feature = "multi-window")]
    pub fn title(&self, id: window::Id) -> String {
        self.app.title(id).to_string()
    }

    #[allow(clippy::too_many_lines)]
    pub fn surface_update(
        &mut self,
        _surface_message: crate::surface::Action,
    ) -> iced::Task<crate::Action<T::Message>> {
        #[cfg(feature = "surface-message")]
        match _surface_message {
            #[cfg(feature = "wayland")]
            crate::surface::Action::AppSubsurface(settings, view) => {
                let Some(settings) = std::sync::Arc::try_unwrap(settings)
                    .ok()
                    .and_then(|s| s.downcast::<Box<dyn Fn(&mut T) -> iced_runtime::platform_specific::wayland::subsurface::SctkSubsurfaceSettings + Send + Sync>>().ok()) else {
                    tracing::error!("Invalid settings for subsurface");
                    return Task::none();
                    };

                if let Some(view) = view.and_then(|view| {
                    match std::sync::Arc::try_unwrap(view).ok()?.downcast::<Box<
                        dyn for<'a> Fn(&'a T) -> Element<'a, crate::Action<T::Message>>
                            + Send
                            + Sync,
                    >>() {
                        Ok(v) => Some(v),
                        Err(err) => {
                            tracing::error!("Invalid view for subsurface view: {err:?}");

                            None
                        }
                    }
                }) {
                    let settings = settings(&mut self.app);

                    self.get_subsurface(settings, *view)
                } else {
                    iced_winit::commands::subsurface::get_subsurface(settings(&mut self.app))
                }
            }
            #[cfg(feature = "wayland")]
            crate::surface::Action::Subsurface(settings, view) => {
                let Some(settings) = std::sync::Arc::try_unwrap(settings)
                    .ok()
                    .and_then(|s| s.downcast::<Box<dyn Fn() -> iced_runtime::platform_specific::wayland::subsurface::SctkSubsurfaceSettings + Send + Sync>>().ok()) else {
                    tracing::error!("Invalid settings for subsurface");
                    return Task::none();
                };

                if let Some(view) = view.and_then(|view| {
                    match std::sync::Arc::try_unwrap(view).ok()?.downcast::<Box<
                            dyn Fn() -> Element<'static, crate::Action<T::Message>> + Send + Sync,
                        >>() {
                            Ok(v) => Some(v),
                            Err(err) => {
                                tracing::error!("Invalid view for subsurface view: {err:?}");

                                None
                            }
                        }
                }) {
                    let settings = settings();

                    self.get_subsurface(settings, Box::new(move |_| view()))
                } else {
                    iced_winit::commands::subsurface::get_subsurface(settings())
                }
            }
            #[cfg(feature = "wayland")]
            crate::surface::Action::AppPopup(settings, view) => {
                let Some(settings) = std::sync::Arc::try_unwrap(settings)
                    .ok()
                    .and_then(|s| s.downcast::<Box<dyn Fn(&mut T) -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings + Send + Sync>>().ok()) else {
                    tracing::error!("Invalid settings for popup");
                    return Task::none();
                };

                if let Some(view) = view.and_then(|view| {
                    match std::sync::Arc::try_unwrap(view).ok()?.downcast::<Box<
                        dyn for<'a> Fn(&'a T) -> Element<'a, crate::Action<T::Message>>
                            + Send
                            + Sync,
                    >>() {
                        Ok(v) => Some(v),
                        Err(err) => {
                            tracing::error!("Invalid view for subsurface view: {err:?}");
                            None
                        }
                    }
                }) {
                    let settings = settings(&mut self.app);

                    self.get_popup(settings, *view)
                } else {
                    iced_winit::commands::popup::get_popup(settings(&mut self.app))
                }
            }
            #[cfg(feature = "wayland")]
            crate::surface::Action::DestroyPopup(id) => {
                iced_winit::commands::popup::destroy_popup(id)
            }
            #[cfg(feature = "wayland")]
            crate::surface::Action::DestroySubsurface(id) => {
                iced_winit::commands::subsurface::destroy_subsurface(id)
            }
            #[cfg(feature = "wayland")]
            crate::surface::Action::DestroyWindow(id) => iced::window::close(id),
            crate::surface::Action::ResponsiveMenuBar {
                menu_bar,
                limits,
                size,
            } => {
                let core = self.app.core_mut();
                core.menu_bars.insert(menu_bar, (limits, size));
                iced::Task::none()
            }
            #[cfg(feature = "wayland")]
            crate::surface::Action::Popup(settings, view) => {
                let Some(settings) = std::sync::Arc::try_unwrap(settings)
                    .ok()
                    .and_then(|s| s.downcast::<Box<dyn Fn() -> iced_runtime::platform_specific::wayland::popup::SctkPopupSettings + Send + Sync>>().ok()) else {
                    tracing::error!("Invalid settings for popup");
                    return Task::none();
                };

                if let Some(view) = view.and_then(|view| {
                    match std::sync::Arc::try_unwrap(view).ok()?.downcast::<Box<
                            dyn Fn() -> Element<'static, crate::Action<T::Message>> + Send + Sync,
                        >>() {
                            Ok(v) => Some(v),
                            Err(err) => {
                                tracing::error!("Invalid view for subsurface view: {err:?}");
                                None
                            }
                        }
                }) {
                    let settings = settings();

                    self.get_popup(settings, Box::new(move |_| view()))
                } else {
                    iced_winit::commands::popup::get_popup(settings())
                }
            }
            #[cfg(feature = "wayland")]
            crate::surface::Action::AppWindow(id, settings, view) => {
                let Some(settings) = std::sync::Arc::try_unwrap(settings).ok().and_then(|s| {
                    s.downcast::<Box<dyn Fn(&mut T) -> iced::window::Settings + Send + Sync>>()
                        .ok()
                }) else {
                    tracing::error!("Invalid settings for AppWindow");
                    return Task::none();
                };

                if let Some(view) = view.and_then(|view| {
                    match std::sync::Arc::try_unwrap(view).ok()?.downcast::<Box<
                        dyn for<'a> Fn(&'a T) -> Element<'a, crate::Action<T::Message>>
                            + Send
                            + Sync,
                    >>() {
                        Ok(v) => Some(v),
                        Err(err) => {
                            tracing::error!("Invalid view for AppWindow: {err:?}");
                            None
                        }
                    }
                }) {
                    let settings = settings(&mut self.app);
                    self.tracked_windows.insert(id);

                    self.get_window(id, settings, *view)
                } else {
                    let settings = settings(&mut self.app);

                    self.tracked_windows.insert(id);
                    iced_runtime::task::oneshot(|channel| {
                        iced_runtime::Action::Window(iced_runtime::window::Action::Open(
                            id, settings, channel,
                        ))
                    })
                    .discard()
                }
            }
            #[cfg(feature = "wayland")]
            crate::surface::Action::Window(id, settings, view) => {
                let Some(settings) = std::sync::Arc::try_unwrap(settings).ok().and_then(|s| {
                    s.downcast::<Box<dyn Fn() -> iced::window::Settings + Send + Sync>>()
                        .ok()
                }) else {
                    tracing::error!("Invalid settings for Window");
                    return Task::none();
                };

                if let Some(view) = view.and_then(|view| {
                    match std::sync::Arc::try_unwrap(view).ok()?.downcast::<Box<
                            dyn Fn() -> Element<'static, crate::Action<T::Message>> + Send + Sync,
                        >>() {
                            Ok(v) => Some(v),
                            Err(err) => {
                                tracing::error!("Invalid view for Window: {err:?}");
                                None
                            }
                        }
                }) {
                    let settings = settings();
                    self.tracked_windows.insert(id);

                    self.get_window(id, settings, Box::new(move |_| view()))
                } else {
                    let settings = settings();

                    self.tracked_windows.insert(id);

                    iced_runtime::task::oneshot(|channel| {
                        iced_runtime::Action::Window(iced_runtime::window::Action::Open(
                            id, settings, channel,
                        ))
                    })
                    .discard()
                }
            }

            crate::surface::Action::Ignore => iced::Task::none(),
            crate::surface::Action::Task(f) => {
                f().map(|sm| crate::Action::Cosmic(Action::Surface(sm)))
            }
            _ => iced::Task::none(),
        }

        #[cfg(not(feature = "surface-message"))]
        iced::Task::none()
    }

    pub fn update(
        &mut self,
        message: crate::Action<T::Message>,
    ) -> iced::Task<crate::Action<T::Message>> {
        let message = match message {
            crate::Action::App(message) => self.app.update(message),
            crate::Action::Cosmic(message) => self.cosmic_update(message),
            crate::Action::None => iced::Task::none(),
            #[cfg(feature = "single-instance")]
            crate::Action::DbusActivation(message) => self.app.dbus_activation(message),
        };

        #[cfg(all(target_env = "gnu", not(target_os = "windows")))]
        crate::malloc::trim(0);

        message
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
        } else if self.app.core().window.is_maximized {
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
    #[cold]
    pub fn subscription(&self) -> Subscription<crate::Action<T::Message>> {
        let window_events = listen_with(|event, _, id| {
            match event {
                iced::Event::Window(window::Event::Resized(iced::Size { width, height })) => {
                    return Some(Action::WindowResize(id, width, height));
                }
                iced::Event::Window(window::Event::Opened { .. }) => {
                    return Some(Action::Opened(id));
                }
                iced::Event::Window(window::Event::Closed) => {
                    return Some(Action::SurfaceClosed(id));
                }
                iced::Event::Window(window::Event::Focused) => return Some(Action::Focus(id)),
                iced::Event::Window(window::Event::Unfocused) => return Some(Action::Unfocus(id)),
                #[cfg(feature = "wayland")]
                iced::Event::PlatformSpecific(iced::event::PlatformSpecific::Wayland(event)) => {
                    match event {
                        wayland::Event::Popup(wayland::PopupEvent::Done, _, id)
                        | wayland::Event::Layer(wayland::LayerEvent::Done, _, id) => {
                            return Some(Action::SurfaceClosed(id));
                        }
                        #[cfg(feature = "applet")]
                        wayland::Event::Window(
                            iced::event::wayland::WindowEvent::SuggestedBounds(b),
                        ) => {
                            return Some(Action::SuggestedBounds(b));
                        }
                        #[cfg(feature = "wayland")]
                        wayland::Event::Window(iced::event::wayland::WindowEvent::WindowState(
                            s,
                        )) => {
                            return Some(Action::WindowState(id, s));
                        }
                        _ => (),
                    }
                }
                _ => (),
            }

            None
        });

        let mut subscriptions = vec![
            self.app.subscription().map(crate::Action::App),
            self.app
                .core()
                .watch_config::<crate::config::CosmicTk>(crate::config::ID)
                .map(|update| {
                    for why in update
                        .errors
                        .into_iter()
                        .filter(cosmic_config::Error::is_err)
                    {
                        if let cosmic_config::Error::GetKey(_, err) = &why {
                            if err.kind() == std::io::ErrorKind::NotFound {
                                // No system default config installed; don't error
                                continue;
                            }
                        }
                        tracing::error!(?why, "cosmic toolkit config update error");
                    }

                    crate::Action::Cosmic(Action::ToolkitConfig(update.config))
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
                    for why in update
                        .errors
                        .into_iter()
                        .filter(cosmic_config::Error::is_err)
                    {
                        tracing::error!(?why, "cosmic theme config update error");
                    }
                    Action::SystemThemeChange(
                        update.keys,
                        crate::theme::Theme::system(Arc::new(update.config)),
                    )
                })
                .map(crate::Action::Cosmic),
            self.app
                .core()
                .watch_config::<ThemeMode>(cosmic_theme::THEME_MODE_ID)
                .map(|update| {
                    for error in update
                        .errors
                        .into_iter()
                        .filter(cosmic_config::Error::is_err)
                    {
                        tracing::error!(?error, "error reading system theme mode update");
                    }
                    Action::SystemThemeModeChange(update.keys, update.config)
                })
                .map(crate::Action::Cosmic),
            window_events.map(crate::Action::Cosmic),
            #[cfg(feature = "xdg-portal")]
            crate::theme::portal::desktop_settings()
                .map(Action::DesktopSettings)
                .map(crate::Action::Cosmic),
        ];

        if self.app.core().keyboard_nav {
            subscriptions.push(
                keyboard_nav::subscription()
                    .map(Action::KeyboardNav)
                    .map(crate::Action::Cosmic),
            );
        }

        #[cfg(feature = "single-instance")]
        if self.app.core().single_instance {
            subscriptions.push(crate::dbus_activation::subscription::<T>());
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
    pub fn view(&self, id: window::Id) -> Element<'_, crate::Action<T::Message>> {
        #[cfg(feature = "wayland")]
        if let Some((_, _, v)) = self.surface_views.get(&id) {
            return v(&self.app);
        }
        if self
            .app
            .core()
            .main_window_id()
            .is_none_or(|main_id| main_id != id)
        {
            return self.app.view_window(id).map(crate::Action::App);
        }

        let view = if self.app.core().window.use_template {
            self.app.view_main()
        } else {
            self.app.view().map(crate::Action::App)
        };

        #[cfg(all(target_env = "gnu", not(target_os = "windows")))]
        crate::malloc::trim(0);

        view
    }

    #[cfg(not(feature = "multi-window"))]
    pub fn view(&self) -> Element<crate::Action<T::Message>> {
        let view = self.app.view_main();

        #[cfg(all(target_env = "gnu", not(target_os = "windows")))]
        crate::malloc::trim(0);

        view
    }
}

impl<T: Application> Cosmic<T> {
    #[allow(clippy::unused_self)]
    #[cold]
    pub fn close(&mut self) -> iced::Task<crate::Action<T::Message>> {
        if let Some(id) = self.app.core().main_window_id() {
            iced::window::close(id)
        } else {
            iced::Task::none()
        }
    }

    #[allow(clippy::too_many_lines)]
    fn cosmic_update(&mut self, message: Action) -> iced::Task<crate::Action<T::Message>> {
        match message {
            Action::WindowMaximized(id, maximized) => {
                #[cfg(not(feature = "wayland"))]
                if self
                    .app
                    .core()
                    .main_window_id()
                    .is_some_and(|main_id| main_id == id)
                {
                    self.app.core_mut().window.sharp_corners = maximized;
                }
            }

            Action::WindowResize(id, width, height) => {
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
                    crate::Action::Cosmic(Action::WindowMaximized(id, maximized))
                });
            }

            #[cfg(feature = "wayland")]
            Action::WindowState(id, state) => {
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
                    self.app.core_mut().window.is_maximized =
                        state.intersects(WindowState::MAXIMIZED | WindowState::FULLSCREEN);
                }
                if self.app.core().sync_window_border_radii_to_theme() {
                    use iced_runtime::platform_specific::wayland::CornerRadius;
                    use iced_winit::platform_specific::commands::corner_radius::corner_radius;

                    let theme = THEME.lock().unwrap();
                    let t = theme.cosmic();
                    let radii = t.radius_s().map(|x| if x < 4.0 { x } else { x + 4.0 });
                    let cur_rad = CornerRadius {
                        top_left: radii[0].round() as u32,
                        top_right: radii[1].round() as u32,
                        bottom_right: radii[2].round() as u32,
                        bottom_left: radii[3].round() as u32,
                    };
                    let rounded = !self.app.core().window.sharp_corners;
                    return Task::batch([corner_radius(
                        id,
                        if rounded {
                            Some(cur_rad)
                        } else {
                            let rad_0 = t.radius_0();
                            Some(CornerRadius {
                                top_left: rad_0[0].round() as u32,
                                top_right: rad_0[1].round() as u32,
                                bottom_right: rad_0[2].round() as u32,
                                bottom_left: rad_0[3].round() as u32,
                            })
                        },
                    )
                    .discard()]);
                }
            }

            #[cfg(feature = "wayland")]
            Action::WmCapabilities(id, capabilities) => {
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

            Action::KeyboardNav(message) => match message {
                keyboard_nav::Action::FocusNext => {
                    return iced::widget::focus_next().map(crate::Action::Cosmic);
                }
                keyboard_nav::Action::FocusPrevious => {
                    return iced::widget::focus_previous().map(crate::Action::Cosmic);
                }
                keyboard_nav::Action::Escape => return self.app.on_escape(),
                keyboard_nav::Action::Search => return self.app.on_search(),

                keyboard_nav::Action::Fullscreen => return self.app.core().toggle_maximize(None),
            },

            Action::ContextDrawer(show) => {
                self.app.core_mut().set_show_context(show);
                return self.app.on_context_drawer();
            }

            Action::Drag => return self.app.core().drag(None),

            Action::Minimize => return self.app.core().minimize(None),

            Action::Maximize => return self.app.core().toggle_maximize(None),

            Action::NavBar(key) => {
                self.app.core_mut().nav_bar_set_toggled_condensed(false);
                return self.app.on_nav_select(key);
            }

            Action::NavBarContext(key) => {
                self.app.core_mut().nav_bar_set_context(key);
                return self.app.on_nav_context(key);
            }

            Action::ToggleNavBar => {
                self.app.core_mut().nav_bar_toggle();
            }

            Action::ToggleNavBarCondensed => {
                self.app.core_mut().nav_bar_toggle_condensed();
            }

            Action::AppThemeChange(mut theme) => {
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

            Action::SystemThemeChange(keys, theme) => {
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
                        #[cfg(feature = "wayland")]
                        if self.app.core().sync_window_border_radii_to_theme() {
                            use iced_runtime::platform_specific::wayland::CornerRadius;
                            use iced_winit::platform_specific::commands::corner_radius::corner_radius;

                            let t = cosmic_theme.cosmic();

                            let radii = t.radius_s().map(|x| if x < 4.0 { x } else { x + 4.0 });
                            let cur_rad = CornerRadius {
                                top_left: radii[0].round() as u32,
                                top_right: radii[1].round() as u32,
                                bottom_right: radii[2].round() as u32,
                                bottom_left: radii[3].round() as u32,
                            };

                            let rounded = !self.app.core().window.sharp_corners;
                            // Update radius for the main window
                            let main_window_id = self
                                .app
                                .core()
                                .main_window_id()
                                .unwrap_or(window::Id::RESERVED);
                            let mut cmds = vec![
                                corner_radius(
                                    main_window_id,
                                    if rounded {
                                        Some(cur_rad)
                                    } else {
                                        let rad_0 = t.radius_0();
                                        Some(CornerRadius {
                                            top_left: rad_0[0].round() as u32,
                                            top_right: rad_0[1].round() as u32,
                                            bottom_right: rad_0[2].round() as u32,
                                            bottom_left: rad_0[3].round() as u32,
                                        })
                                    },
                                )
                                .discard(),
                            ];
                            // Update radius for each tracked view with the window surface type
                            for (id, (_, surface_type, _)) in self.surface_views.iter() {
                                if let SurfaceIdWrapper::Window(_) = surface_type {
                                    cmds.push(
                                        corner_radius(
                                            *id,
                                            if rounded {
                                                Some(cur_rad)
                                            } else {
                                                let rad_0 = t.radius_0();
                                                Some(CornerRadius {
                                                    top_left: rad_0[0].round() as u32,
                                                    top_right: rad_0[1].round() as u32,
                                                    bottom_right: rad_0[2].round() as u32,
                                                    bottom_left: rad_0[3].round() as u32,
                                                })
                                            },
                                        )
                                        .discard(),
                                    );
                                }
                            }
                            // Update radius for all tracked windows
                            for id in self.tracked_windows.iter() {
                                cmds.push(
                                    corner_radius(
                                        *id,
                                        if rounded {
                                            Some(cur_rad)
                                        } else {
                                            let rad_0 = t.radius_0();
                                            Some(CornerRadius {
                                                top_left: rad_0[0].round() as u32,
                                                top_right: rad_0[1].round() as u32,
                                                bottom_right: rad_0[2].round() as u32,
                                                bottom_left: rad_0[3].round() as u32,
                                            })
                                        },
                                    )
                                    .discard(),
                                );
                            }

                            return Task::batch(cmds);
                        }
                    }
                }

                return cmd;
            }

            Action::ScaleFactor(factor) => {
                self.app.core_mut().set_scale_factor(factor);
            }

            Action::Close => {
                return match self.app.on_app_exit() {
                    Some(message) => self.app.update(message),
                    None => self.close(),
                };
            }
            Action::SystemThemeModeChange(keys, mode) => {
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
                core.system_theme_mode = mode;
                let is_dark = core.system_is_dark();
                let changed = core.system_theme_mode.is_dark != is_dark
                    || core.portal_is_dark != Some(is_dark)
                    || core.system_theme.cosmic().is_dark != is_dark;
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
                        if let ThemeType::System { .. } = cosmic_theme.theme_type {
                            cosmic_theme.set_theme(new_theme.theme_type);
                            #[cfg(feature = "wayland")]
                            if self.app.core().sync_window_border_radii_to_theme() {
                                use iced_runtime::platform_specific::wayland::CornerRadius;
                                use iced_winit::platform_specific::commands::corner_radius::corner_radius;

                                let t = cosmic_theme.cosmic();

                                let radii = t.radius_s().map(|x| if x < 4.0 { x } else { x + 4.0 });
                                let cur_rad = CornerRadius {
                                    top_left: radii[0].round() as u32,
                                    top_right: radii[1].round() as u32,
                                    bottom_right: radii[2].round() as u32,
                                    bottom_left: radii[3].round() as u32,
                                };
                                let rounded = !self.app.core().window.sharp_corners;

                                // Update radius for the main window
                                let main_window_id = self
                                    .app
                                    .core()
                                    .main_window_id()
                                    .unwrap_or(window::Id::RESERVED);
                                let mut cmds = vec![
                                    corner_radius(
                                        main_window_id,
                                        if rounded {
                                            Some(cur_rad)
                                        } else {
                                            let rad_0 = t.radius_0();
                                            Some(CornerRadius {
                                                top_left: rad_0[0].round() as u32,
                                                top_right: rad_0[1].round() as u32,
                                                bottom_right: rad_0[2].round() as u32,
                                                bottom_left: rad_0[3].round() as u32,
                                            })
                                        },
                                    )
                                    .discard(),
                                ];
                                // Update radius for each tracked view with the window surface type
                                for (id, (_, surface_type, _)) in self.surface_views.iter() {
                                    if let SurfaceIdWrapper::Window(_) = surface_type {
                                        cmds.push(
                                            corner_radius(
                                                *id,
                                                if rounded {
                                                    Some(cur_rad)
                                                } else {
                                                    let rad_0 = t.radius_0();
                                                    Some(CornerRadius {
                                                        top_left: rad_0[0].round() as u32,
                                                        top_right: rad_0[1].round() as u32,
                                                        bottom_right: rad_0[2].round() as u32,
                                                        bottom_left: rad_0[3].round() as u32,
                                                    })
                                                },
                                            )
                                            .discard(),
                                        );
                                    }
                                }
                                // Update radius for all tracked windows
                                for id in self.tracked_windows.iter() {
                                    cmds.push(
                                        corner_radius(
                                            *id,
                                            if rounded {
                                                Some(cur_rad)
                                            } else {
                                                let rad_0 = t.radius_0();
                                                Some(CornerRadius {
                                                    top_left: rad_0[0].round() as u32,
                                                    top_right: rad_0[1].round() as u32,
                                                    bottom_right: rad_0[2].round() as u32,
                                                    bottom_left: rad_0[3].round() as u32,
                                                })
                                            },
                                        )
                                        .discard(),
                                    );
                                }

                                return Task::batch(cmds);
                            }
                        }
                    }
                }
                return Task::batch(cmds);
            }
            Action::Activate(_token) =>
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

            Action::Surface(action) => return self.surface_update(action),

            Action::SurfaceClosed(id) => {
                if self.opened_surfaces.get_mut(&id).is_some_and(|v| {
                    *v = v.saturating_sub(1);
                    *v == 0
                }) {
                    self.opened_surfaces.remove(&id);
                    #[cfg(feature = "wayland")]
                    self.surface_views.remove(&id);
                    self.tracked_windows.remove(&id);
                }

                let mut ret = if let Some(msg) = self.app.on_close_requested(id) {
                    self.app.update(msg)
                } else {
                    Task::none()
                };
                let core = self.app.core();
                if core.exit_on_main_window_closed
                    && core.main_window_id().is_some_and(|m_id| id == m_id)
                {
                    ret = Task::batch([iced::exit::<crate::Action<T::Message>>()]);
                }
                return ret;
            }

            Action::ShowWindowMenu => {
                if let Some(id) = self.app.core().main_window_id() {
                    return iced::window::show_system_menu(id);
                }
            }

            #[cfg(feature = "single-instance")]
            Action::DbusConnection(conn) => {
                return self.app.dbus_connection(conn);
            }

            #[cfg(feature = "xdg-portal")]
            Action::DesktopSettings(crate::theme::portal::Desktop::ColorScheme(s)) => {
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

                core.portal_is_dark = is_dark;
                let is_dark = core.system_is_dark();
                let changed = core.system_theme_mode.is_dark != is_dark
                    || core.portal_is_dark != Some(is_dark)
                    || core.system_theme.cosmic().is_dark != is_dark;

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
            Action::DesktopSettings(crate::theme::portal::Desktop::Accent(c)) => {
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
            Action::DesktopSettings(crate::theme::portal::Desktop::Contrast(_)) => {
                // TODO when high contrast is integrated in settings and all custom themes
            }

            Action::ToolkitConfig(config) => {
                // Change the icon theme if not defined by the application.
                if !self.app.core().icon_theme_override
                    && crate::icon_theme::default() != config.icon_theme
                {
                    crate::icon_theme::set_default(config.icon_theme.clone());
                }

                *crate::config::COSMIC_TK.write().unwrap() = config;
            }

            Action::Focus(f) => {
                #[cfg(all(
                    feature = "wayland",
                    feature = "multi-window",
                    feature = "surface-message"
                ))]
                if let Some((
                    parent,
                    SurfaceIdWrapper::Subsurface(_) | SurfaceIdWrapper::Popup(_),
                    _,
                )) = self.surface_views.get(&f)
                {
                    // If the parent is already focused, push the new focus
                    // to the end of the focus chain.
                    if parent.is_some_and(|p| self.app.core().focused_window.last() == Some(&p)) {
                        self.app.core_mut().focused_window.push(f);
                        return iced::Task::none();
                    } else {
                        // set the whole parent chain to the focus chain
                        let mut parent_chain = vec![f];
                        let mut cur = *parent;
                        while let Some(p) = cur {
                            parent_chain.push(p);
                            cur = self
                                .surface_views
                                .get(&p)
                                .and_then(|(parent, _, _)| *parent);
                        }
                        parent_chain.reverse();
                        self.app.core_mut().focused_window = parent_chain;
                        return iced::Task::none();
                    }
                }
                self.app.core_mut().focused_window = vec![f];
            }

            Action::Unfocus(id) => {
                let core = self.app.core_mut();
                if core.focused_window().as_ref().is_some_and(|cur| *cur == id) {
                    core.focused_window.pop();
                }
            }
            #[cfg(feature = "applet")]
            Action::SuggestedBounds(b) => {
                tracing::info!("Suggested bounds: {b:?}");
                let core = self.app.core_mut();
                core.applet.suggested_bounds = b;
            }
            Action::Opened(id) => {
                #[cfg(feature = "wayland")]
                if self.app.core().sync_window_border_radii_to_theme() {
                    use iced_runtime::platform_specific::wayland::CornerRadius;
                    use iced_winit::platform_specific::commands::corner_radius::corner_radius;

                    let theme = THEME.lock().unwrap();
                    let t = theme.cosmic();
                    let radii = t.radius_s().map(|x| if x < 4.0 { x } else { x + 4.0 });
                    let cur_rad = CornerRadius {
                        top_left: radii[0].round() as u32,
                        top_right: radii[1].round() as u32,
                        bottom_right: radii[2].round() as u32,
                        bottom_left: radii[3].round() as u32,
                    };
                    // TODO do we need per window sharp corners?
                    let rounded = !self.app.core().window.sharp_corners;

                    return Task::batch([
                        corner_radius(
                            id,
                            if rounded {
                                Some(cur_rad)
                            } else {
                                let rad_0 = t.radius_0();
                                Some(CornerRadius {
                                    top_left: rad_0[0].round() as u32,
                                    top_right: rad_0[1].round() as u32,
                                    bottom_right: rad_0[2].round() as u32,
                                    bottom_left: rad_0[3].round() as u32,
                                })
                            },
                        )
                        .discard(),
                        iced_runtime::window::run_with_handle(id, init_windowing_system),
                    ]);
                }
                return iced_runtime::window::run_with_handle(id, init_windowing_system);
            }
            _ => {}
        }

        iced::Task::none()
    }
}

impl<App: Application> Cosmic<App> {
    pub fn new(app: App) -> Self {
        Self {
            app,
            #[cfg(feature = "wayland")]
            surface_views: HashMap::new(),
            tracked_windows: HashSet::new(),
            opened_surfaces: HashMap::new(),
        }
    }

    #[cfg(feature = "wayland")]
    /// Create a subsurface
    pub fn get_subsurface(
        &mut self,
        settings: iced_runtime::platform_specific::wayland::subsurface::SctkSubsurfaceSettings,
        view: Box<
            dyn for<'a> Fn(&'a App) -> Element<'a, crate::Action<App::Message>> + Send + Sync,
        >,
    ) -> Task<crate::Action<App::Message>> {
        use iced_winit::commands::subsurface::get_subsurface;

        *self.opened_surfaces.entry(settings.id).or_insert_with(|| 0) += 1;
        self.surface_views.insert(
            settings.id,
            (
                Some(settings.parent),
                SurfaceIdWrapper::Subsurface(settings.id),
                view,
            ),
        );
        get_subsurface(settings)
    }

    #[cfg(feature = "wayland")]
    /// Create a subsurface
    pub fn get_popup(
        &mut self,
        settings: iced_runtime::platform_specific::wayland::popup::SctkPopupSettings,
        view: Box<
            dyn for<'a> Fn(&'a App) -> Element<'a, crate::Action<App::Message>> + Send + Sync,
        >,
    ) -> Task<crate::Action<App::Message>> {
        use iced_winit::commands::popup::get_popup;
        *self.opened_surfaces.entry(settings.id).or_insert_with(|| 0) += 1;
        self.surface_views.insert(
            settings.id,
            (
                Some(settings.parent),
                SurfaceIdWrapper::Popup(settings.id),
                view,
            ),
        );
        get_popup(settings)
    }

    #[cfg(feature = "wayland")]
    /// Create a window surface
    pub fn get_window(
        &mut self,
        id: iced::window::Id,
        settings: iced::window::Settings,
        view: Box<
            dyn for<'a> Fn(&'a App) -> Element<'a, crate::Action<App::Message>> + Send + Sync,
        >,
    ) -> Task<crate::Action<App::Message>> {
        use iced_winit::SurfaceIdWrapper;
        *self.opened_surfaces.entry(id).or_insert(0) += 1;
        self.surface_views.insert(
            id,
            (
                None, // TODO parent for window, platform specific option maybe?
                SurfaceIdWrapper::Window(id),
                view,
            ),
        );
        iced_runtime::task::oneshot(|channel| {
            iced_runtime::Action::Window(iced_runtime::window::Action::Open(id, settings, channel))
        })
        .discard()
    }
}
