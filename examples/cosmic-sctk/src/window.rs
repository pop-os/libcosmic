// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use cosmic::{
    iced::{self, wayland::window::set_mode_window, Application, Command, Length},
    iced::{
        wayland::window::{start_drag_window, toggle_maximize},
        widget::{column, container, horizontal_space, pick_list, progress_bar, row, slider},
        window, Color,
    },
    iced_futures::Subscription,
    iced_style::application,
    iced_widget::text,
    theme::{self, Theme},
    widget::{
        button, cosmic_container, header_bar, icon, inline_input, nav_bar, nav_bar_toggle,
        rectangle_tracker::{rectangle_tracker_subscription, RectangleTracker, RectangleUpdate},
        scrollable, search_input, segmented_button, segmented_selection, settings, text_input,
        IconSource,
    },
    Element, ElementExt,
};
use cosmic_time::{anim, chain, id, once_cell::sync::Lazy, Instant, Timeline};
use std::{
    sync::atomic::{AtomicU32, Ordering},
    vec,
};
use theme::Button as ButtonTheme;

static DEBUG_TOGGLER: Lazy<id::Toggler> = Lazy::new(id::Toggler::unique);
static TOGGLER: Lazy<id::Toggler> = Lazy::new(id::Toggler::unique);
static CARDS: Lazy<id::Cards> = Lazy::new(id::Cards::unique);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Page {
    Demo,
    WiFi,
    Networking,
    Bluetooth,
    Desktop,
    InputDevices,
    Displays,
    PowerAndBattery,
    Sound,
    PrintersAndScanners,
    PrivacyAndSecurity,
    SystemAndAccounts,
    UpdatesAndRecovery,
    TimeAndLanguage,
    Accessibility,
    Applications,
}

impl Page {
    //TODO: translate
    pub fn title(&self) -> &'static str {
        use Page::*;
        match self {
            Demo => "Demo",
            WiFi => "Wi-Fi",
            Networking => "Networking",
            Bluetooth => "Bluetooth",
            Desktop => "Desktop",
            InputDevices => "Input Devices",
            Displays => "Displays",
            PowerAndBattery => "Power & Battery",
            Sound => "Sound",
            PrintersAndScanners => "Printers & Scanners",
            PrivacyAndSecurity => "Privacy & Security",
            SystemAndAccounts => "System & Accounts",
            UpdatesAndRecovery => "Updates & Recovery",
            TimeAndLanguage => "Time & Language",
            Accessibility => "Accessibility",
            Applications => "Applications",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        use Page::*;
        match self {
            Demo => "document-properties-symbolic",
            WiFi => "network-wireless-symbolic",
            Networking => "network-workgroup-symbolic",
            Bluetooth => "bluetooth-active-symbolic",
            Desktop => "video-display-symbolic",
            InputDevices => "input-keyboard-symbolic",
            Displays => "preferences-desktop-display-symbolic",
            PowerAndBattery => "battery-full-charged-symbolic",
            Sound => "multimedia-volume-control-symbolic",
            PrintersAndScanners => "printer-symbolic",
            PrivacyAndSecurity => "preferences-system-privacy-symbolic",
            SystemAndAccounts => "system-users-symbolic",
            UpdatesAndRecovery => "software-update-available-symbolic",
            TimeAndLanguage => "preferences-system-time-symbolic",
            Accessibility => "preferences-desktop-accessibility-symbolic",
            Applications => "preferences-desktop-apps-symbolic",
        }
    }
}

impl Default for Page {
    fn default() -> Page {
        //TODO: what should the default page be?
        Page::Desktop
    }
}

static WINDOW_WIDTH: AtomicU32 = AtomicU32::new(0);
const BREAK_POINT: u32 = 900;

#[derive(Default)]
pub struct Window {
    title: String,
    page: Page,
    debug: bool,
    theme: Theme,
    slider_value: f32,
    checkbox_value: bool,
    toggler_value: bool,
    cards_value: bool,
    pick_list_selected: Option<&'static str>,
    nav_bar_pages: segmented_button::SingleSelectModel,
    nav_bar_toggled_condensed: bool,
    nav_bar_toggled: bool,
    show_minimize: bool,
    show_maximize: bool,
    exit: bool,
    rectangle_tracker: Option<RectangleTracker<u32>>,
    pub selection: segmented_button::SingleSelectModel,
    timeline: Timeline,
    input_value: String,
}

