// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Calendar widget example

use chrono::NaiveDate;
use cosmic::app::{Core, Settings, Task};
use cosmic::widget::calendar::CalendarModel;
use cosmic::{executor, iced, ApplicationExt, Element};

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    cosmic::app::run::<App>(Settings::default(), ())?;

    Ok(())
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    DateSelected(NaiveDate),
    PrevMonth,
    NextMonth,
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    calendar_model: CalendarModel,
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
        let mut app = App {
            core,
            calendar_model: CalendarModel::now(),
        };

        let command = app.update_title();

        (app, command)
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::DateSelected(date) => {
                self.calendar_model.selected = date;
            }
            Message::PrevMonth => {
                self.calendar_model.show_prev_month();
            }
            Message::NextMonth => {
                self.calendar_model.show_next_month();
            }
        }

        println!("Date selected: {:?}", &self.calendar_model.selected);

        Task::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        let mut content = cosmic::widget::column().spacing(12);

        let calendar = cosmic::widget::calendar(
            &self.calendar_model,
            |date| Message::DateSelected(date),
            || Message::PrevMonth,
            || Message::NextMonth,
        );

        content = content.push(calendar);

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
        let title = "Calendar Demo";
        self.set_header_title(title.to_owned());

        self.core
            .main_window_id()
            .map(|window_id| self.set_window_title(title.into(), window_id))
            .unwrap_or_else(Task::none)
    }
}
