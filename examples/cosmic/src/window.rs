/// Copyright 2022 System76 <info@system76.com>
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
    widget::{button, icon, list, list_column, nav_bar, nav_button, header_bar, settings, scrollable, toggler, spin_button::{SpinButtonModel, SpinMessage}},
    Element,
    ElementExt,
};
use std::vec;
use theme::Button as ButtonTheme;

pub trait SubPage {
    fn title(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn icon_name(&self) -> &'static str;
    fn parent_page(&self) -> Page;
    fn into_page(self) -> Page;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NetworkingPage {
    Wired,
    OnlineAccounts,
}

impl SubPage for NetworkingPage {
    //TODO: translate
    fn title(&self) -> &'static str {
        use NetworkingPage::*;
        match self {
            Wired => "Wired",
            OnlineAccounts => "Online Accounts",
        }
    }

    //TODO: translate
    fn description(&self) -> &'static str {
        use NetworkingPage::*;
        match self {
            Wired => "Wired connection, connection profiles",
            OnlineAccounts => "Add accounts, IMAP and SMTP, enterprise logins",
        }
    }

    fn icon_name(&self) -> &'static str {
        use NetworkingPage::*;
        match self {
            Wired => "network-workgroup-symbolic",
            OnlineAccounts => "goa-panel-symbolic", //TODO: new icon
        }
    }

    fn parent_page(&self) -> Page {
        Page::Networking(None)
    }

    fn into_page(self) -> Page {
        Page::Networking(Some(self))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DesktopPage {
    DesktopOptions,
    Wallpaper,
    Appearance,
    DockAndTopPanel,
    Workspaces,
    Notifications,
}

impl SubPage for DesktopPage {
    //TODO: translate
    fn title(&self) -> &'static str {
        use DesktopPage::*;
        match self {
            DesktopOptions => "Desktop Options",
            Wallpaper => "Wallpaper",
            Appearance => "Appearance",
            DockAndTopPanel => "Dock & Top Panel",
            Workspaces => "Workspaces",
            Notifications => "Notifications",
        }
    }

    //TODO: translate
    fn description(&self) -> &'static str {
        use DesktopPage::*;
        match self {
            DesktopOptions => "Super Key action, hot corners, window control options.",
            Wallpaper => "Background images, colors, and slideshow options.",
            Appearance => "Accent colors and COSMIC theming",
            DockAndTopPanel => "Customize size, positions, and more for Dock and Top Panel.",
            Workspaces => "Set workspace number, behavior, and placement.",
            Notifications => "Do Not Disturb, lockscreen notifications, and per-application settings.",
        }
    }

    fn icon_name(&self) -> &'static str {
        use DesktopPage::*;
        match self {
            DesktopOptions => "video-display-symbolic",
            Wallpaper => "preferences-desktop-wallpaper-symbolic",
            Appearance => "preferences-pop-desktop-appearance-symbolic",
            DockAndTopPanel => "preferences-pop-desktop-dock-symbolic",
            Workspaces => "preferences-pop-desktop-workspaces-symbolic",
            Notifications => "preferences-system-notifications-symbolic",
        }
    }

    fn parent_page(&self) -> Page {
        Page::Desktop(None)
    }

    fn into_page(self) -> Page {
        Page::Desktop(Some(self))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputDevicesPage {
    Keyboard,
    Touchpad,
    Mouse,
}

impl SubPage for InputDevicesPage {
    //TODO: translate
    fn title(&self) -> &'static str {
        use InputDevicesPage::*;
        match self {
            Keyboard => "Keyboard",
            Touchpad => "Touchpad",
            Mouse => "Mouse",
        }
    }

    //TODO: translate
    fn description(&self) -> &'static str {
        use InputDevicesPage::*;
        match self {
            Keyboard => "Input sources, switching, special character entry, shortcuts.",
            Touchpad => "Touchpad speed, click options, gestures.",
            Mouse => "Mouse speed, acceleration, natural scrolling.",
        }
    }

    fn icon_name(&self) -> &'static str {
        use InputDevicesPage::*;
        match self {
            Keyboard => "input-keyboard-symbolic",
            Touchpad => "input-touchpad-symbolic",
            Mouse => "input-mouse-symbolic",
        }
    }

    fn parent_page(&self) -> Page {
        Page::InputDevices(None)
    }

    fn into_page(self) -> Page {
        Page::InputDevices(Some(self))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemAndAccountsPage {
    Users,
    About,
    Firmware,
}

impl SubPage for SystemAndAccountsPage {
    //TODO: translate
    fn title(&self) -> &'static str {
        use SystemAndAccountsPage::*;
        match self {
            Users => "Users",
            About => "About",
            Firmware => "Firmware",
        }
    }

    //TODO: translate
    fn description(&self) -> &'static str {
        use SystemAndAccountsPage::*;
        match self {
            Users => "Authentication and login, lock screen.",
            About => "Device name, hardware information, operating system defaults.",
            Firmware => "Firmware details.",
        }
    }

    fn icon_name(&self) -> &'static str {
        use SystemAndAccountsPage::*;
        match self {
            Users => "system-users-symbolic",
            About => "help-about-symbolic",
            Firmware => "firmware-manager-symbolic",
        }
    }

    fn parent_page(&self) -> Page {
        Page::SystemAndAccounts(None)
    }

    fn into_page(self) -> Page {
        Page::SystemAndAccounts(Some(self))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeAndLanguagePage {
    DateAndTime,
    RegionAndLanguage,
}

impl SubPage for TimeAndLanguagePage {
    //TODO: translate
    fn title(&self) -> &'static str {
        use TimeAndLanguagePage::*;
        match self {
            DateAndTime => "Date & Time",
            RegionAndLanguage => "Region & Language",
        }
    }

    //TODO: translate
    fn description(&self) -> &'static str {
        use TimeAndLanguagePage::*;
        match self {
            DateAndTime => "Time zone, automatic clock settings, and some time formatting.",
            RegionAndLanguage => "Format dates, times, and numbers based on your region",
        }
    }

    fn icon_name(&self) -> &'static str {
        use TimeAndLanguagePage::*;
        match self {
            DateAndTime => "preferences-system-time-symbolic",
            RegionAndLanguage => "preferences-desktop-locale-symbolic",
        }
    }

    fn parent_page(&self) -> Page {
        Page::TimeAndLanguage(None)
    }

    fn into_page(self) -> Page {
        Page::TimeAndLanguage(Some(self))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Page {
    Demo,
    WiFi,
    Networking(Option<NetworkingPage>),
    Bluetooth,
    Desktop(Option<DesktopPage>),
    InputDevices(Option<InputDevicesPage>),
    Displays,
    PowerAndBattery,
    Sound,
    PrintersAndScanners,
    PrivacyAndSecurity,
    SystemAndAccounts(Option<SystemAndAccountsPage>),
    UpdatesAndRecovery,
    TimeAndLanguage(Option<TimeAndLanguagePage>),
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
            Networking(_) => "Networking",
            Bluetooth => "Bluetooth",
            Desktop(_) => "Desktop",
            InputDevices(_) => "Input Devices",
            Displays => "Displays",
            PowerAndBattery => "Power & Battery",
            Sound => "Sound",
            PrintersAndScanners => "Printers & Scanners",
            PrivacyAndSecurity => "Privacy & Security",
            SystemAndAccounts(_) => "System & Accounts",
            UpdatesAndRecovery => "Updates & Recovery",
            TimeAndLanguage(_) => "Time & Language",
            Accessibility => "Accessibility",
            Applications => "Applications",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        use Page::*;
        match self {
            Demo => "document-properties-symbolic",
            WiFi => "network-wireless-symbolic",
            Networking(_) => "network-workgroup-symbolic",
            Bluetooth => "bluetooth-active-symbolic",
            Desktop(_) => "video-display-symbolic",
            InputDevices(_) => "input-keyboard-symbolic",
            Displays => "preferences-desktop-display-symbolic",
            PowerAndBattery => "battery-full-charged-symbolic",
            Sound => "multimedia-volume-control-symbolic",
            PrintersAndScanners => "printer-symbolic",
            PrivacyAndSecurity => "preferences-system-privacy-symbolic",
            SystemAndAccounts(_) => "system-users-symbolic",
            UpdatesAndRecovery => "software-update-available-symbolic",
            TimeAndLanguage(_) => "preferences-system-time-symbolic",
            Accessibility => "preferences-desktop-accessibility-symbolic",
            Applications => "preferences-desktop-apps-symbolic",
        }
    }
}

