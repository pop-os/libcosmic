// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Build interactive cross-platform COSMIC applications.
//!
//! Check out our [application](https://github.com/pop-os/libcosmic/tree/master/examples/application)
//! example in our repository.

mod action;
pub use action::Action;
use cosmic_config::CosmicConfigEntry;
pub mod context_drawer;
pub use context_drawer::{ContextDrawer, context_drawer};
pub mod cosmic;
#[cfg(all(feature = "winit", feature = "multi-window"))]
pub(crate) mod multi_window;
pub mod settings;

pub type Task<M> = iced::Task<crate::Action<M>>;

pub use crate::Core;
use crate::prelude::*;
use crate::theme::THEME;
use crate::widget::{container, horizontal_space, id_container, menu, nav_bar, popover};
use apply::Apply;
use iced::window;
use iced::{Length, Subscription};
pub use settings::Settings;
use std::borrow::Cow;

#[cold]
pub(crate) fn iced_settings<App: Application>(
    settings: Settings,
    flags: App::Flags,
) -> (iced::Settings, (Core, App::Flags), iced::window::Settings) {
    preload_fonts();

    let mut core = Core::default();
    core.debug = settings.debug;
    core.icon_theme_override = settings.default_icon_theme.is_some();
    core.set_scale_factor(settings.scale_factor);
    core.set_window_width(settings.size.width);
    core.set_window_height(settings.size.height);

    if let Some(icon_theme) = settings.default_icon_theme {
        crate::icon_theme::set_default(icon_theme);
    } else {
        crate::icon_theme::set_default(crate::config::icon_theme());
    }

    THEME.lock().unwrap().set_theme(settings.theme.theme_type);

    if settings.no_main_window {
        core.main_window = Some(iced::window::Id::NONE);
    }

    let mut iced = iced::Settings::default();

    iced.antialiasing = settings.antialiasing;
    iced.default_font = settings.default_font;
    iced.default_text_size = iced::Pixels(settings.default_text_size);
    let exit_on_close = settings.exit_on_close;
    iced.is_daemon = false;
    iced.exit_on_close_request = settings.is_daemon;
    let mut window_settings = iced::window::Settings::default();
    window_settings.exit_on_close_request = exit_on_close;
    iced.id = Some(App::APP_ID.to_owned());
    #[cfg(target_os = "linux")]
    {
        window_settings.platform_specific.application_id = App::APP_ID.to_string();
    }
    core.exit_on_main_window_closed = exit_on_close;

    if let Some(border_size) = settings.resizable {
        window_settings.resize_border = border_size as u32;
        window_settings.resizable = true;
    }
    window_settings.decorations = !settings.client_decorations;
    window_settings.size = settings.size;
    let min_size = settings.size_limits.min();
    if min_size != iced::Size::ZERO {
        window_settings.min_size = Some(min_size);
    }
    let max_size = settings.size_limits.max();
    if max_size != iced::Size::INFINITY {
        window_settings.max_size = Some(max_size);
    }

    window_settings.transparent = settings.transparent;
    (iced, (core, flags), window_settings)
}

/// Launch a COSMIC application with the given [`Settings`].
///
/// # Errors
///
/// Returns error on application failure.
pub fn run<App: Application>(settings: Settings, flags: App::Flags) -> iced::Result {
    #[cfg(all(target_env = "gnu", not(target_os = "windows")))]
    if let Some(threshold) = settings.default_mmap_threshold {
        crate::malloc::limit_mmap_threshold(threshold);
    }

    let default_font = settings.default_font;
    let (settings, mut flags, window_settings) = iced_settings::<App>(settings, flags);
    #[cfg(not(feature = "multi-window"))]
    {
        flags.0.main_window = Some(iced::window::Id::RESERVED);
        iced::application(
            cosmic::Cosmic::title,
            cosmic::Cosmic::update,
            cosmic::Cosmic::view,
        )
        .subscription(cosmic::Cosmic::subscription)
        .style(cosmic::Cosmic::style)
        .theme(cosmic::Cosmic::theme)
        .window_size((500.0, 800.0))
        .settings(settings)
        .window(window_settings)
        .run_with(move || cosmic::Cosmic::<App>::init(flags))
    }
    #[cfg(feature = "multi-window")]
    {
        let mut app = multi_window::multi_window::<_, _, _, _, App::Executor>(
            cosmic::Cosmic::title,
            cosmic::Cosmic::update,
            cosmic::Cosmic::view,
        );
        if flags.0.main_window.is_none() {
            app = app.window(window_settings);
            flags.0.main_window = Some(iced_core::window::Id::RESERVED);
        }
        app.subscription(cosmic::Cosmic::subscription)
            .style(cosmic::Cosmic::style)
            .theme(cosmic::Cosmic::theme)
            .settings(settings)
            .run_with(move || cosmic::Cosmic::<App>::init(flags))
    }
}

