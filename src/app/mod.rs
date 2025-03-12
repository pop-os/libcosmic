// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Build interactive cross-platform COSMIC applications.
//!
//! Check out our [application](https://github.com/pop-os/libcosmic/tree/master/examples/application)
//! example in our repository.

pub mod command;
pub mod context_drawer;
mod core;
pub mod cosmic;
#[cfg(all(feature = "winit", feature = "multi-window"))]
pub(crate) mod multi_window;
pub mod settings;

pub mod message {
    #[derive(Clone, Debug)]
    #[must_use]
    pub enum Message<M> {
        /// Messages from the application, for the application.
        App(M),
        /// Internal messages to be handled by libcosmic.
        Cosmic(super::cosmic::Message),
        #[cfg(feature = "single-instance")]
        /// Dbus activation messages
        DbusActivation(super::DbusActivationMessage),
        /// Do nothing
        None,
    }

    pub const fn app<M>(message: M) -> Message<M> {
        Message::App(message)
    }

    pub const fn cosmic<M>(message: super::cosmic::Message) -> Message<M> {
        Message::Cosmic(message)
    }

    pub const fn none<M>() -> Message<M> {
        Message::None
    }

    impl<M> From<M> for Message<M> {
        fn from(value: M) -> Self {
            Self::App(value)
        }
    }
}

use std::borrow::Cow;

pub use self::command::Task;
pub use self::core::Core;
pub use self::settings::Settings;
use crate::prelude::*;
use crate::theme::THEME;
use crate::widget::{container, horizontal_space, id_container, menu, nav_bar, popover};
use apply::Apply;
use context_drawer::ContextDrawer;
use iced::window;
use iced::{Length, Subscription};
pub use message::Message;
use url::Url;
#[cfg(feature = "single-instance")]
use {
    iced_futures::futures::channel::mpsc::{Receiver, Sender},
    iced_futures::futures::SinkExt,
    std::any::TypeId,
    std::collections::HashMap,
    zbus::{interface, proxy, zvariant::Value},
};

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
    #[cfg(target_env = "gnu")]
    if let Some(threshold) = settings.default_mmap_threshold {
        crate::malloc::limit_mmap_threshold(threshold);
    }

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
#[derive(Debug, Clone)]
pub struct DbusActivationMessage<Action = String, Args = Vec<String>> {
    pub activation_token: Option<String>,
    pub desktop_startup_id: Option<String>,
    pub msg: DbusActivationDetails<Action, Args>,
}

#[derive(Debug, Clone)]
pub enum DbusActivationDetails<Action = String, Args = Vec<String>> {
    Activate,
    Open {
        url: Vec<Url>,
    },
    /// action can be deserialized as Flags
    ActivateAction {
        action: Action,
        args: Args,
    },
}
#[cfg(feature = "single-instance")]
#[derive(Debug, Default)]
pub struct DbusActivation(Option<Sender<DbusActivationMessage>>);
#[cfg(feature = "single-instance")]
impl DbusActivation {
    #[must_use]
    pub fn new() -> Self {
        Self(None)
    }

    pub fn rx(&mut self) -> Receiver<DbusActivationMessage> {
        let (tx, rx) = iced_futures::futures::channel::mpsc::channel(10);
        self.0 = Some(tx);
        rx
    }
}

