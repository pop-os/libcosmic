// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod buttons;
mod cards;
mod debug;
mod inputs;
mod typography;

use cosmic::app::{Command, Core, Settings};
use cosmic::cosmic_theme::palette::{rgb::Rgb, Srgba};
use cosmic::cosmic_theme::ThemeBuilder;
use cosmic::iced::Length;
use cosmic::widget::{
    button, column, container, icon, nav_bar, scrollable, segmented_button, spin_button,
};
use cosmic::{executor, ApplicationExt, Apply, Element, Theme};
use cosmic_time::Timeline;
use debug::ThemeVariant;
use fraction::{Decimal, ToPrimitive};
use std::sync::Arc;

#[derive(Clone, Copy)]
pub enum Page {
    Buttons,
    Cards,
    Inputs,
    Typography,
}

impl Page {
    const fn as_str(self) -> &'static str {
        match self {
            Page::Buttons => "Buttons",
            Page::Cards => "Cards",
            Page::Inputs => "Inputs",
            Page::Typography => "Typography",
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
        .size((1280, 768))
        .theme(cosmic::Theme::dark());

    cosmic::app::run::<App>(settings, ())?;

    Ok(())
}

/// Messages that are used specifically by our [`App`].
#[derive(Clone, Debug)]
pub enum Message {
    CardsToggled(bool),
    CheckboxToggled(bool),
    Clicked,
    DebugToggled(bool),
    Ignore,
    LayerSelect(&'static str),
    PickListSelected(&'static str),
    ScalingFactorChanged(spin_button::Message),
    SecureInputToggled,
    Selection(segmented_button::Entity),
    SliderChanged(f32),
    TextInputChanged(String),
    ThemeChanged(ThemeVariant),
    Tick(cosmic_time::Instant),
    TogglerToggled(bool),
}

/// The [`App`] stores application-specific state.
pub struct App {
    core: Core,
    nav_model: nav_bar::Model,
    layer_selection: &'static str,

    // cosmic-time dependency
    timeline: Timeline,

    // Buttons page
    app_icon: icon::Handle,
    bt_icon: icon::Handle,
    leading_icon: icon::Handle,
    trailing_icon: icon::Handle,

    // Cards page
    cards_value: bool,

    // Debug page
    scale_factor: spin_button::Model<Decimal>,
    scale_factor_str: String,

    // Inputs page
    checkbox_value: bool,
    pick_list_selected: Option<&'static str>,
    pick_list_options: Vec<&'static str>,
    text_input_value: String,
    secure_input_visible: bool,
    selection: segmented_button::SingleSelectModel,
    slider_value: f32,
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
    const APP_ID: &'static str = "com.system76.CosmicDesignDemo";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// Creates the application, and optionally emits command on initialize.
    fn init(core: Core, _input: Self::Flags) -> (Self, Command<Self::Message>) {
        let nav_model = nav_bar::Model::builder()
            .insert(|e| e.text(Page::Typography.as_str()).data(Page::Typography))
            .insert(|e| {
                e.text(Page::Buttons.as_str())
                    .data(Page::Buttons)
                    .activate()
            })
            .insert(|e| e.text(Page::Cards.as_str()).data(Page::Cards))
            .insert(|e| e.text(Page::Inputs.as_str()).data(Page::Inputs))
            .build();

        let mut app = App {
            nav_model,
            layer_selection: "Default",

            // cosmic-time dependency
            timeline: Timeline::default(),

            // Buttons page
            app_icon: icon::from_name("firefox").into(),
            bt_icon: icon::from_name("bluetooth-active-symbolic").size(16).into(),
            leading_icon: icon::from_name("document-save-symbolic").size(16).into(),
            trailing_icon: button::link::icon(),

            // Cards page
            cards_value: false,

            // Debug page
            scale_factor: spin_button::Model::default()
                .value(core.scale_factor())
                .min(0.5)
                .max(4.0)
                .step(0.25),
            scale_factor_str: core.scale_factor().to_string(),

            // Inputs page
            checkbox_value: false,
            pick_list_selected: Some("Option 1"),
            pick_list_options: vec!["Option 1", "Option 2", "Option 3", "Option 4"],
            text_input_value: String::new(),
            secure_input_visible: false,
            selection: segmented_button::Model::builder()
                .insert(|b| b.text("Choice A").activate())
                .insert(|b| b.text("Choice B"))
                .insert(|b| b.text("Choice C"))
                .build(),
            slider_value: 0.0,

            core,
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
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            Message::Tick(now) => self.timeline.now(now),

            Message::TextInputChanged(input) => {
                self.text_input_value = input;
            }

            Message::Clicked => {
                eprintln!("button clicked");
            }

            Message::CardsToggled(value) => {
                self.cards_value = value;
                self.update_cards();
            }

            Message::DebugToggled(value) => {
                self.core.debug = value;
                self.update_togglers();
            }

            Message::LayerSelect(selection) => {
                self.layer_selection = selection;
            }

            Message::SecureInputToggled => {
                self.secure_input_visible = !self.secure_input_visible;
            }

            Message::Selection(key) => self.selection.activate(key),

            Message::CheckboxToggled(value) => {
                self.checkbox_value = value;
            }

            Message::SliderChanged(value) => {
                self.slider_value = value;
            }

            Message::TogglerToggled(value) => {
                eprintln!("card toggler: {value}");
            }

            Message::PickListSelected(value) => self.pick_list_selected = Some(value),

            Message::ThemeChanged(theme) => {
                return cosmic::app::command::set_theme(match theme {
                    ThemeVariant::Light => Theme::light(),
                    ThemeVariant::Dark => Theme::dark(),
                    ThemeVariant::HighContrastDark => Theme::dark_hc(),
                    ThemeVariant::HighContrastLight => Theme::light_hc(),
                    ThemeVariant::Custom => Theme::custom(Arc::new(
                        ThemeBuilder::light()
                            .bg_color(Srgba::new(1.0, 0.9, 0.9, 1.0))
                            .text_tint(Rgb::new(0.0, 1.0, 0.0))
                            .neutral_tint(Rgb::new(0.0, 0.5, 1.0))
                            .accent(Rgb::new(0.5, 0.1, 0.5))
                            .success(Rgb::new(0.0, 0.5, 0.3))
                            .warning(Rgb::new(0.894, 0.816, 0.039))
                            .destructive(Rgb::new(0.890, 0.145, 0.420))
                            .build(),
                    )),
                    ThemeVariant::System => cosmic::theme::system_preference(),
                });
            }

            Message::ScalingFactorChanged(message) => {
                self.scale_factor.update(message);
                if let Some(factor) = self.scale_factor.value.to_f32() {
                    self.scale_factor_str = factor.to_string();
                    return cosmic::app::command::set_scaling_factor(factor);
                }
            }

            Message::Ignore => (),
        }

        Command::none()
    }

    /// Creates a view after each update.
    fn view(&self) -> Element<Self::Message> {
        // Generate a view for the active page.
        let page_view = match self.nav_model.active_data::<Page>() {
            Some(Page::Buttons) => self.view_buttons(),
            Some(Page::Cards) => self.view_cards(),
            Some(Page::Inputs) => self.view_text_input(),
            Some(Page::Typography) => self.view_typography(),
            None => cosmic::widget::text("Unknown page selected").into(),
        };

        column()
            .spacing(24)
            // Place debug view atop each page
            .push(self.view_debug())
            // Insert page view beneath it
            .push(
                container(page_view)
                    .width(Length::Fill)
                    .style(match self.layer_selection {
                        "Primary" => cosmic::theme::Container::Primary,
                        "Secondary" => cosmic::theme::Container::Secondary,
                        _ => cosmic::theme::Container::default(),
                    }),
            )
            // Wrap page views in container that expands up to 1000 px wide.
            .apply(container)
            .width(Length::Fill)
            .max_width(1000)
            // Wrap again to center-align the expanded container.
            .apply(container)
            .center_x()
            .width(Length::Fill)
            // Make it scrollable if height exceeds window.
            .apply(scrollable)
            .into()
    }

    fn subscription(&self) -> cosmic::iced::Subscription<Self::Message> {
        cosmic::iced::Subscription::batch(vec![self
            .timeline
            .as_subscription()
            .map(|(_, instant)| Message::Tick(instant))])
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
        let window_title = format!("{title} - COSMIC Design System");
        self.core.window.header_title = title.clone();
        self.set_title(window_title)
    }

    fn update_togglers(&mut self) {
        let chain = if self.core.debug {
            cosmic_time::chain::Toggler::on(debug::DEBUG_TOGGLER.clone(), 1.)
        } else {
            cosmic_time::chain::Toggler::off(debug::DEBUG_TOGGLER.clone(), 1.)
        };

        self.timeline.set_chain(chain);

        self.timeline.start();
    }
}
