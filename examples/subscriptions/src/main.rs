// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Application API example

use cosmic::app::{Core, Settings, Task};
use cosmic::iced::Subscription;
use cosmic::{executor, prelude::*, widget};

/// Runs application with these settings
fn main() -> Result<(), Box<dyn std::error::Error>> {
    cosmic::app::run::<App>(Settings::default(), ())?;
    Ok(())
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
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
    const APP_ID: &'static str = "org.cosmic.TextInputsDemo";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Creates the application, and optionally emits task on initialize.
    fn init(core: Core, _input: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut app = App { core };

        let commands = Task::batch(vec![app.update_title()]);

        (app, commands)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::none()
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        Task::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<'_, Self::Message> {
        widget::row().into()
    }
}

impl App
where
    Self: cosmic::Application,
{
    fn update_title(&mut self) -> Task<Message> {
        let window_title = format!("COSMIC Subscriptions Demo");
        self.set_header_title(window_title.clone());
        self.set_window_title(window_title, self.core.main_window_id().unwrap())
    }
}
