// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Application API example

use cosmic::app::{about::About, Core, Settings, Task};
use cosmic::iced::widget::column;
use cosmic::iced_core::Size;
use cosmic::widget::{self, nav_bar};
use cosmic::{executor, iced, ApplicationExt, Element};

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let _ = tracing_log::LogTracer::init();

    let settings = Settings::default()
        .size(Size::new(1024., 768.));

    cosmic::app::run::<App>(settings, ())?;

    Ok(())
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    ToggleAbout,
    Cosmic(cosmic::app::cosmic::Message),
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    nav_model: nav_bar::Model,
    about: About,
}

/// Implement [`cosmic::Application`] to integrate with COSMIC.
impl cosmic::Application for App {
    /// Default async executor to use with the app.
    type Executor = executor::Default;

    /// Argument received [`cosmic::Application::new`].
    type Flags = ();

    /// Message type specific to our [`App`].
    type Message = Message;

    /// The unique application ID to supply to the window manager.
    const APP_ID: &'static str = "org.cosmic.AboutDemo";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Creates the application, and optionally emits command on initialize.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let nav_model = nav_bar::Model::default();

        let about = About::default()
            .set_application_name("About Demo")
            .set_application_icon(Self::APP_ID)
            .set_developer_name("System 76")
            .set_license_type("GPL-3.0")
            .set_website("https://system76.com/cosmic")
            .set_repository_url("https://github.com/pop-os/libcosmic")
            .set_support_url("https://github.com/pop-os/libcosmic/issues")
            .set_developers([("Michael Murphy".into(), "mmstick@system76.com".into())]);

        let mut app = App {
            core,
            nav_model,
            about,
        };

        let command = app.update_title();

        (app, command)
    }

    /// Allows COSMIC to integrate with your application's [`nav_bar::Model`].
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    /// Called when a navigation item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.nav_model.activate(id);
        self.update_title()
    }

    fn context_drawer(&self) -> Option<Element<Self::Message>> {
        if !self.core.window.show_context {
            return None;
        }

        if let Some(abuot_view) = self.about_view() {
            Some(abuot_view.map(Message::Cosmic))
        } else {
            None
        }
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::ToggleAbout => {
                self.core.window.show_context = !self.core.window.show_context;
                self.core.set_show_context(self.core.window.show_context)
            }
            Message::Cosmic(message) => {
                return cosmic::command::message(cosmic::app::Message::Cosmic(message))
            }
        }
        Task::none()
    }

    fn about(&self) -> Option<&About> {
        Some(&self.about)
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        let centered = cosmic::widget::container(
            column![widget::button::text("Show about").on_press(Message::ToggleAbout)]
                .width(iced::Length::Fill)
                .height(iced::Length::Shrink)
                .align_x(iced::alignment::Horizontal::Center),
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Shrink)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center);

        Element::from(centered)
    }
}

impl App
where
    Self: cosmic::Application,
{
    fn active_page_title(&mut self) -> &str {
        self.nav_model
            .text(self.nav_model.active())
            .unwrap_or("Unknown Page")
    }

    fn update_title(&mut self) -> Task<Message> {
        let header_title = self.active_page_title().to_owned();
        let window_title = format!("{header_title} — COSMIC AppDemo");
        self.set_header_title(header_title);
        if let Some(id) = self.core.main_window_id() {
            self.set_window_title(window_title, id)
        } else {
            Task::none()
        }
    }
}
