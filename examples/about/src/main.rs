// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Application API example

use cosmic::app::context_drawer::{self, ContextDrawer};
use cosmic::app::{Core, Settings, Task};
use cosmic::executor;
use cosmic::iced::{alignment, Length, Size};
use cosmic::prelude::*;
use cosmic::widget::{self, about::About, nav_bar};

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default()
        .size(Size::new(1024., 768.));

    cosmic::app::run::<App>(settings, ())?;

    Ok(())
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    ToggleAbout,
    Open(String),
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    nav_model: nav_bar::Model,
    about: About,
    show_about: bool,
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

    /// Creates the application, and optionally emits task on initialize.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let nav_model = nav_bar::Model::default();

        let about = About::default()
            .name("About Demo")
            .icon(widget::icon::from_name(Self::APP_ID))
            .version("0.1.0")
            .author("System76")
            .license("GPL-3.0-only")
            .license_url("https://choosealicense.com/licenses/gpl-3.0/")
            .developers([("Michael Murphy", "info@system76.com")])
            .links([
                ("Website", "https://system76.com/cosmic"),
                ("Repository", "https://github.com/pop-os/libcosmic"),
                ("Support", "https://github.com/pop-os/libcosmic/issues"),
            ]);

        let mut app = App {
            core,
            nav_model,
            about,
            show_about: false,
        };

        app.set_header_title("COSMIC About Example".into());
        let command = app.set_window_title(
            "COSMIC About Example".into(),
            app.core.main_window_id().unwrap(),
        );

        (app, command)
    }

    /// Allows COSMIC to integrate with your application's [`nav_bar::Model`].
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    /// Called when a navigation item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Self::Message> {
        self.nav_model.activate(id);
        Task::none()
    }

    fn context_drawer(&self) -> Option<ContextDrawer<'_, Self::Message>> {
        self.show_about.then(|| {
            context_drawer::about(
                &self.about,
                |url| Message::Open(url.to_owned()),
                Message::ToggleAbout,
            )
        })
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::ToggleAbout => {
                self.set_show_context(!self.core.window.show_context);
                self.show_about = !self.show_about;
            }
            Message::Open(url) => match open::that_detached(url) {
                Ok(_) => (),
                Err(err) => eprintln!("Failed to open URL: {err}"),
            },
        }
        Task::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<'_, Self::Message> {
        let show_about_button = widget::button::text("Show about").on_press(Message::ToggleAbout);
        let centered = cosmic::widget::container(
            widget::column()
                .push(show_about_button)
                .width(Length::Fill)
                .height(Length::Shrink)
                .align_x(alignment::Horizontal::Center),
        )
        .width(Length::Fill)
        .height(Length::Shrink)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);

        Element::from(centered)
    }
}