#[cfg(feature = "single-instance")]
/// Launch a COSMIC application with the given [`Settings`].
/// If the application is already running, the arguments will be passed to the
/// running instance.
/// # Errors
/// Returns error on application failure.
pub fn run_single_instance<App: Application>(settings: Settings, flags: App::Flags) -> iced::Result
where
    App::Flags: CosmicFlags,
    App::Message: Clone + std::fmt::Debug + Send + 'static,
{
    use std::collections::HashMap;

    let activation_token = std::env::var("XDG_ACTIVATION_TOKEN").ok();

    let override_single = std::env::var("COSMIC_SINGLE_INSTANCE")
        .map(|v| &v.to_lowercase() == "false" || &v == "0")
        .unwrap_or_default();
    if override_single {
        return run::<App>(settings, flags);
    }

    let path: String = format!("/{}", App::APP_ID.replace('.', "/"));

    let Ok(conn) = zbus::blocking::Connection::session() else {
        tracing::warn!("Failed to connect to dbus");
        return run::<App>(settings, flags);
    };

    if crate::dbus_activation::DbusActivationInterfaceProxyBlocking::builder(&conn)
        .destination(App::APP_ID)
        .ok()
        .and_then(|b| b.path(path).ok())
        .and_then(|b| b.destination(App::APP_ID).ok())
        .and_then(|b| b.build().ok())
        .is_some_and(|mut p| {
            let res = {
                let mut platform_data = HashMap::new();
                if let Some(activation_token) = activation_token {
                    platform_data.insert("activation-token", activation_token.into());
                }
                if let Ok(startup_id) = std::env::var("DESKTOP_STARTUP_ID") {
                    platform_data.insert("desktop-startup-id", startup_id.into());
                }
                if let Some(action) = flags.action() {
                    let action = action.to_string();
                    p.activate_action(&action, flags.args(), platform_data)
                } else {
                    p.activate(platform_data)
                }
            };
            match res {
                Ok(()) => {
                    tracing::info!("Successfully activated another instance");
                    true
                }
                Err(err) => {
                    tracing::warn!(?err, "Failed to activate another instance");
                    false
                }
            }
        })
    {
        tracing::info!("Another instance is running");
        Ok(())
    } else {
        let (settings, mut flags, window_settings) = iced_settings::<App>(settings, flags);
        flags.0.single_instance = true;

        #[cfg(not(feature = "multi-window"))]
        {
            iced::application(
                cosmic::Cosmic::title,
                cosmic::Cosmic::update,
                cosmic::Cosmic::view,
            )
            .subscription(cosmic::Cosmic::subscription)
            .style(cosmic::Cosmic::style)
            .theme(cosmic::Cosmic::theme)
            .window_size((500.0, 800.0))
            .settings(settings)
            .window(window_settings)
            .run_with(move || cosmic::Cosmic::<App>::init(flags))
        }
        #[cfg(feature = "multi-window")]
        {
            let mut app = multi_window::multi_window::<_, _, _, _, App::Executor>(
                cosmic::Cosmic::title,
                cosmic::Cosmic::update,
                cosmic::Cosmic::view,
            );
            if flags.0.main_window.is_none() {
                app = app.window(window_settings);
                flags.0.main_window = Some(iced_core::window::Id::RESERVED);
            }
            app.subscription(cosmic::Cosmic::subscription)
                .style(cosmic::Cosmic::style)
                .theme(cosmic::Cosmic::theme)
                .settings(settings)
                .run_with(move || cosmic::Cosmic::<App>::init(flags))
        }
    }
}

pub trait CosmicFlags {
    type SubCommand: ToString + std::fmt::Debug + Clone + Send + 'static;
    type Args: Into<Vec<String>> + std::fmt::Debug + Clone + Send + 'static;
    #[must_use]
    fn action(&self) -> Option<&Self::SubCommand> {
        None
    }

