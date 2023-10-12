// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Application API example

use cosmic::app::{Command, Core, Settings};
use cosmic::widget::nav_bar;
use cosmic::{executor, iced, ApplicationExt, Element};

#[derive(Clone, Copy)]
pub enum Page {
    Page1,
    Page2,
    Page3,
    Page4,
}

impl Page {
    const fn as_str(self) -> &'static str {
        match self {
            Page::Page1 => "Page 1",
            Page::Page2 => "Page 2",
            Page::Page3 => "Page 3",
            Page::Page4 => "Page 4",
        }
    }
}

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = vec![
        (Page::Page1, "🖖 Hello from libcosmic.".into()),
        (Page::Page2, "🌟 This is an example application.".into()),
        (Page::Page3, "🚧 The libcosmic API is not stable yet.".into()),
        (Page::Page4, "🚀 Copy the source code and experiment today!".into()),
    ];

    let settings = Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .debug(false)
        .default_icon_theme("Pop")
        .default_text_size(16.0)
        .scale_factor(1.0)
        .size((1024, 768))
        .theme(cosmic::Theme::dark());

    cosmic::app::run::<App>(settings, input)?;

    Ok(())
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    nav_model: nav_bar::Model,
}

/// Implement [`cosmic::Application`] to integrate with COSMIC.
impl cosmic::Application for App {
    /// Default async executor to use with the app.
    type Executor = executor::Default;

    /// Argument received [`cosmic::Application::new`].
    type Flags = Vec<(Page, String)>;

    /// Message type specific to our [`App`].
    type Message = Message;

    /// The unique application ID to supply to the window manager.
    const APP_ID: &'static str = "org.cosmic.AppDemo";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Creates the application, and optionally emits command on initialize.
    fn init(core: Core, input: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut nav_model = nav_bar::Model::default();

        for (title, content) in input {
            nav_model.insert().text(title.as_str()).data(content);
        }

        nav_model.activate_position(0);

        let mut app = App { core, nav_model };

        let command = app.update_title();

        (app, command)
    }

    /// Allows COSMIC to integrate with your application's [`nav_bar::Model`].
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    /// Called when a navigation item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Command<Self::Message> {
        self.nav_model.activate(id);
        self.update_title()
    }

    /// Handle application events here.
    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        let page_content = self
            .nav_model
            .active_data::<String>()
            .map(String::as_str)
            .unwrap_or("No page selected");

        let text = cosmic::widget::text(page_content);

        let centered = cosmic::widget::container(text)
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

    fn update_title(&mut self) -> Command<Message> {
        let header_title = self.active_page_title().to_owned();
        let window_title = format!("{header_title} — COSMIC AppDemo");
        self.set_header_title(header_title);
        self.set_window_title(window_title)
    }
}
