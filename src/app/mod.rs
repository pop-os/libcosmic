// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Build interactive cross-platform COSMIC applications.
//!
//! Check out our [application](https://github.com/pop-os/libcosmic/tree/master/examples/application)
//! example in our repository.

#[cfg(feature = "applet")]
pub mod applet;
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

pub use self::command::Command;
pub use self::core::Core;
pub use self::settings::Settings;
use crate::theme::THEME;
use crate::widget::nav_bar;
use crate::{Element, ElementExt};
use apply::Apply;
use iced::Subscription;
use iced::{window, Application as IcedApplication};
pub use message::Message;

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
    THEME.with(move |t| {
        let mut cosmic_theme = t.borrow_mut();
        cosmic_theme.set_theme(settings.theme.theme_type);
    });

    let mut iced = iced::Settings::with_flags((core, flags));

    iced.antialiasing = settings.antialiasing;
    iced.default_font = settings.default_font;
    iced.default_text_size = settings.default_text_size;
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

    /// Set the title of the main window.
    fn set_title(&mut self, title: String) -> iced::Command<Message<Self::Message>>;

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
    fn set_title(&mut self, title: String) -> iced::Command<Message<Self::Message>> {
        self.core_mut().title = title.clone();
        command::set_title(title)
    }

    #[cfg(not(feature = "wayland"))]
    fn set_title(&mut self, title: String) -> iced::Command<Message<Self::Message>> {
        self.core_mut().title = title.clone();
        iced::Command::none()
    }

    /// Creates the view for the main window.
    fn view_main<'a>(&'a self) -> Element<'a, Message<Self::Message>> {
        let core = self.core();
        let is_condensed = core.is_condensed();
        let mut main: Vec<Element<'a, Message<Self::Message>>> = Vec::with_capacity(2);

        if core.window.show_headerbar {
            main.push({
                let mut header = crate::widget::header_bar()
                    .title(self.title())
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

                Element::from(header).debug(core.debug)
            });
        }

        // The content element contains every element beneath the header.
        main.push(
            iced::widget::row({
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

                    widgets.push(main_content);
                }

                widgets
            })
            .spacing(8)
            .apply(iced::widget::container)
            .padding([0, 8, 8, 8])
            .width(iced::Length::Fill)
            .height(iced::Length::Fill)
            .style(crate::theme::Container::Background)
            .into(),
        );

        iced::widget::column(main).into()
    }
}