    #[must_use]
    fn args(&self) -> Vec<&str> {
        Vec::new()
    }
}

/// An interactive cross-platform COSMIC application.
#[allow(unused_variables)]
pub trait Application
where
    Self: Sized + 'static,
{
    /// Default async executor to use with the app.
    type Executor: iced_futures::Executor;

    /// Argument received [`Application::new`].
    type Flags;

    /// Message type specific to our app.
    type Message: Clone + std::fmt::Debug + Send + 'static;

    /// An ID that uniquely identifies the application.
    /// The standard is to pick an ID based on a reverse-domain name notation.
    /// IE: `com.system76.Settings`
    const APP_ID: &'static str;

    /// Grants access to the COSMIC Core.
    fn core(&self) -> &Core;

    /// Grants access to the COSMIC Core.
    fn core_mut(&mut self) -> &mut Core;

    /// Creates the application, and optionally emits task on initialize.
    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Self::Message>);

    /// Displays a context drawer on the side of the application window when `Some`.
    /// Use the [`ApplicationExt::set_show_context`] function for this to take effect.
    fn context_drawer(&self) -> Option<ContextDrawer<'_, Self::Message>> {
        None
    }

    /// Displays a dialog in the center of the application window when `Some`.
    fn dialog(&self) -> Option<Element<'_, Self::Message>> {
        None
    }

    /// Displays a footer at the bottom of the application window when `Some`.
    fn footer(&self) -> Option<Element<'_, Self::Message>> {
        None
    }

    /// Attaches elements to the start section of the header.
    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        Vec::new()
    }

    /// Attaches elements to the center of the header.
    fn header_center(&self) -> Vec<Element<'_, Self::Message>> {
        Vec::new()
    }

    /// Attaches elements to the end section of the header.
    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        Vec::new()
    }

    /// Allows overriding the default nav bar widget.
    fn nav_bar(&self) -> Option<Element<'_, crate::Action<Self::Message>>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        let nav_model = self.nav_model()?;

        let mut nav =
            crate::widget::nav_bar(nav_model, |id| crate::Action::Cosmic(Action::NavBar(id)))
                .on_context(|id| crate::Action::Cosmic(Action::NavBarContext(id)))
                .context_menu(self.nav_context_menu(self.core().nav_bar_context()))
                .into_container()
                // XXX both must be shrink to avoid flex layout from ignoring it
                .width(iced::Length::Shrink)
                .height(iced::Length::Shrink);

        if !self.core().is_condensed() {
            nav = nav.max_width(280);
        }

        Some(Element::from(nav))
    }

    /// Shows a context menu for the active nav bar item.
    fn nav_context_menu(
        &self,
        id: nav_bar::Id,
    ) -> Option<Vec<menu::Tree<crate::Action<Self::Message>>>> {
        None
    }

    /// Allows COSMIC to integrate with your application's [`nav_bar::Model`].
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        None
    }

    /// Called before closing the application. Returning a message will override closing windows.
    fn on_app_exit(&mut self) -> Option<Self::Message> {
        None
    }

    /// Called when a window requests to be closed.
    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        None
    }

    // Called when context drawer is toggled
    fn on_context_drawer(&mut self) -> Task<Self::Message> {
        Task::none()
    }

    /// Called when the escape key is pressed.
    fn on_escape(&mut self) -> Task<Self::Message> {
        Task::none()
    }

    /// Called when a navigation item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        Task::none()
    }

    /// Called when a context menu is requested for a navigation item.
    fn on_nav_context(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        Task::none()
    }

    /// Called when the search function is requested.
    fn on_search(&mut self) -> Task<Self::Message> {
        Task::none()
    }

    /// Called when a window is resized.
    fn on_window_resize(&mut self, id: window::Id, width: f32, height: f32) {}

    /// Event sources that are to be listened to.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Respond to an application-specific message.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        Task::none()
    }

    /// Respond to a system theme change
    fn system_theme_update(
        &mut self,
        keys: &[&'static str],
        new_theme: &cosmic_theme::Theme,
    ) -> Task<Self::Message> {
        Task::none()
    }

    /// Respond to a system theme mode change
    fn system_theme_mode_update(
        &mut self,
        keys: &[&'static str],
        new_theme: &cosmic_theme::ThemeMode,
    ) -> Task<Self::Message> {
        Task::none()
    }

    /// Constructs the view for the main window.
    fn view(&self) -> Element<'_, Self::Message>;

    /// Constructs views for other windows.
    fn view_window(&self, id: window::Id) -> Element<'_, Self::Message> {
        panic!("no view for window {id:?}");
    }

    /// Overrides the default style for applications
    fn style(&self) -> Option<iced_runtime::Appearance> {
        None
    }

    /// Handles dbus activation messages
    #[cfg(feature = "single-instance")]
    fn dbus_activation(&mut self, msg: crate::dbus_activation::Message) -> Task<Self::Message> {
        Task::none()
    }

    /// Invoked on connect to dbus session socket used for dbus activation
    ///
    /// Can be used to expose custom interfaces on the same owned name.
    #[cfg(feature = "single-instance")]
    fn dbus_connection(&mut self, conn: zbus::Connection) -> Task<Self::Message> {
        Task::none()
    }
}