impl Default for Page {
    fn default() -> Page {
        //TODO: what should the default page be?
        Page::SystemAndAccounts(Some(SystemAndAccountsPage::About))
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
    fn page_title(&self, page: Page) -> Element<Message> {
        row!(
            text(page.title()).size(30),
            horizontal_space(Length::Fill),
        ).into()
    }

    fn parent_page_button(&self, sub_page: impl SubPage) -> Element<Message> {
        let page = sub_page.parent_page();
        column!(
            iced::widget::Button::new(row!(
                icon("go-previous-symbolic", 16).style(theme::Svg::SymbolicLink),
                text(page.title()).size(16),
            ))
            .padding(0)
            .style(theme::Button::Link)
            .on_press(Message::Page(page)),

            row!(
                text(sub_page.title()).size(30),
                horizontal_space(Length::Fill),
            ),
        )
        .spacing(10)
        .into()
    }

    fn sub_page_button(&self, sub_page: impl SubPage) -> Element<Message> {
        iced::widget::Button::new(
            container(settings::item_row(vec![
                icon(sub_page.icon_name(), 20).style(theme::Svg::Symbolic).into(),
                column!(
                    text(sub_page.title()).size(18),
                    text(sub_page.description()).size(12),
                ).spacing(2).into(),
                horizontal_space(iced::Length::Fill).into(),
                icon("go-next-symbolic", 20).style(theme::Svg::Symbolic).into(),
            ]).spacing(16))
            .padding([20, 24])
            .style(theme::Container::Custom(list::column::style))
        )
        .padding(0)
        .style(theme::Button::Transparent)
        .on_press(Message::Page(sub_page.into_page()))
        .into()
    }

    fn view_unimplemented_page(&self, page: Page) -> Element<Message> {
        settings::view_column(vec![
            self.page_title(page),
            text("We haven't created that panel yet, and/or it is using a similar idea as current Pop! designs.").into(),
        ]).into()
    }

    fn view_unimplemented_sub_page(&self, sub_page: impl SubPage) -> Element<Message> {
        settings::view_column(vec![
            self.parent_page_button(sub_page),
            text("We haven't created that panel yet, and/or it is using a similar idea as current Pop! designs.").into(),
        ]).into()
    }

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
            self.page_title(Page::Demo),

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

    fn view_bluetooth(&self) -> Element<Message> {
        settings::view_column(vec![
            self.page_title(Page::Bluetooth),

            column!(
                list_column()
                    .add(settings::item("Bluetooth", toggler(None, self.toggler_value, Message::TogglerToggled))),
                text("Now visible as \"TODO\", just kidding")
            ).spacing(8).into(),

            settings::view_section("Devices")
                .add(settings::item("No devices found", text("")))
                .into()
        ]).into()
    }

    fn view_desktop_options(&self) -> Element<Message> {
        settings::view_column(vec![
            self.parent_page_button(DesktopPage::DesktopOptions),

            settings::view_section("Super Key Action")
                .add(settings::item("TODO", horizontal_space(Length::Fill)))
                .into(),

            settings::view_section("Hot Corner")
                .add(settings::item("Enable top-left hot corner for Workspaces", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .into(),

            settings::view_section("Top Panel")
                .add(settings::item("Show Workspaces Button", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .add(settings::item("Show Applications Button", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .into(),

            settings::view_section("Window Controls")
                .add(settings::item("Show Minimize Button", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .add(settings::item("Show Maximize Button", toggler(None, self.toggler_value, Message::TogglerToggled)))
                .into(),
        ]).into()
    }

    fn view_system_and_accounts_about(&self) -> Element<Message> {
        settings::view_column(vec![
            self.parent_page_button(SystemAndAccountsPage::About),

            row!(
                horizontal_space(Length::Fill),
                icon("distributor-logo", 78),
                horizontal_space(Length::Fill),
            ).into(),

            list_column()
                .add(settings::item("Device name", text("TODO")))
                .into(),

            settings::view_section("Hardware")
                .add(settings::item("Hardware model", text("TODO")))
                .add(settings::item("Memory", text("TODO")))
                .add(settings::item("Processor", text("TODO")))
                .add(settings::item("Graphics", text("TODO")))
                .add(settings::item("Disk Capacity", text("TODO")))
                .into(),

            settings::view_section("Operating System")
                .add(settings::item("Operating system", text("TODO")))
                .add(settings::item("Operating system architecture", text("TODO")))
                .add(settings::item("Desktop environment", text("TODO")))
                .add(settings::item("Windowing system", text("TODO")))
                .into(),

            settings::view_section("Related settings")
                .add(settings::item("Get support", text("TODO")))
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
                let sidebar_button_complex = |page: Page, active| {
                    cosmic::nav_button!(
                        page.icon_name(),
                        page.title(),
                        active
                    )
                    .on_press(Message::Page(page))
                };

                let sidebar_button = |page: Page| {
                    sidebar_button_complex(page, self.page == page)
                };

                let mut sidebar = container(scrollable(column!(
                    sidebar_button(Page::Demo),
                    sidebar_button(Page::WiFi),
                    sidebar_button_complex(Page::Networking(None), matches!(self.page, Page::Networking(_))),
                    sidebar_button(Page::Bluetooth),
                    sidebar_button_complex(Page::Desktop(None), matches!(self.page, Page::Desktop(_))),
                    sidebar_button_complex(Page::InputDevices(None), matches!(self.page, Page::InputDevices(_))),
                    sidebar_button(Page::Displays),
                    sidebar_button(Page::PowerAndBattery),
                    sidebar_button(Page::Sound),
                    sidebar_button(Page::PrintersAndScanners),
                    sidebar_button(Page::PrivacyAndSecurity),
                    sidebar_button_complex(Page::SystemAndAccounts(None), matches!(self.page, Page::SystemAndAccounts(_))),
                    sidebar_button(Page::UpdatesAndRecovery),
                    sidebar_button_complex(Page::TimeAndLanguage(None), matches!(self.page, Page::TimeAndLanguage(_))),
                    sidebar_button(Page::Accessibility),
                    sidebar_button(Page::Applications),
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
                    Page::Networking(None) => settings::view_column(vec![
                        self.page_title(self.page),
                        column!(
                            self.sub_page_button(NetworkingPage::Wired),
                            self.sub_page_button(NetworkingPage::OnlineAccounts),
                        ).spacing(16).into()
                    ]).into(),
                    Page::Networking(Some(sub_page)) => self.view_unimplemented_sub_page(sub_page),
                    Page::Bluetooth => self.view_bluetooth(),
                    Page::Desktop(None) => settings::view_column(vec![
                        self.page_title(self.page),
                        column!(
                            self.sub_page_button(DesktopPage::DesktopOptions),
                            self.sub_page_button(DesktopPage::Wallpaper),
                            self.sub_page_button(DesktopPage::Appearance),
                            self.sub_page_button(DesktopPage::DockAndTopPanel),
                            self.sub_page_button(DesktopPage::Workspaces),
                            self.sub_page_button(DesktopPage::Notifications),
                        ).spacing(16).into()
                    ]).into(),
                    Page::Desktop(Some(DesktopPage::DesktopOptions)) => self.view_desktop_options(),
                    Page::Desktop(Some(sub_page)) => self.view_unimplemented_sub_page(sub_page),
                    Page::InputDevices(None) => settings::view_column(vec![
                        self.page_title(self.page),
                        column!(
                            self.sub_page_button(InputDevicesPage::Keyboard),
                            self.sub_page_button(InputDevicesPage::Touchpad),
                            self.sub_page_button(InputDevicesPage::Mouse),
                        ).spacing(16).into()
                    ]).into(),
                    Page::InputDevices(Some(sub_page)) => self.view_unimplemented_sub_page(sub_page),
                    Page::SystemAndAccounts(None) => settings::view_column(vec![
                        self.page_title(self.page),
                        column!(
                            self.sub_page_button(SystemAndAccountsPage::Users),
                            self.sub_page_button(SystemAndAccountsPage::About),
                            self.sub_page_button(SystemAndAccountsPage::Firmware),
                        ).spacing(16).into()
                    ]).into(),
                    Page::SystemAndAccounts(Some(SystemAndAccountsPage::About)) => self.view_system_and_accounts_about(),
                    Page::SystemAndAccounts(Some(sub_page)) => self.view_unimplemented_sub_page(sub_page),
                    Page::TimeAndLanguage(None) => settings::view_column(vec![
                        self.page_title(self.page),
                        column!(
                            self.sub_page_button(TimeAndLanguagePage::DateAndTime),
                            self.sub_page_button(TimeAndLanguagePage::RegionAndLanguage),
                        ).spacing(16).into()
                    ]).into(),
                    Page::TimeAndLanguage(Some(sub_page)) => self.view_unimplemented_sub_page(sub_page),
                    _ =>  self.view_unimplemented_page(self.page),
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
