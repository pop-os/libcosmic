// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Build interactive cross-platform COSMIC applications.
//!
//! Check out our [application](https://github.com/pop-os/libcosmic/tree/master/examples/application)
//! example in our repository.

pub mod command;
mod core;
pub mod cosmic;
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
}

pub use self::command::Command;
pub use self::core::Core;
pub use self::settings::Settings;
use crate::config::CosmicTk;
use crate::prelude::*;
use crate::theme::THEME;
use crate::widget::{context_drawer, menu, nav_bar, popover};
use apply::Apply;
use iced::Subscription;
#[cfg(all(feature = "winit", feature = "multi-window"))]
use iced::{multi_window::Application as IcedApplication, window};
#[cfg(any(not(feature = "winit"), not(feature = "multi-window")))]
use iced::{window, Application as IcedApplication};
use iced_core::mouse;
pub use message::Message;
use url::Url;
#[cfg(feature = "single-instance")]
use {
    iced_futures::futures::channel::mpsc::{Receiver, Sender},
    iced_futures::futures::SinkExt,
    std::any::TypeId,
    std::collections::HashMap,
    zbus::{dbus_interface, dbus_proxy, zvariant::Value},
};

pub(crate) fn iced_settings<App: Application>(
    settings: Settings,
    flags: App::Flags,
) -> iced::Settings<(Core, App::Flags)> {
    let mut core = Core::default();
    core.debug = settings.debug;
    core.icon_theme_override = settings.default_icon_theme.is_some();
    core.set_scale_factor(settings.scale_factor);
    core.set_window_width(settings.size.width as u32);
    core.set_window_height(settings.size.height as u32);

    if let Some(icon_theme) = settings.default_icon_theme {
        crate::icon_theme::set_default(icon_theme);
    } else {
        crate::icon_theme::set_default(core.toolkit_config.icon_theme.clone());
    }

    THEME.with(move |t| {
        let mut cosmic_theme = t.borrow_mut();
        cosmic_theme.set_theme(settings.theme.theme_type);
    });

    let mut iced = iced::Settings::with_flags((core, flags));

    iced.antialiasing = settings.antialiasing;
    iced.default_font = settings.default_font;
    iced.default_text_size = iced::Pixels(settings.default_text_size);
    iced.exit_on_close_request = settings.exit_on_close;
    iced.id = Some(App::APP_ID.to_owned());
    #[cfg(all(not(feature = "wayland"), target_os = "linux"))]
    {
        iced.window.platform_specific.application_id = App::APP_ID.to_string();
    }

    #[cfg(feature = "wayland")]
    {
        use iced::wayland::actions::window::SctkWindowSettings;
        use iced_sctk::settings::InitialSurface;
        iced.initial_surface = if settings.no_main_window {
            InitialSurface::None
        } else {
            InitialSurface::XdgWindow(SctkWindowSettings {
                app_id: Some(App::APP_ID.to_owned()),
                autosize: settings.autosize,
                client_decorations: settings.client_decorations,
                resizable: settings.resizable,
                size: (settings.size.width as u32, settings.size.height as u32).into(),
                size_limits: settings.size_limits,
                title: None,
                transparent: settings.transparent,
                xdg_activation_token: std::env::var("XDG_ACTIVATION_TOKEN").ok(),
                ..SctkWindowSettings::default()
            })
        };
    }

    #[cfg(not(feature = "wayland"))]
    {
        if let Some(border_size) = settings.resizable {
            iced.window.resize_border = border_size as u32;
            iced.window.resizable = true;
        }
        iced.window.decorations = !settings.client_decorations;
        iced.window.size = settings.size;
        let min_size = settings.size_limits.min();
        if min_size != iced::Size::ZERO {
            iced.window.min_size = Some(min_size);
        }
        let max_size = settings.size_limits.max();
        if max_size != iced::Size::INFINITY {
            iced.window.max_size = Some(max_size);
        }
        iced.window.transparent = settings.transparent;
    }

    iced
}

/// Launch a COSMIC application with the given [`Settings`].
///
/// # Errors
///
/// Returns error on application failure.
pub fn run<App: Application>(settings: Settings, flags: App::Flags) -> iced::Result {
    #[cfg(feature = "wgpu")]
    wgpu_power_pref();

    let settings = iced_settings::<App>(settings, flags);

    cosmic::Cosmic::<App>::run(settings)
}

