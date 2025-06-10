// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Table API example

use std::collections::HashMap;

use chrono::Datelike;
use cosmic::app::{Core, Settings, Task};
use cosmic::iced_core::Size;
use cosmic::prelude::*;
use cosmic::widget::table;
use cosmic::widget::{self, nav_bar};
use cosmic::{executor, iced};

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Category {
    #[default]
    Name,
    Date,
    Size,
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Name => "Name",
            Self::Date => "Date",
            Self::Size => "Size",
        })
    }
}

impl table::ItemCategory for Category {
    fn width(&self) -> iced::Length {
        match self {
            Self::Name => iced::Length::Fill,
            Self::Date => iced::Length::Fixed(200.0),
            Self::Size => iced::Length::Fixed(150.0),
        }
    }
}

struct Item {
    name: String,
    date: chrono::DateTime<chrono::Local>,
    size: u64,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            name: Default::default(),
            date: Default::default(),
            size: Default::default(),
        }
    }
}

impl table::ItemInterface<Category> for Item {
    fn get_icon(&self, category: Category) -> Option<cosmic::widget::Icon> {
        if category == Category::Name {
            Some(cosmic::widget::icon::from_name("application-x-executable-symbolic").icon())
        } else {
            None
        }
    }

    fn get_text(&self, category: Category) -> std::borrow::Cow<'static, str> {
        match category {
            Category::Name => self.name.clone().into(),
            Category::Date => self.date.format("%Y/%m/%d").to_string().into(),
            Category::Size => format!("{} items", self.size).into(),
        }
    }

    fn compare(&self, other: &Self, category: Category) -> std::cmp::Ordering {
        match category {
            Category::Name => self.name.to_lowercase().cmp(&other.name.to_lowercase()),
            Category::Date => self.date.cmp(&other.date),
            Category::Size => self.size.cmp(&other.size),
        }
    }
}

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let _ = tracing_log::LogTracer::init();

    let settings = Settings::default()
        .size(Size::new(1024., 768.));

    cosmic::app::run::<App>(settings, ())?;

    Ok(())
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    ItemSelect(table::Entity),
    CategorySelect(Category),
    PrintMsg(String),
    NoOp,
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    table_model: table::SingleSelectModel<Item, Category>,
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
    const APP_ID: &'static str = "org.cosmic.AppDemoTable";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Creates the application, and optionally emits task on initialize.
    fn init(core: Core, _: Self::Flags) -> (Self, Task<Self::Message>) {
        let mut nav_model = nav_bar::Model::default();

        nav_model.activate_position(0);

        let mut table_model =
            table::Model::new(vec![Category::Name, Category::Date, Category::Size]);

        let _ = table_model.insert(Item {
            name: "Foo".into(),
            date: chrono::DateTime::default()
                .with_day(1)
                .unwrap()
                .with_month(1)
                .unwrap()
                .with_year(1970)
                .unwrap(),
            size: 2,
        });
        let _ = table_model.insert(Item {
            name: "Bar".into(),
            date: chrono::DateTime::default()
                .with_day(2)
                .unwrap()
                .with_month(1)
                .unwrap()
                .with_year(1970)
                .unwrap(),
            size: 4,
        });
        let _ = table_model.insert(Item {
            name: "Baz".into(),
            date: chrono::DateTime::default()
                .with_day(3)
                .unwrap()
                .with_month(1)
                .unwrap()
                .with_year(1970)
                .unwrap(),
            size: 12,
        });

        let app = App { core, table_model };

        let command = Task::none();

        (app, command)
    }

    /// Handle application events here.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::ItemSelect(entity) => self.table_model.activate(entity),
            Message::CategorySelect(category) => {
                let mut ascending = true;
                if let Some(old_sort) = self.table_model.get_sort() {
                    if old_sort.0 == category {
                        ascending = !old_sort.1;
                    }
                }
                self.table_model.sort(category, ascending)
            }
            Message::PrintMsg(string) => tracing_log::log::info!("{}", string),
            Message::NoOp => {}
        }
        Task::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        cosmic::widget::responsive(|size| {
            if size.width < 600.0 {
                widget::compact_table(&self.table_model)
                    .on_item_left_click(Message::ItemSelect)
                    .item_context(move |item| {
                        Some(widget::menu::items(
                            &HashMap::new(),
                            vec![widget::menu::Item::Button(
                                format!("Action on {}", item.name.to_string()),
                                None,
                                Action::None,
                            )],
                        ))
                    })
                    .apply(Element::from)
            } else {
                widget::table(&self.table_model)
                    .on_item_left_click(Message::ItemSelect)
                    .on_category_left_click(Message::CategorySelect)
                    .item_context(|item| {
                        Some(widget::menu::items(
                            &HashMap::new(),
                            vec![widget::menu::Item::Button(
                                format!("Action on {}", item.name),
                                None,
                                Action::None,
                            )],
                        ))
                    })
                    .category_context(|category| {
                        Some(widget::menu::items(
                            &HashMap::new(),
                            vec![
                                widget::menu::Item::Button(
                                    format!("Action on {} category", category.to_string()),
                                    None,
                                    Action::None,
                                ),
                                widget::menu::Item::Button(
                                    format!("Other action on {} category", category.to_string()),
                                    None,
                                    Action::None,
                                ),
                            ],
                        ))
                    })
                    .apply(Element::from)
            }
        })
        .into()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    None,
}

impl widget::menu::Action for Action {
    type Message = Message;

    fn message(&self) -> Self::Message {
        Message::NoOp
    }
}
