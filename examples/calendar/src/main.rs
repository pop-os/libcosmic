// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Calendar widget example

use chrono::{Datelike, Days, Local, Months, NaiveDate};
use cosmic::app::{Command, Core, Settings};
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
    PrevMonth,
    NextMonth,
    DaySelected(u32),
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    date_selected: NaiveDate,
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

    /// Creates the application, and optionally emits command on initialize.
    fn init(core: Core, _input: Self::Flags) -> (Self, Command<Self::Message>) {
        let now = Local::now();

        let mut app = App {
            core,
            date_selected: NaiveDate::from(now.naive_local()),
        };

        let command = app.update_title();

        (app, command)
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::DaySelected(day) => {
                let current = self.date_selected.day();

                let new_date = if current < day {
                    self.date_selected
                        .checked_add_days(Days::new((day - current) as u64))
                } else if current > day {
                    self.date_selected
                        .checked_sub_days(Days::new((current - day) as u64))
                } else {
                    None
                };

                if let Some(new) = new_date {
                    self.date_selected = new;
                }
            }
            Message::PrevMonth => {
                self.date_selected = self
                    .date_selected
                    .checked_sub_months(Months::new(1))
                    .expect("valid naivedate");
            }
            Message::NextMonth => {
                self.date_selected = self
                    .date_selected
                    .checked_add_months(Months::new(1))
                    .expect("valid naivedate");
            }
        }

        Command::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        let mut content = cosmic::widget::column().spacing(12);

        let calendar = cosmic::widget::calendar(
            &self.date_selected,
            Message::PrevMonth,
            Message::NextMonth,
            |day| Message::DaySelected(day),
        );

        content = content.push(cosmic::widget::container(calendar).width(350));

        let centered = cosmic::widget::container(content)
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
    fn update_title(&mut self) -> Command<Message> {
        self.set_header_title(String::from("Calendar Demo"));
        self.set_window_title(String::from("Calendar Demo"))
    }
}