/// Methods automatically derived for all types implementing [`Application`].
pub trait ApplicationExt: Application {
    /// Initiates a window drag.
    fn drag(&mut self) -> Task<Self::Message>;

    /// Maximizes the window.
    fn maximize(&mut self) -> Task<Self::Message>;

    /// Minimizes the window.
    fn minimize(&mut self) -> Task<Self::Message>;
    /// Get the title of the main window.

    #[cfg(not(feature = "multi-window"))]
    fn title(&self) -> &str;

    #[cfg(feature = "multi-window")]
    /// Get the title of a window.
    fn title(&self, id: window::Id) -> &str;

    /// Set the context drawer visibility.
    fn set_show_context(&mut self, show: bool) {
        self.core_mut().set_show_context(show);
    }

    /// Set the header bar title.
    fn set_header_title(&mut self, title: String) {
        self.core_mut().set_header_title(title);
    }

    #[cfg(not(feature = "multi-window"))]
    /// Set the title of the main window.
    fn set_window_title(&mut self, title: String) -> Task<Self::Message>;

    #[cfg(feature = "multi-window")]
    /// Set the title of a window.
    fn set_window_title(&mut self, title: String, id: window::Id) -> Task<Self::Message>;

    /// View template for the main window.
    fn view_main(&self) -> Element<'_, crate::Action<Self::Message>>;

    fn watch_config<T: CosmicConfigEntry + Send + Sync + Default + 'static + Clone + PartialEq>(
        &self,
        id: &'static str,
    ) -> iced::Subscription<cosmic_config::Update<T>> {
        self.core().watch_config(id)
    }

    fn watch_state<T: CosmicConfigEntry + Send + Sync + Default + 'static + Clone + PartialEq>(
        &self,
        id: &'static str,
    ) -> iced::Subscription<cosmic_config::Update<T>> {
        self.core().watch_state(id)
    }
}

impl<App: Application> ApplicationExt for App {
    fn drag(&mut self) -> Task<Self::Message> {
        self.core().drag(None)
    }

    fn maximize(&mut self) -> Task<Self::Message> {
        self.core().maximize(None, true)
    }

    fn minimize(&mut self) -> Task<Self::Message> {
        self.core().minimize(None)
    }

    #[cfg(feature = "multi-window")]
    fn title(&self, id: window::Id) -> &str {
        self.core().title.get(&id).map_or("", |s| s.as_str())
    }

    #[cfg(not(feature = "multi-window"))]
    fn title(&self) -> &str {
        self.core()
            .main_window_id()
            .and_then(|id| self.core().title.get(&id).map(std::string::String::as_str))
            .unwrap_or("")
    }

    #[cfg(feature = "multi-window")]
    fn set_window_title(&mut self, title: String, id: window::Id) -> Task<Self::Message> {
        self.core_mut().title.insert(id, title.clone());
        self.core().set_title(Some(id), title)
    }

    #[cfg(not(feature = "multi-window"))]
    fn set_window_title(&mut self, title: String) -> Task<Self::Message> {
        let Some(id) = self.core().main_window_id() else {
            return Task::none();
        };

        self.core_mut().title.insert(id, title.clone());
        Task::none()
    }

    #[allow(clippy::too_many_lines)]
    /// Creates the view for the main window.
    fn view_main(&self) -> Element<'_, crate::Action<Self::Message>> {
        let core = self.core();
        let is_condensed = core.is_condensed();
        let sharp_corners = core.window.sharp_corners;
        let maximized = core.window.is_maximized;
        let content_container = core.window.content_container;
        let show_context = core.window.show_context;
        let nav_bar_active = core.nav_bar_active();
        let focused = core
            .focus_chain()
            .iter()
            .any(|i| Some(*i) == self.core().main_window_id());

