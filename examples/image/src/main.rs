// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Application API example

use cosmic::app::{Core, Settings, Task};
use cosmic::{executor, iced, widget, ApplicationExt, Element};

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    cosmic::app::run::<App>(Settings::default(), ())?;

    Ok(())
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    SetAlternateHandle,
}

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
    const APP_ID: &'static str = "org.cosmic.AppDemo";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Creates the application, and optionally emits task on initialize.
    fn init(core: Core, _input: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut app = App { core };

        let command = app.update_title();

        (app, command)
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::SetAlternateHandle => {
                let path = "/usr/share/backgrounds/pop/nick-nazzaro-ice-cave.png";
                Task::batch(vec![cosmic::widget::image::set_handle(
                    widget::Id::new(path.clone()),
                    path,
                )])
            }
        }
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        let mut content = cosmic::widget::column().spacing(12);
        let path = "/usr/share/backgrounds/pop/kait-herzog-8242.jpg";
        content = content.push(
            cosmic::widget::button::custom(
                cosmic::widget::image::image(path).id(widget::Id::new(path.clone())),
            )
            .width(300.0),
        );

        content = content.push(
            widget::button::text("Set alternate handle").on_press(Message::SetAlternateHandle),
        );

        let centered = cosmic::widget::container(content)
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
        self.set_header_title(String::from("Image Button Demo"));
        self.set_window_title(String::from("Image Button Demo"))
    }
}
