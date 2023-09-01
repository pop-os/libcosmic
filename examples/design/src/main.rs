// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Controls: buttons, radio buttons, toggles, etc.

use cosmic::app::{Command, Core, Settings};
use cosmic::widget::{button, column, container, icon, nav_bar, row, scrollable, text};
use cosmic::{executor, iced, ApplicationExt, Apply, Element};

#[derive(Clone, Copy)]
pub enum Page {
    Buttons,
}

impl Page {
    const fn as_str(self) -> &'static str {
        match self {
            Page::Buttons => "Buttons",
        }
    }
}

/// Runs application with these settings
#[rustfmt::skip]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default()
        .antialiasing(true)
        .client_decorations(true)
        .debug(false)
        .size((1024, 768))
        .theme(cosmic::Theme::dark());

    cosmic::app::run::<App>(settings, &[Page::Buttons])?;

    Ok(())
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    Clicked,
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    nav_model: nav_bar::Model,
    app_icon: icon::Handle,
    bt_icon: icon::Handle,
    leading_icon: icon::Handle,
    trailing_icon: icon::Handle,
}

/// Implement [`cosmic::Application`] to integrate with COSMIC.
impl cosmic::Application for App {
    /// Default async executor to use with the app.
    type Executor = executor::Default;

    /// Argument received [`cosmic::Application::new`].
    type Flags = &'static [Page];

    /// Message type specific to our [`App`].
    type Message = Message;

    const APP_ID: &'static str = "org.cosmic.DesignDemo";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Creates the application, and optionally emits command on initialize.
    fn init(core: Core, input: Self::Flags) -> (Self, Command<Self::Message>) {
        let mut nav_model = nav_bar::Model::default();

        for &page in input {
            nav_model.insert().text(page.as_str()).data(page);
        }

        nav_model.activate_position(0);

        let mut app = App {
            core,
            app_icon: icon::handle::from_name("firefox").size(16).handle(),
            bt_icon: icon::handle::from_name("bluetooth-active-symbolic")
                .size(16)
                .handle(),
            leading_icon: icon::handle::from_name("document-save-symbolic")
                .size(16)
                .handle(),
            trailing_icon: button::hyperlink::icon(),
            nav_model,
        };

        let command = app.update_title();

        (app, command)
    }

    /// Allows COSMIC to integrate with your application's [`nav_bar::Model`].
    fn nav_model(&self) -> Option<&nav_bar::Model> {
        Some(&self.nav_model)
    }

    /// Called when a navigation item is selected.
    fn on_nav_select(&mut self, id: nav_bar::Id) -> Command<Self::Message> {
        self.nav_model.activate(id);
        self.update_title()
    }

    /// Handle application events here.
    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        let page_content = match self.nav_model.active_data::<Page>() {
            Some(Page::Buttons) => self.view_buttons(),
            None => cosmic::widget::text("Unknown page selected").into(),
        };

        container(page_content)
            .width(iced::Length::Fill)
            .align_x(iced::alignment::Horizontal::Center)
            .apply(scrollable)
            .into()
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

    fn update_title(&mut self) -> Command<Message> {
        let title = self.active_page_title().to_owned();
        self.set_title(title)
    }