impl Window {
    /// Adds a page to the model we use for the navigation bar.
    fn insert_page(&mut self, page: Page) -> segmented_button::SingleSelectEntityMut {
        self.nav_bar_pages
            .insert()
            .text(page.title())
            .icon(IconSource::from(page.icon_name()))
            .data(page)
    }

    fn is_condensed(&self) -> bool {
        WINDOW_WIDTH.load(Ordering::Relaxed) < BREAK_POINT
    }

    pub fn nav_bar_toggled(mut self, toggled: bool) -> Self {
        self.nav_bar_toggled = toggled;
        self
    }

    fn page(&mut self, page: Page) {
        self.nav_bar_toggled_condensed = false;
        self.page = page;
    }

    pub fn show_maximize(mut self, show: bool) -> Self {
        self.show_maximize = show;
        self
    }

    pub fn show_minimize(mut self, show: bool) -> Self {
        self.show_minimize = show;
        self
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Message {
    Page(Page),
    Debug(bool),
    ThemeChanged(Theme),
    ButtonPressed,
    SliderChanged(f32),
    CheckboxToggled(bool),
    TogglerToggled(bool),
    CardsToggled(bool),
    PickListSelected(&'static str),
    RowSelected(usize),
    Close,
    ToggleNavBar,
    ToggleNavBarCondensed,
    Drag,
    Minimize,
    Maximize,
    Rectangle(RectangleUpdate<u32>),
    NavBar(segmented_button::Entity),
    Ignore,
    Selection(segmented_button::Entity),
    Tick(Instant),
    InputChanged(String),
}

impl Window {
    fn update_togglers(&mut self) {
        let timeline = &mut self.timeline;

        let chain = if self.toggler_value {
            chain::Toggler::on(TOGGLER.clone(), 1.)
        } else {
            chain::Toggler::off(TOGGLER.clone(), 1.)
        };
        timeline.set_chain(chain);

        let chain = if self.debug {
            chain::Toggler::on(DEBUG_TOGGLER.clone(), 1.)
        } else {
            chain::Toggler::off(DEBUG_TOGGLER.clone(), 1.)
        };
        timeline.set_chain(chain);

        timeline.start();
    }

    fn update_cards(&mut self) {
        let timeline = &mut self.timeline;
        let chain = if self.cards_value {
            chain::Cards::on(CARDS.clone(), 1.)
        } else {
            chain::Cards::off(CARDS.clone(), 1.)
        };
        timeline.set_chain(chain);
        timeline.start();
    }
}

impl Application for Window {
    type Executor = iced::executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut window = Window::default()
            .nav_bar_toggled(true)
            .show_maximize(true)
            .show_minimize(true);
        window.selection = segmented_button::Model::builder()
            .insert(|b| b.text("Choice A").activate())
            .insert(|b| b.text("Choice B"))
            .insert(|b| b.text("Choice C"))
            .build();
        window.slider_value = 50.0;
        //        window.theme = Theme::Light;
        window.pick_list_selected = Some("Option 1");
        window.title = String::from("COSMIC Design System - Iced");

        window.insert_page(Page::Demo);
        window.insert_page(Page::WiFi);
        window.insert_page(Page::Networking);
        window.insert_page(Page::Bluetooth);
        window.insert_page(Page::Desktop).activate();
        window.insert_page(Page::InputDevices);
        window.insert_page(Page::Displays);
        window.insert_page(Page::PowerAndBattery);
        window.insert_page(Page::Sound);
        window.insert_page(Page::PrintersAndScanners);
        window.insert_page(Page::PrivacyAndSecurity);
        window.insert_page(Page::SystemAndAccounts);
        window.insert_page(Page::TimeAndLanguage);
        window.insert_page(Page::Accessibility);
        window.insert_page(Page::Applications);

        (window, Command::none())
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn update(&mut self, message: Message) -> iced::Command<Self::Message> {
        match message {
            Message::NavBar(key) => {
                if let Some(page) = self.nav_bar_pages.data::<Page>(key).cloned() {
                    self.nav_bar_pages.activate(key);
                    self.page(page);
                }
            }
            Message::Page(page) => self.page = page,
            Message::Debug(debug) => {
                self.debug = debug;
                self.update_togglers();
            }
            Message::ThemeChanged(theme) => self.theme = theme,
            Message::ButtonPressed => {}
            Message::SliderChanged(value) => self.slider_value = value,
            Message::CheckboxToggled(value) => {
                self.checkbox_value = value;
            }
            Message::TogglerToggled(value) => {
                self.toggler_value = value;
                self.update_togglers();
            }
            Message::CardsToggled(value) => {
                self.cards_value = value;
                self.update_cards();
            }
            Message::PickListSelected(value) => self.pick_list_selected = Some(value),
            Message::Close => self.exit = true,
            Message::ToggleNavBar => self.nav_bar_toggled = !self.nav_bar_toggled,
            Message::ToggleNavBarCondensed => {
                self.nav_bar_toggled_condensed = !self.nav_bar_toggled_condensed
            }
            Message::Drag => return start_drag_window(window::Id(0)),
            Message::Minimize => return set_mode_window(window::Id(0), window::Mode::Hidden),
            Message::Maximize => return toggle_maximize(window::Id(0)),
            Message::RowSelected(row) => println!("Selected row {row}"),
            Message::Rectangle(r) => match r {
                RectangleUpdate::Rectangle(_) => {}
                RectangleUpdate::Init(t) => {
                    self.rectangle_tracker.replace(t);
                }
            },
            Message::Ignore => {}
            Message::Selection(key) => self.selection.activate(key),
            Message::Tick(now) => self.timeline.now(now),
            Message::InputChanged(v) => {
                self.input_value = v;
            }
        }

        Command::none()
    }

    fn view(&self, _: window::Id) -> Element<Message> {
        let (nav_bar_message, nav_bar_toggled) = if self.is_condensed() {
            (
                Message::ToggleNavBarCondensed,
                self.nav_bar_toggled_condensed,
            )
        } else {
            (Message::ToggleNavBar, self.nav_bar_toggled)
        };

        let mut header = header_bar()
            .title("COSMIC Design System - Iced")
            .on_close(Message::Close)
            .on_drag(Message::Drag)
            .start(
                nav_bar_toggle()
                    .on_toggle(nav_bar_message)
                    .active(nav_bar_toggled),
            );

        if self.show_maximize {
            header = header.on_maximize(Message::Maximize);
        }

        if self.show_minimize {
            header = header.on_minimize(Message::Minimize);
        }

        let header = Into::<Element<Message>>::into(header).debug(self.debug);

        let mut widgets = Vec::with_capacity(2);

        if nav_bar_toggled {
            let mut nav_bar = nav_bar(&self.nav_bar_pages, Message::NavBar);

            if !self.is_condensed() {
                nav_bar = nav_bar.max_width(300);
            }

            let nav_bar: Element<_> = nav_bar.into();
            widgets.push(nav_bar.debug(self.debug));
        }

        if !nav_bar_toggled {
            let secondary = button(ButtonTheme::Secondary)
                .text("Secondary")
                .on_press(Message::ButtonPressed);

            let secondary = if let Some(tracker) = self.rectangle_tracker.as_ref() {
                tracker.container(0, secondary).into()
            } else {
                secondary.into()
            };
            let content: Element<_> = settings::view_column(vec![
                settings::view_section("Debug")
                    .add(settings::item(
                        "Debug layout",
                        container(anim!(
                            //toggler
                            DEBUG_TOGGLER,
                            &self.timeline,
                            String::from("Debug layout"),
                            self.debug,
                            |_chain, enable| { Message::Debug(enable) },
                        )),
                    ))
                    .into(),
                settings::view_section("Buttons")
                    .add(settings::item_row(vec![
                        button(ButtonTheme::Primary)
                            .text("Primary")
                            .on_press(Message::ButtonPressed)
                            .into(),
                        secondary,
                        button(ButtonTheme::Positive)
                            .text("Positive")
                            .on_press(Message::ButtonPressed)
                            .into(),
                        button(ButtonTheme::Destructive)
                            .text("Destructive")
                            .on_press(Message::ButtonPressed)
                            .into(),
                        button(ButtonTheme::Text)
                            .text("Text")
                            .on_press(Message::ButtonPressed)
                            .into(),
                    ]))
                    .add(settings::item_row(vec![
                        button(ButtonTheme::Primary).text("Primary").into(),
                        button(ButtonTheme::Secondary).text("Secondary").into(),
                        button(ButtonTheme::Positive).text("Positive").into(),
                        button(ButtonTheme::Destructive).text("Destructive").into(),
                        button(ButtonTheme::Text).text("Text").into(),
                    ]))
                    .into(),
                settings::view_section("Controls")
                    .add(settings::item(
                        "Toggler",
                        anim!(
                            //toggler
                            TOGGLER,
                            &self.timeline,
                            None,
                            self.toggler_value,
                            |_chain, enable| { Message::TogglerToggled(enable) },
                        ),
                    ))
                    .add(settings::item(
                        "Pick List (TODO)",
                        pick_list(
                            vec!["Option 1", "Option 2", "Option 3", "Option 4"],
                            self.pick_list_selected,
                            Message::PickListSelected,
                        )
                        .text_size(14.0),
                    ))
                    .add(settings::item(
                        "Slider",
                        slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                            .width(Length::Fixed(250.0)),
                    ))
                    .add(settings::item(
                        "Progress",
                        progress_bar(0.0..=100.0, self.slider_value)
                            .width(Length::Fixed(250.0))
                            .height(Length::Fixed(4.0)),
                    ))
                    .add(settings::item(
                        "Segmented Button",
                        segmented_selection::horizontal(&self.selection)
                            .on_activate(Message::Selection),
                    ))
                    .add(settings::item(
                        "Cards",
                        cosmic_container::container(anim!(
                            //cards
                            CARDS,
                            &self.timeline,
                            vec![
                                text("Card 1").size(24).width(Length::Fill).into(),
                                text("Card 2").size(24).width(Length::Fill).into(),
                                text("Card 3").size(24).width(Length::Fill).into(),
                                text("Card 4").size(24).width(Length::Fill).into(),
                            ],
                            Message::Ignore,
                            |_, e| Message::CardsToggled(e),
                            "Show More",
                            "Show Less",
                            "Clear All",
                            None,
                            self.cards_value,
                        ))
                        .layer(cosmic::cosmic_theme::Layer::Secondary)
                        .padding(16)
                        .style(cosmic::theme::Container::Secondary),
                    ))
                    .add(settings::item(
                        "Text Input",
                        text_input("test", &self.input_value)
                            .width(Length::Fill)
                            .on_input(Message::InputChanged),
                    ))
                    .add(settings::item(
                        "Text Input",
                        text_input("test", &self.input_value)
                            .start_icon(icon("document-properties-symbolic", 16).into())
                            .end_icon(icon("document-properties-symbolic", 16).into())
                            .label("Test Label")
                            .helper_text("helper_text")
                            .width(Length::Fill)
                            .on_input(Message::InputChanged),
                    ))
                    .add(settings::item(
                        "Text Input",
                        search_input(
                            "search for stuff",
                            &self.input_value,
                            Message::InputChanged("".to_string()),
                        )
                        .width(Length::Fill)
                        .on_input(Message::InputChanged),
                    ))
                    .add(settings::item(
                        "Text Input",
                        inline_input(&self.input_value)
                            .width(Length::Fill)
                            .on_input(Message::InputChanged),
                    ))
                    .into(),
            ])
            .into();

            widgets.push(
                scrollable(row![
                    horizontal_space(Length::Fill),
                    content.debug(self.debug),
                    horizontal_space(Length::Fill),
                ])
                .into(),
            );
        }

        let content = container(row(widgets))
            .padding([0, 8, 8, 8])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::Container::Background)
            .into();

        column(vec![header, content]).into()
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }

    fn close_requested(&self, _id: window::Id) -> Self::Message {
        Message::Close
    }
    fn subscription(&self) -> iced::Subscription<Self::Message> {
        Subscription::batch(vec![
            rectangle_tracker_subscription(0).map(|(_, e)| Self::Message::Rectangle(e)),
            self.timeline
                .as_subscription()
                .map(|(_, instant)| Self::Message::Tick(instant)),
        ])
    }

    fn style(&self) -> <Self::Theme as cosmic::iced_style::application::StyleSheet>::Style {
        cosmic::theme::Application::Custom(Box::new(|theme| application::Appearance {
            background_color: Color::TRANSPARENT,
            text_color: theme.cosmic().on_bg_color().into(),
        }))
    }
}
