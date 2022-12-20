// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use cosmic::{
    iced_native::window,
    iced::widget::{
        column, container, horizontal_space, pick_list, progress_bar, radio, row, slider, checkbox, text,
    },
    iced::{self, Alignment, Application, Command, Length},
    iced_lazy::responsive,
    iced_winit::window::{close, drag, toggle_maximize, minimize},
    theme::{self, Theme},
    widget::{button, icon, list, nav_bar, nav_button, header_bar, settings, scrollable, toggler, spin_button::{SpinButtonModel, SpinMessage}},
    Element,
    ElementExt,
};
use std::vec;
use theme::Button as ButtonTheme;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Page {
    Demo,
    WiFi,
    Networking,
    Bluetooth,
    #[default] //TODO: what should the default page be?
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

#[derive(Default)]
pub struct Window {
    title: String,
    page: Page,
    debug: bool,
    theme: Theme,
    slider_value: f32,
    spin_button: SpinButtonModel<i32>,
    checkbox_value: bool,
    toggler_value: bool,
    pick_list_selected: Option<&'static str>,
    sidebar_toggled: bool,
    show_minimize: bool,
    show_maximize: bool,
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
    Page(Page),
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
    InputChanged,
    SpinButton(SpinMessage)
}

impl Window {
    fn view_demo(&self) -> Element<Message> {
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

        settings::view_column(vec![
            text("Demo").size(32).into(),
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
                .add(settings::item_row(vec![
                    checkbox("Checkbox", self.checkbox_value, Message::CheckboxToggled).into()
                ]))
                .add(settings::item(
                    format!("Spin Button (Range {}:{})", self.spin_button.min, self.spin_button.max),
                    self.spin_button.view(Message::SpinButton),
                ))
                .into()
        ])
        .into()
    }

    fn view_desktop(&self) -> Element<Message> {
        settings::view_column(vec![
            text("Desktop").size(32).into(),

            //TODO: simplify these buttons!

            container(settings::item_row(vec![
                icon("video-display-symbolic", 16).style(theme::Svg::Symbolic).into(),
                column!(
                    text("Desktop Options").size(16),
                    text("Super Key action, hot corners, window control options.").size(12),
                ).into(),
                horizontal_space(iced::Length::Fill).into(),
                icon("go-next-symbolic", 16).style(theme::Svg::Symbolic).into(),
            ]).spacing(16))
            .padding([20, 32])
            .style(theme::Container::Custom(list::column::style))
            .into(),

            container(settings::item_row(vec![
                icon("preferences-desktop-wallpaper-symbolic", 16).style(theme::Svg::Symbolic).into(),
                column!(
                    text("Wallpaper").size(16),
                    text("Background images, colors, and slideshow options.").size(12),
                ).into(),
                horizontal_space(iced::Length::Fill).into(),
                icon("go-next-symbolic", 16).style(theme::Svg::Symbolic).into(),
            ]).spacing(16))
            .padding([20, 32])
            .style(theme::Container::Custom(list::column::style))
            .into(),

            container(settings::item_row(vec![
                icon("preferences-pop-desktop-appearance-symbolic", 16).style(theme::Svg::Symbolic).into(),
                column!(
                    text("Appearance").size(16),
                    text("Accent colors and COSMIC theming").size(12),
                ).into(),
                horizontal_space(iced::Length::Fill).into(),
                icon("go-next-symbolic", 16).style(theme::Svg::Symbolic).into(),
            ]).spacing(16))
            .padding([20, 32])
            .style(theme::Container::Custom(list::column::style))
            .into(),

            container(settings::item_row(vec![
                icon("preferences-pop-desktop-dock-symbolic", 16).style(theme::Svg::Symbolic).into(),
                column!(
                    text("Dock & Top Panel").size(16),
                    text("Customize size, positions, and more for Dock and Top Panel.").size(12),
                ).into(),
                horizontal_space(iced::Length::Fill).into(),
                icon("go-next-symbolic", 16).style(theme::Svg::Symbolic).into(),
            ]).spacing(16))
            .padding([20, 32])
            .style(theme::Container::Custom(list::column::style))
            .into(),

            container(settings::item_row(vec![
                icon("preferences-pop-desktop-workspaces-symbolic", 16).style(theme::Svg::Symbolic).into(),
                column!(
                    text("Workspaces").size(16),
                    text("Set workspace number, behavior, and placement.").size(12),
                ).into(),
                horizontal_space(iced::Length::Fill).into(),
                icon("go-next-symbolic", 16).style(theme::Svg::Symbolic).into(),
            ]).spacing(16))
            .padding([20, 32])
            .style(theme::Container::Custom(list::column::style))
            .into(),
        ])
        .into()
    }
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
        window.spin_button.min = -10;
        window.spin_button.max = 10;
        (window, Command::none())
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn update(&mut self, message: Message) -> iced::Command<Self::Message> {
        match message {
            Message::Page(page) => self.page = page,
            Message::Debug(debug) => self.debug = debug,
            Message::ThemeChanged(theme) => self.theme = theme,
            Message::ButtonPressed => {}
            Message::SliderChanged(value) => self.slider_value = value,
            Message::CheckboxToggled(value) => {
                self.checkbox_value = value;
            },
            Message::TogglerToggled(value) => self.toggler_value = value,
            Message::PickListSelected(value) => self.pick_list_selected = Some(value),
            Message::ToggleSidebar => self.sidebar_toggled = !self.sidebar_toggled,
            Message::Drag => return drag(window::Id::new(0)),
            Message::Close => return close(window::Id::new(0)),
            Message::Minimize => return minimize(window::Id::new(0), true),
            Message::Maximize => return toggle_maximize(window::Id::new(0)),
            Message::RowSelected(row) => println!("Selected row {row}"),
            Message::InputChanged => {},
            Message::SpinButton(msg) => self.spin_button.update(msg),

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

            let sidebar: Element<_> = container(scrollable(column!(
                cosmic::nav_button!("document-properties-symbolic", "Demo", condensed, self.page == Page::Demo)
                    .on_press(Message::Page(Page::Demo)),
                cosmic::nav_button!("network-wireless-symbolic", "Wi-Fi", condensed, self.page == Page::WiFi)
                    .on_press(Message::Page(Page::WiFi)),
                cosmic::nav_button!("network-workgroup-symbolic", "Networking", condensed, self.page == Page::Networking)
                    .on_press(Message::Page(Page::Networking)),
                cosmic::nav_button!("bluetooth-active-symbolic", "Bluetooth", condensed, self.page == Page::Bluetooth)
                    .on_press(Message::Page(Page::Bluetooth)),
                cosmic::nav_button!("video-display-symbolic", "Desktop", condensed, self.page == Page::Desktop)
                    .on_press(Message::Page(Page::Desktop)),
                cosmic::nav_button!("input-keyboard-symbolic", "Input Devices", condensed, self.page == Page::InputDevices)
                    .on_press(Message::Page(Page::InputDevices)),
                cosmic::nav_button!("preferences-desktop-display-symbolic", "Displays", condensed, self.page == Page::Displays)
                    .on_press(Message::Page(Page::Displays)),
                cosmic::nav_button!("battery-full-charged-symbolic", "Power & Battery", condensed, self.page == Page::PowerAndBattery)
                    .on_press(Message::Page(Page::PowerAndBattery)),
                cosmic::nav_button!("multimedia-volume-control-symbolic", "Sound", condensed, self.page == Page::Sound)
                    .on_press(Message::Page(Page::Sound)),
                cosmic::nav_button!("printer-symbolic", "Printers & Scanners", condensed, self.page == Page::PrintersAndScanners)
                    .on_press(Message::Page(Page::PrintersAndScanners)),
                cosmic::nav_button!("preferences-system-privacy-symbolic", "Privacy & Security", condensed, self.page == Page::PrivacyAndSecurity)
                    .on_press(Message::Page(Page::PrivacyAndSecurity)),
                cosmic::nav_button!("system-users-symbolic", "System & Accounts", condensed, self.page == Page::SystemAndAccounts)
                    .on_press(Message::Page(Page::SystemAndAccounts)),
                cosmic::nav_button!("software-update-available-symbolic", "Updates & Recovery", condensed, self.page == Page::UpdatesAndRecovery)
                    .on_press(Message::Page(Page::UpdatesAndRecovery)),
                cosmic::nav_button!("preferences-system-time-symbolic", "Time & Language", condensed, self.page == Page::TimeAndLanguage)
                    .on_press(Message::Page(Page::TimeAndLanguage)),
                cosmic::nav_button!("preferences-desktop-accessibility-symbolic", "Accessibility", condensed, self.page == Page::Accessibility)
                    .on_press(Message::Page(Page::Accessibility)),
                cosmic::nav_button!("preferences-desktop-apps-symbolic", "Applications", condensed, self.page == Page::Applications)
                    .on_press(Message::Page(Page::Applications)),
            ).spacing(14)))
            .height(Length::Fill)
            .padding(11)
            .max_width(300)
            .style(theme::Container::Custom(nav_bar::nav_bar_sections_style))
            .into();

            let content: Element<_> = match self.page {
                Page::Demo => self.view_demo(),
                Page::Desktop => self.view_desktop(),
                _ =>  settings::view_column(vec![
                    text("Unimplemented page").into()
                ]).into(),
            };

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
                .padding([16, 16])
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        })
        .into();

        column(vec![header, content]).into()
    }

    fn theme(&self) -> Theme {
        self.theme
    }
}