/// Default to rendering the application with the low power GPU preference.
#[cfg(feature = "wgpu")]
fn wgpu_power_pref() {
    // Ignore if requested to run on NVIDIA GPU
    if std::env::var("__NV_PRIME_RENDER_OFFLOAD").ok().as_deref() == Some("1") {
        return;
    }

    const VAR: &str = "WGPU_POWER_PREF";
    if std::env::var(VAR).is_err() {
        std::env::set_var(VAR, "low");
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
#[dbus_proxy(interface = "org.freedesktop.DbusActivation", assume_defaults = true)]
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
#[dbus_interface(name = "org.freedesktop.DbusActivation")]
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
    App::Flags: CosmicFlags + Clone,
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
        let mut settings = iced_settings::<App>(settings, flags);
        settings.flags.0.single_instance = true;
        cosmic::Cosmic::<App>::run(settings)
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

    /// Creates the application, and optionally emits command on initialize.
    fn init(core: Core, flags: Self::Flags) -> (Self, iced::Command<Message<Self::Message>>);

    /// Displays a context drawer on the side of the application window when `Some`.
    fn context_drawer(&self) -> Option<Element<Self::Message>> {
        None
    }

    /// Displays a dialog in the center of the application window when `Some`.
    fn dialog(&self) -> Option<Element<Self::Message>> {
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

    /// Get the main [`window::Id`], which is [`window::Id::MAIN`] by default
    fn main_window_id(&self) -> window::Id {
        window::Id::MAIN
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

    /// Called before closing the application.
    fn on_app_exit(&mut self) {}

    /// Called when a window requests to be closed.
    fn on_close_requested(&self, id: window::Id) -> Option<Self::Message> {
        None
    }

    /// Called when the escape key is pressed.
    fn on_escape(&mut self) -> iced::Command<Message<Self::Message>> {
        iced::Command::none()
    }

    /// Called when a navigation item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> iced::Command<Message<Self::Message>> {
        iced::Command::none()
    }

    /// Called when a context menu is requested for a navigation item.
    fn on_nav_context(&mut self, id: nav_bar::Id) -> iced::Command<Message<Self::Message>> {
        iced::Command::none()
    }

    /// Called when the search function is requested.
    fn on_search(&mut self) -> iced::Command<Message<Self::Message>> {
        iced::Command::none()
    }

    /// Called when a window is resized.
    fn on_window_resize(&mut self, id: window::Id, width: u32, height: u32) {}

    /// Event sources that are to be listened to.
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Respond to an application-specific message.
    fn update(&mut self, message: Self::Message) -> iced::Command<Message<Self::Message>> {
        iced::Command::none()
    }

    /// Respond to a system theme change
    fn system_theme_update(
        &mut self,
        keys: &[&'static str],
        new_theme: &cosmic_theme::Theme,
    ) -> iced::Command<Message<Self::Message>> {
        iced::Command::none()
    }

    /// Respond to a system theme mode change
    fn system_theme_mode_update(
        &mut self,
        keys: &[&'static str],
        new_theme: &cosmic_theme::ThemeMode,
    ) -> iced::Command<Message<Self::Message>> {
        iced::Command::none()
    }

    /// Constructs the view for the main window.
    fn view(&self) -> Element<Self::Message>;

    /// Constructs views for other windows.
    fn view_window(&self, id: window::Id) -> Element<Self::Message> {
        panic!("no view for window {id:?}");
    }

    /// Overrides the default style for applications
    fn style(&self) -> Option<<crate::Theme as iced_style::application::StyleSheet>::Style> {
        None
    }

    /// Handles dbus activation messages
    #[cfg(feature = "single-instance")]
    fn dbus_activation(
        &mut self,
        msg: DbusActivationMessage,
    ) -> iced::Command<Message<Self::Message>> {
        iced::Command::none()
    }
}

/// Methods automatically derived for all types implementing [`Application`].
pub trait ApplicationExt: Application {
    /// Initiates a window drag.
    fn drag(&mut self) -> iced::Command<Message<Self::Message>>;

    /// Maximizes the window.
    fn maximize(&mut self) -> iced::Command<Message<Self::Message>>;

    /// Minimizes the window.
    fn minimize(&mut self) -> iced::Command<Message<Self::Message>>;
    /// Get the title of the main window.

    #[cfg(not(any(feature = "multi-window", feature = "wayland")))]
    fn title(&self) -> &str;

    #[cfg(any(feature = "multi-window", feature = "wayland"))]
    /// Get the title of a window.
    fn title(&self, id: window::Id) -> &str;

    /// Set the context drawer title.
    fn set_context_title(&mut self, title: String) {
        self.core_mut().set_context_title(title);
    }

    /// Set the header bar title.
    fn set_header_title(&mut self, title: String) {
        self.core_mut().set_header_title(title);
    }

    #[cfg(not(any(feature = "multi-window", feature = "wayland")))]
    /// Set the title of the main window.
    fn set_window_title(&mut self, title: String) -> iced::Command<Message<Self::Message>>;

    #[cfg(any(feature = "multi-window", feature = "wayland"))]
    /// Set the title of a window.
    fn set_window_title(
        &mut self,
        title: String,
        id: window::Id,
    ) -> iced::Command<Message<Self::Message>>;

    /// View template for the main window.
    fn view_main(&self) -> Element<Message<Self::Message>>;
}

impl<App: Application> ApplicationExt for App {
    fn drag(&mut self) -> iced::Command<Message<Self::Message>> {
        command::drag(Some(self.main_window_id()))
    }

    fn maximize(&mut self) -> iced::Command<Message<Self::Message>> {
        command::maximize(Some(self.main_window_id()), true)
    }

    fn minimize(&mut self) -> iced::Command<Message<Self::Message>> {
        command::minimize(Some(self.main_window_id()))
    }

    #[cfg(any(feature = "multi-window", feature = "wayland"))]
    fn title(&self, id: window::Id) -> &str {
        self.core().title.get(&id).map_or("", |s| s.as_str())
    }

    #[cfg(not(any(feature = "multi-window", feature = "wayland")))]
    fn title(&self) -> &str {
        self.core()
            .title
            .get(&self.main_window_id())
            .map_or("", |s| s.as_str())
    }

    #[cfg(any(feature = "multi-window", feature = "wayland"))]
    fn set_window_title(
        &mut self,
        title: String,
        id: window::Id,
    ) -> iced::Command<Message<Self::Message>> {
        self.core_mut().title.insert(id, title.clone());
        command::set_title(Some(id), title)
    }

    #[cfg(not(any(feature = "multi-window", feature = "wayland")))]
    fn set_window_title(&mut self, title: String) -> iced::Command<Message<Self::Message>> {
        let id = self.main_window_id();

        self.core_mut().title.insert(id, title.clone());
        iced::Command::none()
    }

    /// Creates the view for the main window.
    fn view_main(&self) -> Element<Message<Self::Message>> {
        let core = self.core();
        let is_condensed = core.is_condensed();
        let focused = core
            .focused_window()
            .map(|i| i == self.main_window_id())
            .unwrap_or_default();

        let content_row = crate::widget::row::with_children({
            let mut widgets = Vec::with_capacity(2);

            // Insert nav bar onto the left side of the window.
            if let Some(nav) = self.nav_bar() {
                widgets.push(nav);
            }

            if self.nav_model().is_none() || core.show_content() {
                let main_content = self.view().map(Message::App);

                widgets.push(if let Some(context) = self.context_drawer() {
                    context_drawer(
                        &core.window.context_title,
                        Message::Cosmic(cosmic::Message::ContextDrawer(false)),
                        main_content,
                        context.map(Message::App),
                    )
                    .into()
                } else {
                    main_content
                });
            }

            widgets
        })
        .spacing(8);
        let content: Element<_> = if core.window.content_container {
            content_row
                .apply(crate::widget::container)
                .padding([0, 8, 8, 8])
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .style(crate::theme::Container::WindowBackground)
                .into()
        } else {
            content_row.into()
        };

        let view_column = crate::widget::column::with_capacity(2)
            .push_maybe(if core.window.show_headerbar {
                Some({
                    let mut header = crate::widget::header_bar()
                        .focused(focused)
                        .title(&core.window.header_title)
                        .on_drag(Message::Cosmic(cosmic::Message::Drag))
                        .on_close(Message::Cosmic(cosmic::Message::Close))
                        .on_right_click(Message::Cosmic(cosmic::Message::ShowWindowMenu));

                    if self.nav_model().is_some() {
                        let toggle = crate::widget::nav_bar_toggle()
                            .active(core.nav_bar_active())
                            .on_toggle(if is_condensed {
                                Message::Cosmic(cosmic::Message::ToggleNavBarCondensed)
                            } else {
                                Message::Cosmic(cosmic::Message::ToggleNavBar)
                            });

                        header = header.start(toggle);
                    }

                    if core.window.show_maximize && core.toolkit_config.show_maximize {
                        header = header.on_maximize(Message::Cosmic(cosmic::Message::Maximize));
                    }

                    if core.window.show_minimize && core.toolkit_config.show_minimize {
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

                    header
                })
            } else {
                None
            })
            // The content element contains every element beneath the header.
            .push(content);

        // Show any current dialog on top and centered over the view content
        // We have to use a popover even without a dialog to keep the tree from changing
        let mut popover = popover(view_column).modal(true);
        if let Some(dialog) = self.dialog() {
            popover = popover.popup(dialog.map(Message::App));
        }

        let view_element: Element<_> = popover.into();
        view_element.debug(core.debug)
    }
}

#[cfg(feature = "single-instance")]
fn single_instance_subscription<App: ApplicationExt>() -> Subscription<Message<App::Message>> {
    use iced_futures::futures::StreamExt;

    iced::subscription::channel(
        TypeId::of::<DbusActivation>(),
        10,
        move |mut output| async move {
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
        },
    )
}
