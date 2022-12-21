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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopPage {
    Root,
    DesktopOptions,
    Wallpaper,
    Appearance,
    DockAndTopPanel,
    Workspaces,
    Notifications,
}

impl DesktopPage {
    //TODO: translate
    pub fn title(&self) -> &'static str {
        use DesktopPage::*;
        match self {
            Root => "Desktop",
            DesktopOptions => "Desktop Options",
            Wallpaper => "Wallpaper",
            Appearance => "Appearance",
            DockAndTopPanel => "Dock & Top Panel",
            Workspaces => "Workspaces",
            Notifications => "Notifications",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Page {
    Demo,
    WiFi,
    Networking,
    Bluetooth,
    Desktop(DesktopPage),
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
            Desktop(_) => "Desktop",
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
}

impl Default for Page {
    fn default() -> Page {
        //TODO: what should the default page be?
        Page::Desktop(DesktopPage::Root)
    }
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
    sidebar_toggled_condensed: bool,
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
    ToggleSidebarCondensed,
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
            text("Demo").size(30).into(),
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

    fn view_desktop(&self, desktop_page: DesktopPage) -> Element<Message> {
        match desktop_page {
            DesktopPage::Root => self.view_desktop_root(),
            DesktopPage::DesktopOptions => self.view_desktop_options(),
            _ =>  settings::view_column(vec![
                column!(
                    iced::widget::Button::new(row!(
                        icon("go-previous-symbolic", 16).style(theme::Svg::SymbolicLink),
                        text("Desktop").size(16),
                    ))
                    .padding(0)
                    .style(theme::Button::Link)
                    .on_press(Message::Page(Page::Desktop(DesktopPage::Root))),

                    row!(
                        text(desktop_page.title()).size(30),
                        horizontal_space(Length::Fill),
                    ),
                )
                .spacing(10)
                .into(),

                text("Unimplemented desktop page").into(),
            ]).into(),
        }
    }

    fn view_desktop_root(&self) -> Element<Message> {
        //TODO: rename and move to libcosmic
        let desktop_page_button = |desktop_page: DesktopPage, icon_name, description| {
            iced::widget::Button::new(
                container(settings::item_row(vec![
                    icon(icon_name, 20).style(theme::Svg::Symbolic).into(),
                    column!(
                        text(desktop_page.title()).size(18),
                        text(description).size(12),
                    ).spacing(2).into(),
                    horizontal_space(iced::Length::Fill).into(),
                    icon("go-next-symbolic", 20).style(theme::Svg::Symbolic).into(),
                ]).spacing(16))
                .padding([20, 24])
                .style(theme::Container::Custom(list::column::style))
            )
            .padding(0)
            .style(theme::Button::Transparent)
            .on_press(Message::Page(Page::Desktop(desktop_page)))
        };

        settings::view_column(vec![
            text("Desktop").size(30).into(),

            //TODO: simplify these buttons!
            column!(
                desktop_page_button(
                    DesktopPage::DesktopOptions,
                    "video-display-symbolic",
                    "Super Key action, hot corners, window control options.",
                ),

                desktop_page_button(
                    DesktopPage::Wallpaper,
                    "preferences-desktop-wallpaper-symbolic",
                    "Background images, colors, and slideshow options.",
                ),

                desktop_page_button(
                    DesktopPage::Appearance,
                    "preferences-pop-desktop-appearance-symbolic",
                    "Accent colors and COSMIC theming",
                ),

                desktop_page_button(
                    DesktopPage::DockAndTopPanel,
                    "preferences-pop-desktop-dock-symbolic",
                    "Customize size, positions, and more for Dock and Top Panel.",
                ),

                desktop_page_button(
                    DesktopPage::Workspaces,
                    "preferences-pop-desktop-workspaces-symbolic",
                    "Set workspace number, behavior, and placement.",
                ),

                desktop_page_button(
                    DesktopPage::Notifications,
                    "preferences-system-notifications-symbolic",
                    "Do Not Disturb, lockscreen notifications, and per-application settings.",
                ),
            ).spacing(16).into()
        ])
        .into()
    }

    fn view_desktop_options(&self) -> Element<Message> {
        settings::view_column(vec![
            column!(
                iced::widget::Button::new(row!(
                    icon("go-previous-symbolic", 16).style(theme::Svg::SymbolicLink),
                    text("Desktop").size(16),
                ))
                .padding(0)
                .style(theme::Button::Link)
                .on_press(Message::Page(Page::Desktop(DesktopPage::Root))),
                text("Desktop Options").size(30),
            )
            .spacing(10)
            .into(),

            settings::view_section("Super Key Action")
                .add(settings::item("TODO", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .into(),

            settings::view_section("Hot Corner")
                .add(settings::item("Enable top-left hot corner for Workspaces", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .into(),

            settings::view_section("Top Panel")
                .add(settings::item("Show Workspaces Button", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .add(settings::item("Show Applications Button", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .into(),
        ]).into()
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
            Message::Page(page) => {
                self.sidebar_toggled_condensed = false;
                self.page = page;
            },
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
            Message::ToggleSidebarCondensed => self.sidebar_toggled_condensed = !self.sidebar_toggled_condensed,
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
        // TODO: Adding responsive makes this regenerate on every size change, and regeneration
        // involves allocations for many different items. Ideally, we could only make the nav bar
        // responsive and leave the content to be sized normally.
        responsive(|size| {
            //TODO: send a message when this happens instead of having everything be recalculated on resize
            let condensed = size.width < 900.0;

            let (sidebar_message, sidebar_toggled) = if condensed {
                (Message::ToggleSidebarCondensed, self.sidebar_toggled_condensed)
            } else {
                (Message::ToggleSidebar, self.sidebar_toggled)
            };

            let mut header = header_bar()
                .title("COSMIC Design System - Iced")
                .on_close(Message::Close)
                .on_drag(Message::Drag)
                .start(
                    nav_button("Settings")
                        .on_sidebar_toggled(sidebar_message)
                        .sidebar_active(sidebar_toggled)
                        .into()
                );

            if self.show_maximize {
                header = header.on_maximize(Message::Maximize);
            }

            if self.show_minimize {
                header = header.on_minimize(Message::Minimize);
            }

            let header = Into::<Element<Message>>::into(header).debug(self.debug);

            let mut widgets = Vec::with_capacity(2);

            if sidebar_toggled {
                let sidebar_button_complex = |page: Page, icon_name, active| {
                    cosmic::nav_button!(
                        icon_name,
                        page.title(),
                        active
                    )
                    .on_press(Message::Page(page))
                };

                let sidebar_button = |page: Page, icon_name| {
                    sidebar_button_complex(page, icon_name, self.page == page)
                };

                let mut sidebar = container(scrollable(column!(
                    sidebar_button(Page::Demo, "document-properties-symbolic"),
                    sidebar_button(Page::Networking, "network-wireless-symbolic"),
                    sidebar_button(Page::Bluetooth, "bluetooth-active-symbolic"),
                    sidebar_button_complex(Page::Desktop(DesktopPage::Root), "video-display-symbolic", matches!(self.page, Page::Desktop(_))),
                    sidebar_button(Page::InputDevices, "input-keyboard-symbolic"),
                    sidebar_button(Page::Displays, "preferences-desktop-display-symbolic"),
                    sidebar_button(Page::PowerAndBattery, "battery-full-charged-symbolic"),
                    sidebar_button(Page::Sound, "multimedia-volume-control-symbolic"),
                    sidebar_button(Page::PrintersAndScanners, "printer-symbolic"),
                    sidebar_button(Page::PrivacyAndSecurity, "preferences-system-privacy-symbolic"),
                    sidebar_button(Page::SystemAndAccounts, "system-users-symbolic"),
                    sidebar_button(Page::UpdatesAndRecovery, "software-update-available-symbolic"),
                    sidebar_button(Page::TimeAndLanguage, "preferences-system-time-symbolic"),
                    sidebar_button(Page::Accessibility, "preferences-desktop-accessibility-symbolic"),
                    sidebar_button(Page::Applications, "preferences-desktop-apps-symbolic"),
                ).spacing(14)))
                .height(Length::Fill)
                .padding(8)
                .style(theme::Container::Custom(nav_bar::nav_bar_sections_style));

                if ! condensed {
                    sidebar = sidebar.max_width(300)
                }

                let sidebar: Element<_> = sidebar.into();
                widgets.push(sidebar.debug(self.debug));
            }

            if ! (condensed && sidebar_toggled) {
                let content: Element<_> = match self.page {
                    Page::Demo => self.view_demo(),
                    Page::Desktop(desktop_page) => self.view_desktop(desktop_page),
                    _ =>  settings::view_column(vec![
                        row!(
                            text(self.page.title()).size(30),
                            horizontal_space(Length::Fill),
                        ).into(),
                        text("Unimplemented page").into(),
                    ]).into(),
                };

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
                .into();

            column(vec![header, content]).into()
        })
        .into()
    }

    fn theme(&self) -> Theme {
        self.theme
    }
}
