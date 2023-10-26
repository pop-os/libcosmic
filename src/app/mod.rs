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

use std::any::TypeId;
use std::env::args;

pub use self::command::Command;
pub use self::core::Core;
pub use self::settings::Settings;
use crate::prelude::*;
use crate::theme::THEME;
use crate::widget::{context_drawer, nav_bar};
use apply::Apply;
use iced::Subscription;
use iced::{window, Application as IcedApplication};
use iced_futures::futures::channel::mpsc::{Receiver, Sender};
use iced_futures::futures::{SinkExt, StreamExt};
pub use message::Message;
use zbus::{dbus_interface, dbus_proxy};

/// Launch a COSMIC application with the given [`Settings`].
///
/// # Errors
///
/// Returns error on application failure.
pub fn run<App: Application>(settings: Settings, flags: App::Flags) -> iced::Result {
    if let Some(icon_theme) = settings.default_icon_theme {
        crate::icon_theme::set_default(icon_theme);
    }

    let mut core = Core::default();
    core.debug = settings.debug;
    core.set_scale_factor(settings.scale_factor);
    core.set_window_width(settings.size.0);
    core.set_window_height(settings.size.1);

    core.single_instance = settings.single_instance;

    THEME.with(move |t| {
        let mut cosmic_theme = t.borrow_mut();
        cosmic_theme.set_theme(settings.theme.theme_type);
    });

    let mut iced = iced::Settings::with_flags((core, flags));

    iced.antialiasing = settings.antialiasing;
    iced.default_font = settings.default_font;
    iced.default_text_size = settings.default_text_size;
    iced.exit_on_close_request = settings.exit_on_close;
    iced.id = Some(App::APP_ID.to_owned());

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
                size: settings.size,
                size_limits: settings.size_limits,
                title: None,
                transparent: settings.transparent,
                ..SctkWindowSettings::default()
            })
        };
    }

    #[cfg(not(feature = "wayland"))]
    {
        if let Some(_border_size) = settings.resizable {
            // iced.window.border_size = border_size as u32;
            iced.window.resizable = true;
        }
        iced.window.decorations = !settings.client_decorations;
        iced.window.size = settings.size;
        iced.window.transparent = settings.transparent;
    }

    cosmic::Cosmic::<App>::run(iced)
}

#[derive(Debug, Default)]
pub struct SingleInstance(Option<Sender<Vec<String>>>);

impl SingleInstance {
    #[must_use]
    pub fn new() -> Self {
        Self(None)
    }

    pub fn rx(&mut self) -> Receiver<Vec<String>> {
        let (tx, rx) = iced_futures::futures::channel::mpsc::channel(10);
        self.0 = Some(tx);
        rx
    }
}

#[dbus_interface(name = "com.system76.SingleInstance")]
impl SingleInstance {
    async fn activate(&mut self, args: Vec<String>) {
        if let Some(tx) = &mut self.0 {
            let _ = tx.send(args).await;
        }
    }
}

