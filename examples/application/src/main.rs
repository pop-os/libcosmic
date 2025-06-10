// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Application API example

use std::collections::HashMap;
use std::sync::LazyLock;

use cosmic::app::{Core, Settings, Task};
use cosmic::iced::alignment::{Horizontal, Vertical};
use cosmic::iced::widget::column;
use cosmic::iced::Length;
use cosmic::iced_core::Size;
use cosmic::widget::icon::{from_name, Handle};
use cosmic::widget::menu::KeyBind;
use cosmic::widget::{button, text};
use cosmic::widget::{
    container,
    menu::menu_button,
    menu::{self, action::MenuAction},
    nav_bar, responsive,
};
use cosmic::{executor, iced, ApplicationExt, Element};

static MENU_ID: LazyLock<iced::id::Id> = LazyLock::new(|| iced::id::Id::new("menu_id"));

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Hi,
    Hi2,
    Hi3,
}

impl MenuAction for Action {
    type Message = Message;

    fn message(&self) -> Message {
        match self {
            Action::Hi => Message::Hi,
            Action::Hi2 => Message::Hi2,
            Action::Hi3 => Message::Hi3,
        }
    }
}

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tracing_subscriber::fmt::init();
    // let _ = tracing_log::LogTracer::init();

    let input = vec![
        (Page::Page1, "🖖 Hello from libcosmic.".into()),
        (Page::Page2, "🌟 This is an example application.".into()),
        (Page::Page3, "🚧 The libcosmic API is not stable yet.".into()),
        (Page::Page4, "🚀 Copy the source code and experiment today!".into()),
    ];

    let settings = Settings::default()
        .size(Size::new(1024., 768.));

    cosmic::app::run::<App>(settings, input)?;

    Ok(())
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    Input1(String),
    Input2(String),
    Ignore,
    ToggleHide,
    Surface(cosmic::surface::Action),
    Hi,
    Hi2,
    Hi3,
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    nav_model: nav_bar::Model,
    input_1: String,
    input_2: String,
    hidden: bool,
    keybinds: HashMap<KeyBind, Action>,
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

    /// Creates the application, and optionally emits task on initialize.
    fn init(core: Core, input: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut nav_model = nav_bar::Model::default();

        for (title, content) in input {
            nav_model.insert().text(title.as_str()).data(content);
        }

        nav_model.activate_position(0);

        let mut app = App {
            core,
            nav_model,
            input_1: String::new(),
            input_2: String::new(),
            hidden: true,
            keybinds: HashMap::new(),
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

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::Input1(v) => {
                self.input_1 = v;
            }
            Message::Input2(v) => {
                self.input_2 = v;
            }
            Message::Ignore => {}
            Message::ToggleHide => {
                self.hidden = !self.hidden;
            }
            Message::Surface(a) => {
                return cosmic::task::message(cosmic::Action::Cosmic(
                    cosmic::app::Action::Surface(a),
                ));
            }
            Message::Hi => {
                dbg!("hi");
            }
            Message::Hi2 => {
                dbg!("hi 2");
            }
            Message::Hi3 => {
                dbg!("hi 3");
            }
        }
        Task::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        let page_content = self
            .nav_model
            .active_data::<String>()
            .map_or("No page selected", String::as_str);

        let text = cosmic::widget::text(page_content);

        let centered = cosmic::widget::container(
            column![
                text,
                cosmic::widget::text_input::text_input("", &self.input_1)
                    .on_input(Message::Input1)
                    .on_clear(Message::Ignore),
                cosmic::widget::text_input::secure_input(
                    "",
                    &self.input_1,
                    Some(Message::ToggleHide),
                    self.hidden
                )
                .on_input(Message::Input1),
                cosmic::widget::text_input::text_input("", &self.input_1).on_input(Message::Input1),
                cosmic::widget::text_input::search_input("", &self.input_2)
                    .on_input(Message::Input2)
                    .on_clear(Message::Ignore),
            ]
            .spacing(cosmic::theme::spacing().space_s)
            .width(iced::Length::Fill)
            .height(iced::Length::Shrink)
            .align_x(iced::Alignment::Center),
        )
        .width(iced::Length::Fill)
        .height(iced::Length::Shrink)
        .align_x(iced::Alignment::Center)
        .align_y(iced::Alignment::Center);

        Element::from(centered)
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![cosmic::widget::responsive_menu_bar().into_element(
            self.core(),
            &self.keybinds,
            MENU_ID.clone(),
            Message::Surface,
            vec![
                (
                    "hi 1".into(),
                    vec![
                        menu::Item::Button("hi 12", None, Action::Hi),
                        menu::Item::Button("hi 13", None, Action::Hi2),
                    ],
                ),
                (
                    "hi 2".into(),
                    vec![
                        menu::Item::Button("hi 21", None, Action::Hi),
                        menu::Item::Button("hi 22", None, Action::Hi2),
                        menu::Item::Folder(
                            "nest 3 2 >".into(),
                            vec![
                                menu::Item::Button("21", None, Action::Hi),
                                menu::Item::Button("242", None, Action::Hi2),
                                menu::Item::Button("2443", None, Action::Hi3),
                                menu::Item::Folder(
                                    "nest 4 2 >".into(),
                                    vec![
                                        menu::Item::Button("243", None, Action::Hi2),
                                        menu::Item::Button("2444", None, Action::Hi),
                                    ],
                                ),
                            ],
                        ),
                    ],
                ),
                (
                    "hi 3".into(),
                    vec![
                        menu::Item::Button("hi 31", None, Action::Hi),
                        menu::Item::Button("hi 332", None, Action::Hi2),
                        menu::Item::Button("hi 3333", None, Action::Hi3),
                        menu::Item::Button("hi 33334", None, Action::Hi3),
                        menu::Item::Button("hi 333335", None, Action::Hi3),
                        menu::Item::Button("hi 3333336", None, Action::Hi3),
                    ],
                ),
                (
                    "hiiiiiiiiiiiiiiiiiii 4".into(),
                    vec![
                        menu::Item::Button("hi 4", None, Action::Hi),
                        menu::Item::Button("hi 44", None, Action::Hi2),
                        menu::Item::Button("hi 444", None, Action::Hi3),
                        menu::Item::Folder(
                            "nest 4 >".into(),
                            vec![
                                menu::Item::Button("hi 41", None, Action::Hi),
                                menu::Item::Button("hi 442", None, Action::Hi2),
                                menu::Item::Folder(
                                    "nest 3 4 >".into(),
                                    vec![
                                        menu::Item::Button("hi 443", None, Action::Hi2),
                                        menu::Item::Button("hi 4444", None, Action::Hi),
                                        menu::Item::Button("hi 44444", None, Action::Hi3),
                                        menu::Item::Button("hi 444445", None, Action::Hi3),
                                        menu::Item::Button("hi 4444446", None, Action::Hi3),
                                        menu::Item::Button("hi 44444447", None, Action::Hi3),
                                    ],
                                ),
                            ],
                        ),
                    ],
                ),
            ],
        )]
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