        let border_padding = if maximized { 8 } else { 7 };

        let main_content_padding = if !content_container {
            [0, 0, 0, 0]
        } else {
            let right_padding = if show_context { 0 } else { border_padding };
            let left_padding = if nav_bar_active { 0 } else { border_padding };

            [0, right_padding, 0, left_padding]
        };

        let content_row = crate::widget::row::with_children({
            let mut widgets = Vec::with_capacity(3);

            // Insert nav bar onto the left side of the window.
            let has_nav = if let Some(nav) = self
                .nav_bar()
                .map(|nav| id_container(nav, iced_core::id::Id::new("COSMIC_nav_bar")))
            {
                widgets.push(
                    container(nav)
                        .padding([
                            0,
                            if is_condensed { border_padding } else { 8 },
                            border_padding,
                            border_padding,
                        ])
                        .into(),
                );
                true
            } else {
                false
            };

            if self.nav_model().is_none() || core.show_content() {
                let main_content = self.view();

                //TODO: reduce duplication
                let context_width = core.context_width(has_nav);
                if core.window.context_is_overlay && show_context {
                    if let Some(context) = self.context_drawer() {
                        widgets.push(
                            crate::widget::context_drawer(
                                context.title,
                                context.actions,
                                context.header,
                                context.footer,
                                context.on_close,
                                main_content,
                                context.content,
                                context_width,
                            )
                            .apply(|drawer| {
                                Element::from(id_container(
                                    drawer,
                                    iced_core::id::Id::new("COSMIC_context_drawer"),
                                ))
                            })
                            .apply(container)
                            .padding([0, if content_container { border_padding } else { 0 }, 0, 0])
                            .apply(Element::from)
                            .map(crate::Action::App),
                        );
                    } else {
                        widgets.push(
                            container(main_content.map(crate::Action::App))
                                .padding(main_content_padding)
                                .into(),
                        );
                    }
                } else {
                    //TODO: hide content when out of space
                    widgets.push(
                        container(main_content.map(crate::Action::App))
                            .padding(main_content_padding)
                            .into(),
                    );
                    if let Some(context) = self.context_drawer() {
                        widgets.push(
                            crate::widget::ContextDrawer::new_inner(
                                context.title,
                                context.actions,
                                context.header,
                                context.footer,
                                context.content,
                                context.on_close,
                                context_width,
                            )
                            .apply(Element::from)
                            .map(crate::Action::App)
                            .apply(container)
                            .width(context_width)
                            .apply(|drawer| {
                                Element::from(id_container(
                                    drawer,
                                    iced_core::id::Id::new("COSMIC_context_drawer"),
                                ))
                            })
                            .apply(container)
                            .padding(if content_container {
                                [0, border_padding, border_padding, border_padding]
                            } else {
                                [0, 0, 0, 0]
                            })
                            .into(),
                        )
                    } else {
                        //TODO: this element is added to workaround state issues
                        widgets.push(horizontal_space().width(Length::Shrink).into());
                    }
                }
            }

            widgets
        });
        let content_col = crate::widget::column::with_capacity(2)
            .push(content_row)
            .push_maybe(self.footer().map(|footer| {
                container(footer.map(crate::Action::App)).padding([
                    0,
                    border_padding,
                    border_padding,
                    border_padding,
                ])
            }));
        let content: Element<_> = if content_container {
            content_col
                .apply(container)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .class(crate::theme::Container::WindowBackground)
                .apply(|w| id_container(w, iced_core::id::Id::new("COSMIC_content_container")))
                .into()
        } else {
            content_col.into()
        };

        // Ensures visually aligned radii for content and window corners
        let window_corner_radius = if sharp_corners {
            crate::theme::active().cosmic().radius_0()
        } else {
            crate::theme::active()
                .cosmic()
                .radius_s()
                .map(|x| if x < 4.0 { x } else { x + 4.0 })
        };