#[cfg(feature = "single-instance")]
#[proxy(interface = "org.freedesktop.DbusActivation", assume_defaults = true)]
pub trait DbusActivationInterface {
    /// Activate the application.
    fn activate(&mut self, platform_data: HashMap<&str, Value<'_>>) -> zbus::Result<()>;

    /// Open the given URIs.
    fn open(
        &mut self,
        uris: Vec<&str>,
        platform_data: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<()>;

    /// Activate the given action.
    fn activate_action(
        &mut self,
        action_name: &str,
        parameter: Vec<&str>,
        platform_data: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<()>;
}

#[cfg(feature = "single-instance")]
#[interface(name = "org.freedesktop.DbusActivation")]
impl DbusActivation {
    async fn activate(&mut self, platform_data: HashMap<&str, Value<'_>>) {
        if let Some(tx) = &mut self.0 {
            let _ = tx
                .send(DbusActivationMessage {
                    activation_token: platform_data.get("activation-token").and_then(|t| match t {
                        Value::Str(t) => Some(t.to_string()),
                        _ => None,
                    }),
                    desktop_startup_id: platform_data.get("desktop-startup-id").and_then(
                        |t| match t {
                            Value::Str(t) => Some(t.to_string()),
                            _ => None,
                        },
                    ),
                    msg: DbusActivationDetails::Activate,
                })
                .await;
        }
    }

    async fn open(&mut self, uris: Vec<&str>, platform_data: HashMap<&str, Value<'_>>) {
        if let Some(tx) = &mut self.0 {
            let _ = tx
                .send(DbusActivationMessage {
                    activation_token: platform_data.get("activation-token").and_then(|t| match t {
                        Value::Str(t) => Some(t.to_string()),
                        _ => None,
                    }),
                    desktop_startup_id: platform_data.get("desktop-startup-id").and_then(
                        |t| match t {
                            Value::Str(t) => Some(t.to_string()),
                            _ => None,
                        },
                    ),
                    msg: DbusActivationDetails::Open {
                        url: uris.iter().filter_map(|u| Url::parse(u).ok()).collect(),
                    },
                })
                .await;
        }
    }

    async fn activate_action(
        &mut self,
        action_name: &str,
        parameter: Vec<&str>,
        platform_data: HashMap<&str, Value<'_>>,
    ) {
        if let Some(tx) = &mut self.0 {
            let _ = tx
                .send(DbusActivationMessage {
                    activation_token: platform_data.get("activation-token").and_then(|t| match t {
                        Value::Str(t) => Some(t.to_string()),
                        _ => None,
                    }),
                    desktop_startup_id: platform_data.get("desktop-startup-id").and_then(
                        |t| match t {
                            Value::Str(t) => Some(t.to_string()),
                            _ => None,
                        },
                    ),
                    msg: DbusActivationDetails::ActivateAction {
                        action: action_name.to_string(),
                        args: parameter
                            .iter()
                            .map(std::string::ToString::to_string)
                            .collect(),
                    },
                })
                .await;
        }
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

    if DbusActivationInterfaceProxyBlocking::builder(&conn)
        .destination(App::APP_ID)
        .ok()
        .and_then(|b| b.path(path).ok())
        .and_then(|b| b.destination(App::APP_ID).ok())
        .and_then(|b| b.build().ok())
        .is_some_and(|mut p| {
            match {
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
            } {
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
    fn context_drawer(&self) -> Option<ContextDrawer<Self::Message>> {
        None
    }

    /// Displays a dialog in the center of the application window when `Some`.
    fn dialog(&self) -> Option<Element<Self::Message>> {
        None
    }

    /// Displays a footer at the bottom of the application window when `Some`.
    fn footer(&self) -> Option<Element<Self::Message>> {
        None
    }

    /// Attaches elements to the start section of the header.
    fn header_start(&self) -> Vec<Element<Self::Message>> {
        Vec::new()
    }

    /// Attaches elements to the center of the header.
    fn header_center(&self) -> Vec<Element<Self::Message>> {
        Vec::new()
    }

    /// Attaches elements to the end section of the header.
    fn header_end(&self) -> Vec<Element<Self::Message>> {
        Vec::new()
    }

    /// Allows overriding the default nav bar widget.
    fn nav_bar(&self) -> Option<Element<Message<Self::Message>>> {
        if !self.core().nav_bar_active() {
            return None;
        }

        let nav_model = self.nav_model()?;

        let mut nav =
            crate::widget::nav_bar(nav_model, |id| Message::Cosmic(cosmic::Message::NavBar(id)))
                .on_context(|id| Message::Cosmic(cosmic::Message::NavBarContext(id)))
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
    fn nav_context_menu(&self, id: nav_bar::Id) -> Option<Vec<menu::Tree<Message<Self::Message>>>> {
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
    fn view(&self) -> Element<Self::Message>;

    /// Constructs views for other windows.
    fn view_window(&self, id: window::Id) -> Element<Self::Message> {
        panic!("no view for window {id:?}");
    }

    /// Overrides the default style for applications
    fn style(&self) -> Option<iced_runtime::Appearance> {
        None
    }

    /// Handles dbus activation messages
    #[cfg(feature = "single-instance")]
    fn dbus_activation(&mut self, msg: DbusActivationMessage) -> Task<Self::Message> {
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
    fn view_main(&self) -> Element<Message<Self::Message>>;
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
    fn view_main(&self) -> Element<Message<Self::Message>> {
        let core = self.core();
        let is_condensed = core.is_condensed();
        // TODO: More granularity might be needed for different resize border
        // and window border handling of maximized and tiled windows
        let sharp_corners = core.window.sharp_corners;
        let content_container = core.window.content_container;
        let nav_bar_active = core.nav_bar_active();
        let focused = core
            .focused_window()
            .is_some_and(|i| Some(i) == self.core().main_window_id());

        let main_content_padding = if content_container {
            if nav_bar_active {
                [0, 8, 8, 0]
            } else {
                [0, 8, 8, 8]
            }
        } else {
            [0, 0, 0, 0]
        };

        let content_row = crate::widget::row::with_children({
            let mut widgets = Vec::with_capacity(3);

            // Insert nav bar onto the left side of the window.
            let has_nav = if let Some(nav) = self
                .nav_bar()
                .map(|nav| id_container(nav, iced_core::id::Id::new("COSMIC_nav_bar")))
            {
                widgets.push(container(nav).padding([0, 8, 8, 8]).into());
                true
            } else {
                false
            };

            if self.nav_model().is_none() || core.show_content() {
                let main_content = self.view();

                //TODO: reduce duplication
                let context_width = core.context_width(has_nav);
                if core.window.context_is_overlay && core.window.show_context {
                    if let Some(context) = self.context_drawer() {
                        widgets.push(
                            crate::widget::context_drawer(
                                context.title,
                                context.header_actions,
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
                            .padding(if content_container {
                                [0, 8, 8, 0]
                            } else {
                                [0, 0, 0, 0]
                            })
                            .apply(Element::from)
                            .map(Message::App),
                        );
                    } else {
                        //TODO: container and padding are temporary, until
                        //the `resize_border` is moved to not cover window content
                        widgets.push(
                            container(main_content.map(Message::App))
                                .padding(main_content_padding)
                                .into(),
                        );
                    }
                } else {
                    //TODO: hide content when out of space
                    //TODO: container and padding are temporary, until
                    //the `resize_border` is moved to not cover window content
                    widgets.push(
                        container(main_content.map(Message::App))
                            .padding(main_content_padding)
                            .into(),
                    );
                    if let Some(context) = self.context_drawer() {
                        widgets.push(
                            crate::widget::ContextDrawer::new_inner(
                                context.title,
                                context.header_actions,
                                context.header,
                                context.footer,
                                context.content,
                                context.on_close,
                                context_width,
                            )
                            .apply(Element::from)
                            .map(Message::App)
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
                                [0, 8, 8, 0]
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
            .push_maybe(
                self.footer()
                    .map(|footer| container(footer.map(Message::App)).padding([0, 8, 8, 8])),
            );
        let content: Element<_> = if core.window.content_container {
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
        let window_corner_radius =
            crate::theme::active()
                .cosmic()
                .radius_s()
                .map(|x| if x < 4.0 { x } else { x + 4.0 });

        let view_column = crate::widget::column::with_capacity(2)
            .push_maybe(if core.window.show_headerbar {
                Some({
                    let mut header = crate::widget::header_bar()
                        .focused(focused)
                        .title(&core.window.header_title)
                        .on_drag(Message::Cosmic(cosmic::Message::Drag))
                        .on_right_click(Message::Cosmic(cosmic::Message::ShowWindowMenu))
                        .on_double_click(Message::Cosmic(cosmic::Message::Maximize));

                    if self.nav_model().is_some() {
                        let toggle = crate::widget::nav_bar_toggle()
                            .active(core.nav_bar_active())
                            .selected(focused)
                            .on_toggle(if is_condensed {
                                Message::Cosmic(cosmic::Message::ToggleNavBarCondensed)
                            } else {
                                Message::Cosmic(cosmic::Message::ToggleNavBar)
                            });

                        header = header.start(toggle);
                    }

                    if core.window.show_close {
                        header = header.on_close(Message::Cosmic(cosmic::Message::Close));
                    }

                    if core.window.show_maximize && crate::config::show_maximize() {
                        header = header.on_maximize(Message::Cosmic(cosmic::Message::Maximize));
                    }

                    if core.window.show_minimize && crate::config::show_minimize() {
                        header = header.on_minimize(Message::Cosmic(cosmic::Message::Minimize));
                    }

                    for element in self.header_start() {
                        header = header.start(element.map(Message::App));
                    }

                    for element in self.header_center() {
                        header = header.center(element.map(Message::App));
                    }

                    for element in self.header_end() {
                        header = header.end(element.map(Message::App));
                    }

                    if content_container {
                        header.apply(|w| id_container(w, iced_core::id::Id::new("COSMIC_header")))
                    } else {
                        // Needed to avoid header bar corner gaps for apps without a content container
                        header
                            .apply(container)
                            .class(crate::theme::Container::custom(move |theme| {
                                container::Style {
                                    background: Some(iced::Background::Color(
                                        theme.cosmic().background.base.into(),
                                    )),
                                    border: iced::Border {
                                        radius: [
                                            window_corner_radius[0] - 1.0,
                                            window_corner_radius[1] - 1.0,
                                            theme.cosmic().radius_0()[2],
                                            theme.cosmic().radius_0()[3],
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
            .padding(if sharp_corners { 0 } else { 1 })
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
                        width: if sharp_corners { 0.0 } else { 1.0 },
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
            popover = popover.popup(dialog.map(Message::App));
        }

        let view_element: Element<_> = popover.into();
        view_element.debug(core.debug)
    }
}

#[cfg(feature = "single-instance")]
fn single_instance_subscription<App: ApplicationExt>() -> Subscription<Message<App::Message>> {
    use iced_futures::futures::StreamExt;
    iced_futures::Subscription::run_with_id(
        TypeId::of::<DbusActivation>(),
        iced::stream::channel(10, move |mut output| async move {
            let mut single_instance: DbusActivation = DbusActivation::new();
            let mut rx = single_instance.rx();
            if let Ok(builder) = zbus::ConnectionBuilder::session() {
                let path: String = format!("/{}", App::APP_ID.replace('.', "/"));
                if let Ok(conn) = builder.build().await {
                    // XXX Setup done this way seems to be more reliable.
                    //
                    // the docs for serve_at seem to imply it will replace the
                    // existing interface at the requested path, but it doesn't
                    // seem to work that way all the time. The docs for
                    // object_server().at() imply it won't replace the existing
                    // interface.
                    //
                    // request_name is used either way, with the builder or
                    // with the connection, but it must be done after the
                    // object server is setup.
                    if conn.object_server().at(path, single_instance).await != Ok(true) {
                        tracing::error!("Failed to serve dbus");
                        std::process::exit(1);
                    }
                    if conn.request_name(App::APP_ID).await.is_err() {
                        tracing::error!("Failed to serve dbus");
                        std::process::exit(1);
                    }

                    #[cfg(feature = "smol")]
                    let handle = {
                        std::thread::spawn(move || {
                            let conn_clone = _conn.clone();

                            zbus::block_on(async move {
                                loop {
                                    conn_clone.executor().tick().await;
                                }
                            })
                        })
                    };
                    while let Some(mut msg) = rx.next().await {
                        if let Some(token) = msg.activation_token.take() {
                            if let Err(err) = output
                                .send(Message::Cosmic(cosmic::Message::Activate(token)))
                                .await
                            {
                                tracing::error!(?err, "Failed to send message");
                            }
                        }
                        if let Err(err) = output.send(Message::DbusActivation(msg)).await {
                            tracing::error!(?err, "Failed to send message");
                        }
                    }
                }
            } else {
                tracing::warn!("Failed to connect to dbus for single instance");
            }

            loop {
                iced::futures::pending!();
            }
        }),
    )
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

fn preload_fonts() {
    let mut font_system = iced::advanced::graphics::text::font_system()
        .write()
        .unwrap();

    EMBEDDED_FONTS
        .into_iter()
        .for_each(move |font| font_system.load_font(Cow::Borrowed(font)));
}
