// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use cosmic::{
    iced::widget::{
        column, container, horizontal_space, pick_list, progress_bar, radio, row, slider,
    },
    iced::{self, Alignment, Application, Command, Length},
    iced_lazy::responsive,
    iced_winit::window::{drag, toggle_maximize, minimize},
    theme::{self, Theme},
    widget::{button, nav_button, nav_bar, header_bar, settings, scrollable, toggler, nav_bar_item},
    Element,
    ElementExt,
};
use std::{collections::BTreeMap, vec};
use theme::Button as ButtonTheme;

#[derive(Default)]
pub struct Window {    
    title: String,
    page: usize,
    section: usize,
    debug: bool,
    theme: Theme,
    slider_value: f32,
    checkbox_value: bool,
    toggler_value: bool,
    pick_list_selected: Option<&'static str>,
    sidebar_toggled: bool,
    show_minimize: bool,
    show_maximize: bool,
    exit: bool,
}

impl Window {
    pub fn sidebar_toggled(mut self, toggled: bool) -> Self {
        self.sidebar_toggled = toggled;
        self
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
#[derive(Clone, Copy, Debug)]
pub enum Message {
    Page(usize, usize),
    Debug(bool),
    ThemeChanged(Theme),
    ButtonPressed,
    SliderChanged(f32),
    CheckboxToggled(bool),
    TogglerToggled(bool),
    PickListSelected(&'static str),
    RowSelected(usize),
    Close,
    ToggleSidebar,
    Drag,
    Minimize,
    Maximize,
}

impl Application for Window {
    type Executor = iced::executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut window = Window::default()
            .sidebar_toggled(true)
            .show_maximize(true)
            .show_minimize(true);
        window.slider_value = 50.0;
        //        window.theme = Theme::Light;
        window.pick_list_selected = Some("Option 1");
        window.title = String::from("COSMIC Design System - Iced");
        (window, Command::none())
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn update(&mut self, message: Message) -> iced::Command<Self::Message> {
        match message {
            Message::Page(section, page) => {
                self.page = page;
                self.section = section;
            }
            Message::Debug(debug) => self.debug = debug,
            Message::ThemeChanged(theme) => self.theme = theme,
            Message::ButtonPressed => {}
            Message::SliderChanged(value) => self.slider_value = value,
            Message::CheckboxToggled(value) => self.checkbox_value = value,
            Message::TogglerToggled(value) => self.toggler_value = value,
            Message::PickListSelected(value) => self.pick_list_selected = Some(value),
            Message::Close => self.exit = true,
            Message::ToggleSidebar => self.sidebar_toggled = !self.sidebar_toggled,
            Message::Drag => return drag(),
            Message::Minimize => return minimize(true),
            Message::Maximize => return toggle_maximize(),
            Message::RowSelected(row) => println!("Selected row {row}"),
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let mut header = header_bar()
            .title("COSMIC Design System - Iced")
            .on_close(Message::Close)
            .on_drag(Message::Drag)
            .start(
                nav_button("Settings")
                    .on_sidebar_toggled(Message::ToggleSidebar)
                    .sidebar_active(self.sidebar_toggled)
                    .into()
            );

        if self.show_maximize {
            header = header.on_maximize(Message::Maximize);
        }

        if self.show_minimize {
            header = header.on_minimize(Message::Minimize);
        }

        let header = Into::<Element<Message>>::into(header).debug(self.debug);

        // TODO: Adding responsive makes this regenerate on every size change, and regeneration
        // involves allocations for many different items. Ideally, we could only make the nav bar
        // responsive and leave the content to be sized normally.
        let content = responsive(|size| {
            let condensed = size.width < 900.0;
            let sidebar: Element<_> = nav_bar()
                .source(BTreeMap::from([
                    (
                        nav_bar_item()
                            .title("Network & Wireless")
                            .icon("nm-device-wired"),
                        vec![nav_bar_item().title("Wi-Fi").icon("network-wireless")],
                    ),
                    (
                        nav_bar_item().title("Bluetooth").icon("cs-bluetooth"),
                        vec![nav_bar_item().title("Devices").icon("computer")],
                    ),
                    (
                        nav_bar_item()
                            .title("Personalization")
                            .icon("applications-system"),
                        vec![
                            nav_bar_item()
                                .title("Desktop Session")
                                .icon("desktop-panel"),
                            nav_bar_item()
                                .title("Wallpaper")
                                .icon("preferences-desktop-wallpaper"),
                            nav_bar_item().title("Appearance").icon("cs-color"),
                            nav_bar_item()
                                .title("Dock & Top Panel")
                                .icon("desktop-panel"),
                            nav_bar_item()
                                .title("Workspaces")
                                .icon("preferences-system-windows"),
                            nav_bar_item()
                                .title("Notifications")
                                .icon("cs-notifications"),
                        ],
                    ),
                    (
                        nav_bar_item().title("Input Devices").icon("input-keyboard"),
                        vec![nav_bar_item().title("Keyboard").icon("computer")],
                    ),
                    (
                        nav_bar_item().title("Displays").icon("cs-display"),
                        vec![nav_bar_item().title("External Monitors").icon("computer")],
                    ),
                    (
                        nav_bar_item().title("Power & Battery").icon("battery"),
                        vec![nav_bar_item().title("Status").icon("computer")],
                    ),
                    (
                        nav_bar_item().title("Sound").icon("sound"),
                        vec![nav_bar_item().title("Volume").icon("computer")],
                    ),
                ]))
                .on_page_selected(Box::new(Message::Page))
                .active(self.sidebar_toggled)
                .condensed(condensed)
                .into();

            let choose_theme = [Theme::Light, Theme::Dark].iter().fold(
                row![].spacing(10).align_items(Alignment::Center),
                |row, theme| {
                    row.push(radio(
                        format!("{:?}", theme),
                        *theme,
                        Some(self.theme),
                        Message::ThemeChanged,
                    ))
                },
            );

            let content: Element<_> = settings::view_column(vec![
                settings::view_section("Debug")
                    .add(settings::item("Debug theme", choose_theme))
                    .add(settings::item(
                        "Debug layout",
                        toggler(String::from("Debug layout"), self.debug, Message::Debug)
                    ))
                    .into(),
                settings::view_section("Buttons")
                    .add(settings::item_row(vec![
                        button(ButtonTheme::Primary)
                            .text("Primary")
                            .on_press(Message::ButtonPressed)
                            .into(),
                        button(ButtonTheme::Secondary)
                            .text("Secondary")
                            .on_press(Message::ButtonPressed)
                            .into(),
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
                            .into()
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
                    .add(settings::item("Toggler", toggler(None, self.toggler_value, Message::TogglerToggled)))
                    .add(settings::item(
                        "Pick List (TODO)",
                        pick_list(
                            vec!["Option 1", "Option 2", "Option 3", "Option 4",],
                            self.pick_list_selected,
                            Message::PickListSelected
                        )
                        .padding([8, 0, 8, 16])
                    ))
                    .add(settings::item(
                        "Slider",
                        slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                            .width(Length::Units(250))
                    ))
                    .add(settings::item(
                        "Progress",
                        progress_bar(0.0..=100.0, self.slider_value)
                            .width(Length::Units(250))
                            .height(Length::Units(4))
                    ))
                    .into()
            ])
            .into();

            let mut widgets = Vec::with_capacity(2);

            widgets.push(sidebar.debug(self.debug));

            widgets.push(
                scrollable(row![
                    horizontal_space(Length::Fill),
                    content.debug(self.debug),
                    horizontal_space(Length::Fill),
                ])
                .into(),
            );

            container(row(widgets))
                .padding(12)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        })
        .into();

        column(vec![header, content]).into()
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn theme(&self) -> Theme {
        self.theme
    }
}
