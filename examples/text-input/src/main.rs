// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Application API example

use cosmic::app::{Core, Settings, Task};
use cosmic::{executor, iced, ApplicationExt, Element};

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let _ = tracing_log::LogTracer::init();

    cosmic::app::run::<App>(Settings::default(), ())?;

    Ok(())
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    EditMode(bool),
    Input(String),
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    input: String,
    editing: bool,
    search_id: cosmic::widget::Id,
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
        let mut app = App {
            core,
            editing: false,
            input: String::from("Test"),
            search_id: cosmic::widget::Id::unique(),
        };

        let commands = Task::batch(vec![
            cosmic::widget::text_input::focus(app.search_id.clone()),
            app.update_title(),
        ]);

        (app, commands)
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Input(text) => {
                self.input = text;
            }

            Message::EditMode(editing) => {
                self.editing = editing;
            }
        }

        Task::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        let editable = cosmic::widget::editable_input(
            "Input text here",
            &self.input,
            self.editing,
            Message::EditMode,
        )
        .on_input(Message::Input)
        .id(self.search_id.clone());

        let inline = cosmic::widget::inline_input("", &self.input).on_input(Message::Input);

        let column = cosmic::widget::column().push(editable).push(inline);

        let centered = cosmic::widget::container(column.width(200))
            .width(iced::Length::Fill)
            .height(iced::Length::Shrink)
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Center);

        Element::from(centered)
    }
}

impl App
where
    Self: cosmic::Application,
{
    fn update_title(&mut self) -> Task<Message> {
        let window_title = format!("COSMIC TextInputs Demo");
        self.set_header_title(window_title.clone());
        self.core
            .main_window_id()
            .map(|window_id| self.set_window_title(window_title, window_id))
            .unwrap_or_else(Task::none)
    }
}