#[dbus_proxy(interface = "com.system76.SingleInstance")]
pub trait SingleInstanceClient {
    fn activate(&mut self, args: Vec<String>) -> zbus::Result<()>;
}
/// Launch a COSMIC application with the given [`Settings`].
/// If the application is already running, the arguments will be passed to the
/// running instance.
/// # Errors
/// Returns error on application failure.
pub fn run_single_instance<App: Application>(
    mut settings: Settings,
    flags: App::Flags,
) -> iced::Result {
    // try to claim the dbus name with our app id
    settings.single_instance = true;
    let path: String = format!("/{}", App::APP_ID.replace('.', "/"));

    let override_single = std::env::var("COSMIC_SINGLE_INSTANCE")
        .map(|v| &v.to_lowercase() == "false")
        .unwrap_or_default();

    if override_single {
        return run::<App>(settings, flags);
    }

    let Ok(conn) = zbus::blocking::Connection::session() else {
        tracing::warn!("Failed to connect to dbus");
        return run::<App>(settings, flags);
    };

    if SingleInstanceClientProxyBlocking::builder(&conn)
        .destination(App::APP_ID)
        .ok()
        .and_then(|b| b.path(path).ok())
        .and_then(|b| b.destination(App::APP_ID).ok())
        .and_then(|b| b.build().ok())
        .is_some_and(|mut p| {
            match {
                let args = args().collect::<Vec<String>>();
                p.activate(args)
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
        run::<App>(settings, flags)
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
    type Flags: Clone;

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

    /// Another instance of the application received these arguments.
    #[must_use]
    fn update_args(args: Vec<String>) -> Option<Self::Message> {
        None
    }

    /// Respond to an application-specific message.
    fn update(&mut self, message: Self::Message) -> iced::Command<Message<Self::Message>> {
        iced::Command::none()
    }

    /// Constructs the view for the main window.
    fn view(&self) -> Element<Self::Message>;

    /// Constructs views for other windows.
    fn view_window(&self, id: window::Id) -> Element<Self::Message> {
        panic!("no view for window {}", id.0);
    }

    /// Overrides the default style for applications
    fn style(&self) -> Option<<crate::Theme as iced_style::application::StyleSheet>::Style> {
        None
    }
}

/// Methods automatically derived for all types implementing [`Application`].
pub trait ApplicationExt: Application {
    /// Initiates a window drag.
    fn drag(&mut self) -> iced::Command<Message<Self::Message>>;

    /// Fullscreens the window.
    fn fullscreen(&mut self) -> iced::Command<Message<Self::Message>>;

    /// Minimizes the window.
    fn minimize(&mut self) -> iced::Command<Message<Self::Message>>;

    /// Get the title of the main window.
    fn title(&self) -> &str;

    /// Set the context drawer title.
    fn set_context_title(&mut self, title: String) {
        self.core_mut().set_context_title(title);
    }

    /// Set the header bar title.
    fn set_header_title(&mut self, title: String) {
        self.core_mut().set_header_title(title);
    }

    /// Set the title of the main window.
    fn set_window_title(&mut self, title: String) -> iced::Command<Message<Self::Message>>;

    /// View template for the main window.
    fn view_main(&self) -> Element<Message<Self::Message>>;
}

impl<App: Application> ApplicationExt for App {
    fn drag(&mut self) -> iced::Command<Message<Self::Message>> {
        command::drag()
    }

    fn fullscreen(&mut self) -> iced::Command<Message<Self::Message>> {
        command::fullscreen()
    }

    fn minimize(&mut self) -> iced::Command<Message<Self::Message>> {
        command::minimize()
    }

    fn title(&self) -> &str {
        &self.core().title
    }

    #[cfg(feature = "wayland")]
    fn set_window_title(&mut self, title: String) -> iced::Command<Message<Self::Message>> {
        self.core_mut().title = title.clone();
        command::set_title(title)
    }

    #[cfg(not(feature = "wayland"))]
    fn set_window_title(&mut self, title: String) -> iced::Command<Message<Self::Message>> {
        self.core_mut().title = title.clone();
        iced::Command::none()
    }

    /// Creates the view for the main window.
    fn view_main(&self) -> Element<Message<Self::Message>> {
        let core = self.core();
        let is_condensed = core.is_condensed();

        crate::widget::column::with_capacity(2)
            .push_maybe(if core.window.show_headerbar {
                Some({
                    let mut header = crate::widget::header_bar()
                        .title(&core.window.header_title)
                        .on_drag(Message::Cosmic(cosmic::Message::Drag))
                        .on_close(Message::Cosmic(cosmic::Message::Close));

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

                    if core.window.show_maximize {
                        header = header.on_maximize(Message::Cosmic(cosmic::Message::Maximize));
                    }

                    if core.window.show_minimize {
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
            .push(
                crate::widget::row::with_children({
                    let mut widgets = Vec::with_capacity(2);

                    // Insert nav bar onto the left side of the window.
                    if core.nav_bar_active() {
                        if let Some(nav_model) = self.nav_model() {
                            let mut nav = crate::widget::nav_bar(nav_model, |entity| {
                                Message::Cosmic(cosmic::Message::NavBar(entity))
                            });

                            if !is_condensed {
                                nav = nav.max_width(300);
                            }

                            widgets.push(nav.apply(Element::from).debug(core.debug));
                        }
                    }

                    if self.nav_model().is_none() || core.show_content() {
                        let main_content = self.view().debug(core.debug).map(Message::App);

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
                .spacing(8)
                .apply(crate::widget::container)
                .padding([0, 8, 8, 8])
                .width(iced::Length::Fill)
                .height(iced::Length::Fill)
                .style(crate::theme::Container::Background),
            )
            .into()
    }
}

fn single_instance_subscription<App: ApplicationExt>() -> Subscription<App::Message> {
    iced::subscription::channel(
        TypeId::of::<SingleInstance>(),
        10,
        |mut output| async move {
            let mut single_instance: SingleInstance = SingleInstance::new();
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
                    while let Some(msg) = rx.next().await {
                        if let Some(msg) = App::update_args(msg) {
                            let _ = output.send(msg).await;
                        } else {
                            tracing::warn!("Failed to parse arguments from another instance");
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
