// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Application API example

use std::collections::HashMap;
use std::{env, process};

use cosmic::app::{Command, Core, Settings};
use cosmic::iced::window;
use cosmic::iced_core::alignment::{Horizontal, Vertical};
use cosmic::iced_core::keyboard::Key;
use cosmic::iced_core::{Length, Size};
use cosmic::widget::menu::action::MenuAction;
use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::widget::menu::key_bind::Modifier;
use cosmic::widget::menu::menu_tree::{menu_items, menu_root, MenuItem};
use cosmic::widget::menu::{ItemHeight, ItemWidth, MenuBar, MenuTree};
use cosmic::widget::segmented_button::Entity;
use cosmic::{executor, Element};

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let _ = tracing_log::LogTracer::init();

    let settings = Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .debug(false)
        .default_icon_theme("Pop")
        .default_text_size(16.0)
        .scale_factor(1.0)
        .size(Size::new(1024., 768.));

    cosmic::app::run::<App>(settings, ())?;

    Ok(())
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    WindowClose,
    WindowNew,
    ToggleHideContent,
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    config: Config,
    key_binds: HashMap<KeyBind, Action>,
}

pub struct Config {
    hide_content: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    WindowClose,
    ToggleHideContent,
    WindowNew,
}

impl MenuAction for Action {
    type Message = Message;
    fn message(&self, _entity_opt: Option<Entity>) -> Self::Message {
        match self {
            Action::WindowClose => Message::WindowClose,
            Action::ToggleHideContent => Message::ToggleHideContent,
            Action::WindowNew => Message::WindowNew,
        }
    }
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
        let app = App {
            core,
            config: Config {
                hide_content: false,
            },
            key_binds: key_binds(),
        };

        (app, Command::none())
    }

    fn header_start(&self) -> Vec<Element<Self::Message>> {
        vec![menu_bar(&self.config, &self.key_binds)]
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::WindowClose => {
                return window::close(window::Id::MAIN);
            }
            Message::WindowNew => match env::current_exe() {
                Ok(exe) => match process::Command::new(&exe).spawn() {
                    Ok(_child) => {}
                    Err(err) => {
                        eprintln!("failed to execute {:?}: {}", exe, err);
                    }
                },
                Err(err) => {
                    eprintln!("failed to get current executable path: {}", err);
                }
            },
            Message::ToggleHideContent => self.config.hide_content = !self.config.hide_content,
        }
        Command::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        let text = if self.config.hide_content {
            cosmic::widget::text("")
        } else {
            cosmic::widget::text("Menu Example")
        };

        let centered = cosmic::widget::container(text)
            .width(Length::Fill)
            .height(Length::Shrink)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Center);

        Element::from(centered)
    }
}

pub fn menu_bar<'a>(config: &Config, key_binds: &HashMap<KeyBind, Action>) -> Element<'a, Message> {
    MenuBar::new(vec![MenuTree::with_children(
        menu_root("File"),
        menu_items(
            key_binds,
            vec![
                MenuItem::Button("New window", Action::WindowNew),
                MenuItem::Divider,
                MenuItem::Folder(
                    "View",
                    vec![MenuItem::CheckBox(
                        "Hide content",
                        config.hide_content,
                        Action::ToggleHideContent,
                    )],
                ),
                MenuItem::Divider,
                MenuItem::Button("Quit", Action::WindowClose),
            ],
        ),
    )])
    .item_height(ItemHeight::Dynamic(40))
    .item_width(ItemWidth::Uniform(240))
    .spacing(4.0)
    .into()
}

pub fn key_binds() -> HashMap<KeyBind, Action> {
    let mut key_binds = HashMap::new();

    macro_rules! bind {
        ([$($modifier:ident),* $(,)?], $key:expr, $action:ident) => {{
            key_binds.insert(
                KeyBind {
                    modifiers: vec![$(Modifier::$modifier),*],
                    key: $key,
                },
                Action::$action,
            );
        }};
    }

    bind!([Ctrl], Key::Character("w".into()), WindowClose);
    bind!([Ctrl, Shift], Key::Character("n".into()), WindowNew);

    key_binds
}