        let view_column = crate::widget::column::with_capacity(2)
            .push_maybe(if core.window.show_headerbar {
                Some({
                    let mut header = crate::widget::header_bar()
                        .focused(focused)
                        .maximized(maximized)
                        .sharp_corners(sharp_corners)
                        .title(&core.window.header_title)
                        .on_drag(crate::Action::Cosmic(Action::Drag))
                        .on_right_click(crate::Action::Cosmic(Action::ShowWindowMenu))
                        .on_double_click(crate::Action::Cosmic(Action::Maximize))
                        .is_condensed(is_condensed);

                    if self.nav_model().is_some() {
                        let toggle = crate::widget::nav_bar_toggle()
                            .active(core.nav_bar_active())
                            .selected(focused)
                            .on_toggle(if is_condensed {
                                crate::Action::Cosmic(Action::ToggleNavBarCondensed)
                            } else {
                                crate::Action::Cosmic(Action::ToggleNavBar)
                            });

                        header = header.start(toggle);
                    }

                    if core.window.show_close {
                        header = header.on_close(crate::Action::Cosmic(Action::Close));
                    }

                    if core.window.show_maximize && crate::config::show_maximize() {
                        header = header.on_maximize(crate::Action::Cosmic(Action::Maximize));
                    }

                    if core.window.show_minimize && crate::config::show_minimize() {
                        header = header.on_minimize(crate::Action::Cosmic(Action::Minimize));
                    }

                    for element in self.header_start() {
                        header = header.start(element.map(crate::Action::App));
                    }

                    for element in self.header_center() {
                        header = header.center(element.map(crate::Action::App));
                    }

                    for element in self.header_end() {
                        header = header.end(element.map(crate::Action::App));
                    }

                    if content_container {
                        header.apply(|w| id_container(w, iced_core::id::Id::new("COSMIC_header")))
                    } else {
                        // Needed to avoid header bar corner gaps for apps without a content container
                        header
                            .apply(container)
                            .class(crate::theme::Container::custom(move |theme| {
                                let cosmic = theme.cosmic();
                                container::Style {
                                    background: Some(iced::Background::Color(
                                        cosmic.background.base.into(),
                                    )),
                                    border: iced::Border {
                                        radius: [
                                            (window_corner_radius[0] - 1.0).max(0.0),
                                            (window_corner_radius[1] - 1.0).max(0.0),
                                            cosmic.radius_0()[2],
                                            cosmic.radius_0()[3],
                                        ]
                                        .into(),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                }
                            }))
                            .apply(|w| id_container(w, iced_core::id::Id::new("COSMIC_header")))
                    }
                })
            } else {
                None
            })
            // The content element contains every element beneath the header.
            .push(content)
            .apply(container)
            .padding(if maximized { 0 } else { 1 })
            .class(crate::theme::Container::custom(move |theme| {
                container::Style {
                    background: if content_container {
                        Some(iced::Background::Color(
                            theme.cosmic().background.base.into(),
                        ))
                    } else {
                        None
                    },
                    border: iced::Border {
                        color: theme.cosmic().bg_divider().into(),
                        width: if maximized { 0.0 } else { 1.0 },
                        radius: window_corner_radius.into(),
                    },
                    ..Default::default()
                }
            }));

        // Show any current dialog on top and centered over the view content
        // We have to use a popover even without a dialog to keep the tree from changing
        let mut popover = popover(view_column).modal(true);
        if let Some(dialog) = self
            .dialog()
            .map(|w| Element::from(id_container(w, iced_core::id::Id::new("COSMIC_dialog"))))
        {
            popover = popover.popup(dialog.map(crate::Action::App));
        }

        let view_element: Element<_> = popover.into();
        view_element.debug(core.debug)
    }
}

const EMBEDDED_FONTS: &[&[u8]] = &[
    include_bytes!("../../res/open-sans/OpenSans-Light.ttf"),
    include_bytes!("../../res/open-sans/OpenSans-Regular.ttf"),
    include_bytes!("../../res/open-sans/OpenSans-Semibold.ttf"),
    include_bytes!("../../res/open-sans/OpenSans-Bold.ttf"),
    include_bytes!("../../res/open-sans/OpenSans-ExtraBold.ttf"),
    include_bytes!("../../res/noto/NotoSansMono-Regular.ttf"),
    include_bytes!("../../res/noto/NotoSansMono-Bold.ttf"),
];

#[cold]
fn preload_fonts() {
    let mut font_system = iced::advanced::graphics::text::font_system()
        .write()
        .unwrap();

    EMBEDDED_FONTS
        .iter()
        .for_each(move |font| font_system.load_font(Cow::Borrowed(font)));
}