    fn view_buttons(&self) -> Element<Message> {
        column()
            .max_width(800)
            .spacing(24)
            .push(text::title1("Label Buttons"))
            // Suggested button header
            .push(
                column()
                    .spacing(8)
                    .push(text::title3("Suggested Button"))
                    .push(text("Highest level of attention, there should only be one primary button used on the page.").size(14.0))
            )
            // Suggested button demo
            .push(
                row()
                    .spacing(36)
                    .push(button::suggested("Label").on_press(Message::Clicked))
                    .push(button::suggested("Label").on_press(Message::Clicked).leading_icon(self.leading_icon.clone()))
                    .push(button::suggested("Label").on_press(Message::Clicked).trailing_icon(self.trailing_icon.clone()))
                    .push(button::suggested("Label").on_press(Message::Clicked).leading_icon(self.app_icon.clone()))
                    .push(
                        button::suggested("Label")
                            .on_press(Message::Clicked)
                            .leading_icon(self.app_icon.clone())
                            .trailing_icon(self.trailing_icon.clone())
                    )
            )
            // Destructive button header
            .push(
                column()
                    .spacing(8)
                    .push(text::title3("Destructive Button"))
                    .push(text("Highest level of attention, there should only be one primary button used on the page.").size(14.0))
            )
            // Destructive button demo
            .push(
                row()
                    .spacing(36)
                    .push(button::destructive("Label").on_press(Message::Clicked))
                    .push(button::destructive("Label").on_press(Message::Clicked).leading_icon(self.leading_icon.clone()))
                    .push(button::destructive("Label").on_press(Message::Clicked).trailing_icon(self.trailing_icon.clone()))
                    .push(button::destructive("Label").on_press(Message::Clicked).leading_icon(self.app_icon.clone()))
                    .push(
                        button::destructive("Label")
                            .on_press(Message::Clicked)
                            .leading_icon(self.app_icon.clone())
                            .trailing_icon(self.trailing_icon.clone())
                    )
            )
            // Standard button header
            .push(
                column()
                    .spacing(8)
                    .push(text::title3("Standard Button"))
                    .push(
                        text(
                            "Requires less attention from the user. Could be more \
                            than one button on the page, if necessary."
                        )
                        .size(14.0)
                    )
            )
            // Standard button demo
            .push(
                row()
                    .spacing(36)
                    .push(button::standard("Label").on_press(Message::Clicked))
                    .push(button::standard("Label").on_press(Message::Clicked).leading_icon(self.leading_icon.clone()))
                    .push(button::standard("Label").on_press(Message::Clicked).trailing_icon(self.trailing_icon.clone()))
                    .push(button::standard("Label").on_press(Message::Clicked).leading_icon(self.app_icon.clone()))
                    .push(
                        button::standard("Label")
                            .on_press(Message::Clicked)
                            .leading_icon(self.app_icon.clone())
                            .trailing_icon(self.trailing_icon.clone())
                    )
            )
            // Text button header
            .push(
                column()
                    .spacing(8)
                    .push(text::title3("Text Button"))
                    .push(text(
                        "Lowest priority actions, especially when presenting multiple options. Because text buttons \
                        don’t have a visible container in their default state, they don’t distract from nearby \
                        content. But they are also more difficult to recognize because of that."
                    ).size(14.0))
            )
            // Text button demo
            .push(
                row()
                    .spacing(36)
                    .push(button::text("Label").on_press(Message::Clicked))
                    .push(button::text("Label").on_press(Message::Clicked).leading_icon(self.leading_icon.clone()))
                    .push(button::text("Label").on_press(Message::Clicked).trailing_icon(self.trailing_icon.clone()))
                    .push(button::text("Label").on_press(Message::Clicked).leading_icon(self.app_icon.clone()))
                    .push(
                        button::text("Label")
                            .on_press(Message::Clicked)
                            .leading_icon(self.app_icon.clone())
                            .trailing_icon(self.trailing_icon.clone())
                    )
            )
            // Icon buttons
            .push(text::title1("Icon Buttons"))
            // Extra small icon buttons
            .push(
                row()
                    .spacing(36)
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).extra_small())
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).extra_small().selected(true))
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).extra_small().label("Label"))
                    .push(
                        button::icon(self.bt_icon.clone())
                            .on_press(Message::Clicked)
                            .extra_small()
                            .label("Label")
                            .selected(true)
                    )
            )
            // Small (default) icon buttons
            .push(
                row()
                    .spacing(36)
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked))
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).selected(true))
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).label("Label"))
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).label("Label").selected(true))
            )
            // Medium icon buttons
            .push(
                row()
                    .spacing(36)
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).medium())
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).medium().selected(true))
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).medium().label("Label"))
                    .push(
                        button::icon(self.bt_icon.clone())
                            .on_press(Message::Clicked)
                            .medium()
                            .label("Label")
                            .selected(true)
                    )
            )
            // Large icon buttons
            .push(
                row()
                    .spacing(36)
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).large())
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).large().selected(true))
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).large().label("Label"))
                    .push(
                        button::icon(self.bt_icon.clone())
                            .on_press(Message::Clicked)
                            .large()
                            .label("Label")
                            .selected(true)
                    )
            )
            // Extra large icon buttons
            .push(
                row()
                    .spacing(36)
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).extra_large())
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).extra_large().selected(true))
                    .push(button::icon(self.bt_icon.clone()).on_press(Message::Clicked).extra_large().label("Label"))
                    .push(
                        button::icon(self.bt_icon.clone())
                            .on_press(Message::Clicked)
                            .extra_large()
                            .label("Label")
                            .selected(true)
                    )
            )
            .into()
    }
}
