use std::collections::HashMap;

use cosmic::{
    app::Core,
    iced::{self, event, window},
    iced_core::{id, Alignment, Length, Point},
    iced_widget::{column, container, scrollable, text},
    widget::{button, header_bar},
    ApplicationExt, Task,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    CloseWindow(window::Id),
    WindowOpened(window::Id, Option<Point>),
    WindowClosed(window::Id),
    NewWindow,
    Input(id::Id, String),
}
pub struct MultiWindow {
    core: Core,
    windows: HashMap<window::Id, Window>,
}

pub struct Window {
    input_id: id::Id,
    input_value: String,
}

impl cosmic::Application for MultiWindow {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;

    const APP_ID: &'static str = "org.cosmic.MultiWindowDemo";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn init(core: Core, _input: Self::Flags) -> (Self, cosmic::app::Task<Self::Message>) {
        let windows = MultiWindow {
            windows: HashMap::from([(
                core.main_window_id().unwrap(),
                Window {
                    input_id: id::Id::new("main"),
                    input_value: String::new(),
                },
            )]),
            core,
        };

        (windows, cosmic::app::Task::none())
    }

    fn subscription(&self) -> cosmic::iced_futures::Subscription<Self::Message> {
        event::listen_with(|event, _, id| {
            if let iced::Event::Window(window_event) = event {
                match window_event {
                    window::Event::CloseRequested => Some(Message::CloseWindow(id)),
                    window::Event::Opened { position, .. } => {
                        Some(Message::WindowOpened(id, position))
                    }
                    window::Event::Closed => Some(Message::WindowClosed(id)),
                    _ => None,
                }
            } else {
                None
            }
        })
    }

    fn update(&mut self, message: Self::Message) -> iced::Task<cosmic::Action<Self::Message>> {
        match message {
            Message::CloseWindow(id) => window::close(id),
            Message::WindowClosed(id) => {
                self.windows.remove(&id);
                Task::none()
            }
            Message::WindowOpened(id, ..) => {
                if let Some(window) = self.windows.get(&id) {
                    cosmic::widget::text_input::focus(window.input_id.clone())
                } else {
                    Task::none()
                }
            }
            Message::NewWindow => {
                let count = self.windows.len() + 1;

                let (id, spawn_window) = window::open(window::Settings {
                    position: Default::default(),
                    exit_on_close_request: count % 2 == 0,
                    decorations: false,
                    ..Default::default()
                });

                self.windows.insert(
                    id,
                    Window {
                        input_id: id::Id::new(format!("window_{}", count)),
                        input_value: String::new(),
                    },
                );
                _ = self.set_window_title(format!("window_{}", count), id);

                spawn_window.map(|id| cosmic::Action::App(Message::WindowOpened(id, None)))
            }
            Message::Input(id, value) => {
                if let Some((_, w)) = self.windows.iter_mut().find(|e| e.1.input_id == id) {
                    w.input_value = value;
                }

                Task::none()
            }
        }
    }

    fn view_window(&self, id: window::Id) -> cosmic::prelude::Element<Self::Message> {
        let w = self.windows.get(&id).unwrap();

        let input_id = w.input_id.clone();
        let input = cosmic::widget::text_input::text_input("something", &w.input_value)
            .on_input(move |msg| Message::Input(input_id.clone(), msg))
            .id(w.input_id.clone());
        let focused = self
            .core()
            .focused_window()
            .map(|i| i == id)
            .unwrap_or_default();
        let new_window_button = button::custom(text("New Window")).on_press(Message::NewWindow);

        let content = scrollable(
            column![input, new_window_button]
                .spacing(50)
                .width(Length::Fill)
                .align_x(Alignment::Center),
        );

        let window_content = container(container(content).center_x(Length::Fixed(200.)))
            .class(cosmic::style::Container::Background)
            .center_x(Length::Fill)
            .center_y(Length::Fill);

        if id == self.core.main_window_id().unwrap() {
            window_content.into()
        } else {
            column![header_bar().focused(focused), window_content].into()
        }
    }

    fn view(&self) -> cosmic::prelude::Element<Self::Message> {
        self.view_window(self.core.main_window_id().unwrap())
    }
}
