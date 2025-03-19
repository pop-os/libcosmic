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
}

impl MenuAction for Action {
    type Message = Message;

    fn message(&self) -> Message {
        Message::Hi
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
            Message::Surface(_) => {
                // unimplemented!()
            }
            Message::Hi => {
                dbg!("hi");
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
        use cosmic::widget::menu::Tree;
        #[cfg(not(feature = "wayland"))]
        {
            vec![cosmic::widget::menu::bar(vec![
                Tree::with_children(
                    menu::root("hiiiiiiiiiiiiiiiiiii 1"),
                    menu::items(
                        &self.keybinds,
                        vec![menu::Item::Button("hi", None, Action::Hi)],
                    ),
                ),
                Tree::with_children(
                    menu::root("hiiiiiiiiiiiiiiiiii 2"),
                    menu::items(
                        &self.keybinds,
                        vec![menu::Item::Button("hi 2", None, Action::Hi)],
                    ),
                ),
                Tree::with_children(
                    menu::root("hiiiiiiiiiiiiiiiiiiiii 3"),
                    menu::items(
                        &self.keybinds,
                        vec![
                            menu::Item::Button("hi 3", None, Action::Hi),
                            menu::Item::Button("hi 3 #2", None, Action::Hi),
                        ],
                    ),
                ),
                Tree::with_children(
                    menu::root("hi 3"),
                    menu::items(
                        &self.keybinds,
                        vec![
                            menu::Item::Button("hi 3", None, Action::Hi),
                            menu::Item::Button("hi 3 #2", None, Action::Hi),
                            menu::Item::Button("hi 3 #3", None, Action::Hi),
                        ],
                    ),
                ),
                Tree::with_children(
                    menu::root("hi 4"),
                    menu::items(
                        &self.keybinds,
                        vec![
                            menu::Item::Folder(
                                "hi 41 extra root",
                                vec![menu::Item::Button("hi 3", None, Action::Hi)],
                            ),
                            menu::Item::Button("hi 42", None, Action::Hi),
                            menu::Item::Button("hi 43", None, Action::Hi),
                            menu::Item::Button("hi 44", None, Action::Hi),
                            menu::Item::Button("hi 45", None, Action::Hi),
                            menu::Item::Button("hi 46", None, Action::Hi),
                        ],
                    ),
                ),
            ])
            .into()]
        }
        #[cfg(feature = "wayland")]
        {
            vec![cosmic::widget::responsive_menu_bar(
                self.core(),
                &self.keybinds,
                MENU_ID.clone(),
                Message::Surface,
                vec![
                    (
                        "hiiiiiiiiiiiiiiiiiii 1".into(),
                        vec![menu::Item::Button("hi 1".into(), None, Action::Hi)],
                    ),
                    (
                        "hiiiiiiiiiiiiiiiiiii 2".into(),
                        vec![
                            menu::Item::Button("hi 2".into(), None, Action::Hi),
                            menu::Item::Button("hi 22".into(), None, Action::Hi),
                        ],
                    ),
                    (
                        "hiiiiiiiiiiiiiiiiiii 3".into(),
                        vec![
                            menu::Item::Button("hi 3".into(), None, Action::Hi),
                            menu::Item::Button("hi 33".into(), None, Action::Hi),
                            menu::Item::Button("hi 333".into(), None, Action::Hi),
                        ],
                    ),
                    (
                        "hiiiiiiiiiiiiiiiiiii 4".into(),
                        vec![
                            menu::Item::Button("hi 4".into(), None, Action::Hi),
                            menu::Item::Button("hi 44".into(), None, Action::Hi),
                            menu::Item::Button("hi 444".into(), None, Action::Hi),
                            menu::Item::Folder(
                                "nest 4".into(),
                                vec![
                                    menu::Item::Button("hi 4".into(), None, Action::Hi),
                                    menu::Item::Button("hi 44".into(), None, Action::Hi),
                                    menu::Item::Button("hi 444".into(), None, Action::Hi),
                                    menu::Item::Folder(
                                        "nest 2 4".into(),
                                        vec![
                                            menu::Item::Button("hi 4".into(), None, Action::Hi),
                                            menu::Item::Button("hi 44".into(), None, Action::Hi),
                                            menu::Item::Button("hi 444".into(), None, Action::Hi),
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
