/// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0
use cosmic::{
    iced::widget::{column, container, horizontal_space, row, text},
    iced::{self, Application, Command, Length, Subscription},
    iced_native,
    iced_native::window,
    iced_winit::window::{close, drag, minimize, toggle_maximize},
    theme::{self, Theme},
    widget::{
        header_bar, icon, list, nav_bar, nav_button, scrollable, segmented_button, settings,
        spin_button::{SpinButtonModel, SpinMessage},
    },
    Element, ElementExt,
};
use std::{
    sync::atomic::{AtomicU32, Ordering},
    vec,
};

mod bluetooth;

mod demo;

use self::{demo::DemoView, desktop::DesktopPage};
mod desktop;

use self::input_devices::InputDevicesPage;
mod input_devices;

use self::networking::NetworkingPage;
mod networking;

use self::system_and_accounts::SystemAndAccountsPage;
mod system_and_accounts;

use self::time_and_language::TimeAndLanguagePage;
mod time_and_language;

pub trait SubPage {
    fn title(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn icon_name(&self) -> &'static str;
    fn parent_page(&self) -> Page;
    fn into_page(self) -> Page;
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
        Page::Desktop(None)
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
    bluetooth: bluetooth::State,
    demo: demo::State,
    desktop: desktop::State,
    system_and_accounts: system_and_accounts::State,
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
    Close,
    CondensedViewToggle(()),
    Bluetooth(bluetooth::Message),
    Demo(demo::Message),
    Desktop(desktop::Message),
    Drag,
    InputChanged,
    Maximize,
    Minimize,
    Page(Page),
    ToggleSidebar,
    ToggleSidebarCondensed,
}

impl From<Page> for Message {
    fn from(page: Page) -> Message {
        Message::Page(page)
    }
}

impl Window {
    fn page_title<Message: 'static>(&self, page: Page) -> Element<Message> {
        row!(text(page.title()).size(30), horizontal_space(Length::Fill),).into()
    }

    fn is_condensed(&self) -> bool {
        WINDOW_WIDTH.load(Ordering::Relaxed) < BREAK_POINT
    }

    fn page(&mut self, page: Page) {
        self.sidebar_toggled_condensed = false;
        self.page = page;
    }

    fn parent_page_button<Message: Clone + From<Page> + 'static>(&self, sub_page: impl SubPage) -> Element<Message> {
        let page = sub_page.parent_page();
        column!(
            iced::widget::Button::new(row!(
                icon("go-previous-symbolic", 16).style(theme::Svg::SymbolicLink),
                text(page.title()).size(16),
            ))
            .padding(0)
            .style(theme::Button::Link)
            .on_press(Message::from(page)),
            row!(
                text(sub_page.title()).size(30),
                horizontal_space(Length::Fill),
            ),
        )
        .spacing(10)
        .into()
    }

    fn sub_page_button<Message: Clone + From<Page> + 'static>(&self, sub_page: impl SubPage) -> Element<Message> {
        iced::widget::Button::new(
            container(
                settings::item_row(vec![
                    icon(sub_page.icon_name(), 20)
                        .style(theme::Svg::Symbolic)
                        .into(),
                    column!(
                        text(sub_page.title()).size(18),
                        text(sub_page.description()).size(12),
                    )
                    .spacing(2)
                    .into(),
                    horizontal_space(iced::Length::Fill).into(),
                    icon("go-next-symbolic", 20)
                        .style(theme::Svg::Symbolic)
                        .into(),
                ])
                .spacing(16),
            )
            .padding([20, 24])
            .style(theme::Container::Custom(list::column::style)),
        )
        .padding(0)
        .style(theme::Button::Transparent)
        .on_press(Message::from(sub_page.into_page()))
        .into()
    }

    fn view_unimplemented_page<Message: 'static>(&self, page: Page) -> Element<Message> {
        settings::view_column(vec![
            self.page_title(page),
            text("We haven't created that panel yet, and/or it is using a similar idea as current Pop! designs.").into(),
        ]).into()
    }

    fn view_unimplemented_sub_page<'a, Message: Clone + From<Page> + 'static>(&'a self, sub_page: impl SubPage) -> Element<'a, Message> {
        settings::view_column(vec![
            self.parent_page_button(sub_page),
            text("We haven't created that panel yet, and/or it is using a similar idea as current Pop! designs.").into(),
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
        window.demo.slider_value = 50.0;
        //        window.theme = Theme::Light;
        window.demo.pick_list_selected = Some("Option 1");
        window.title = String::from("COSMIC Design System - Iced");
        window.demo.spin_button.min = -10;
        window.demo.spin_button.max = 10;

        // Configures the demo view switcher.
        let key = window.demo.view_switcher.insert("Controls", DemoView::TabA);
        window.demo.view_switcher.activate(key);
        window.demo.view_switcher.insert("Segmented Button", DemoView::TabB);
        window.demo.view_switcher.insert("Tab C", DemoView::TabC);

        // Configures the demo selection button.
        let key = window.demo.selection.insert("Choice A", ());
        window.demo.selection.activate(key);
        window.demo.selection.insert("Choice B", ());
        window.demo.selection.insert("Choice C", ());


        (window, Command::none())
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events_with(|event, _| match event {
            cosmic::iced::Event::Window(
                _window_id,
                window::Event::Resized { width, height: _ },
            ) => {
                let old_width = WINDOW_WIDTH.load(Ordering::Relaxed);
                if old_width == 0
                    || old_width < BREAK_POINT && width > BREAK_POINT
                    || old_width > BREAK_POINT && width < BREAK_POINT
                {
                    WINDOW_WIDTH.store(width, Ordering::Relaxed);
                    Some(())
                } else {
                    None
                }
            }
            _ => None,
        })
        .map(Message::CondensedViewToggle)
    }

    fn update(&mut self, message: Message) -> iced::Command<Self::Message> {
        match message {
            Message::Page(page) => self.page(page),
            Message::Bluetooth(message) => {
                self.bluetooth.update(message);
            }
            Message::Demo(message) => {
                match self.demo.update(message) {
                    Some(demo::Output::Debug(debug)) => self.debug = debug,
                    Some(demo::Output::ThemeChanged(theme)) => self.theme = theme,
                    None => (),
                }
            }
            Message::Desktop(message) => {
                match self.desktop.update(message) {
                    Some(desktop::Output::Page(page)) => self.page(page),
                    None => (),
                }
            }
            Message::ToggleSidebar => self.sidebar_toggled = !self.sidebar_toggled,
            Message::ToggleSidebarCondensed => {
                self.sidebar_toggled_condensed = !self.sidebar_toggled_condensed
            }
            Message::Drag => return drag(window::Id::new(0)),
            Message::Close => return close(window::Id::new(0)),
            Message::Minimize => return minimize(window::Id::new(0), true),
            Message::Maximize => return toggle_maximize(window::Id::new(0)),

            Message::InputChanged => {}

            Message::CondensedViewToggle(_) => {}

        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let (sidebar_message, sidebar_toggled) = if self.is_condensed() {
            (
                Message::ToggleSidebarCondensed,
                self.sidebar_toggled_condensed,
            )
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
                    .into(),
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
                cosmic::nav_button!(page.icon_name(), page.title(), active)
                    .on_press(Message::Page(page))
            };

            let sidebar_button = |page: Page| sidebar_button_complex(page, self.page == page);

            let mut sidebar = container(scrollable(
                column!(
                    sidebar_button(Page::Demo),
                    sidebar_button(Page::WiFi),
                    sidebar_button_complex(
                        Page::Networking(None),
                        matches!(self.page, Page::Networking(_))
                    ),
                    sidebar_button(Page::Bluetooth),
                    sidebar_button_complex(
                        Page::Desktop(None),
                        matches!(self.page, Page::Desktop(_))
                    ),
                    sidebar_button_complex(
                        Page::InputDevices(None),
                        matches!(self.page, Page::InputDevices(_))
                    ),
                    sidebar_button(Page::Displays),
                    sidebar_button(Page::PowerAndBattery),
                    sidebar_button(Page::Sound),
                    sidebar_button(Page::PrintersAndScanners),
                    sidebar_button(Page::PrivacyAndSecurity),
                    sidebar_button_complex(
                        Page::SystemAndAccounts(None),
                        matches!(self.page, Page::SystemAndAccounts(_))
                    ),
                    sidebar_button(Page::UpdatesAndRecovery),
                    sidebar_button_complex(
                        Page::TimeAndLanguage(None),
                        matches!(self.page, Page::TimeAndLanguage(_))
                    ),
                    sidebar_button(Page::Accessibility),
                    sidebar_button(Page::Applications),
                )
                .spacing(14),
            ))
            .height(Length::Fill)
            .padding(8)
            .style(theme::Container::Custom(nav_bar::nav_bar_sections_style));

            if !self.is_condensed() {
                sidebar = sidebar.max_width(300)
            }

            let sidebar: Element<_> = sidebar.into();
            widgets.push(sidebar.debug(self.debug));
        }

        if !(self.is_condensed() && sidebar_toggled) {
            let content: Element<_> = match self.page {
                Page::Demo => self.demo.view(self).map(Message::Demo),
                Page::Networking(None) => settings::view_column(vec![
                    self.page_title(self.page),
                    column!(
                        self.sub_page_button(NetworkingPage::Wired),
                        self.sub_page_button(NetworkingPage::OnlineAccounts),
                    )
                    .spacing(16)
                    .into(),
                ])
                .into(),
                Page::Networking(Some(sub_page)) => {
                    self.view_unimplemented_sub_page(sub_page)
                }
                Page::Bluetooth => self.bluetooth.view(self).map(Message::Bluetooth),
                Page::Desktop(desktop_page_opt) => self.desktop.view(self, desktop_page_opt).map(Message::Desktop),
                Page::InputDevices(None) => settings::view_column(vec![
                    self.page_title(self.page),
                    column!(
                        self.sub_page_button(InputDevicesPage::Keyboard),
                        self.sub_page_button(InputDevicesPage::Touchpad),
                        self.sub_page_button(InputDevicesPage::Mouse),
                    )
                    .spacing(16)
                    .into(),
                ])
                .into(),
                Page::InputDevices(Some(sub_page)) => self.view_unimplemented_sub_page(sub_page),
                Page::SystemAndAccounts(None) => settings::view_column(vec![
                    self.page_title(self.page),
                    column!(
                        self.sub_page_button(SystemAndAccountsPage::Users),
                        self.sub_page_button(SystemAndAccountsPage::About),
                        self.sub_page_button(SystemAndAccountsPage::Firmware),
                    )
                    .spacing(16)
                    .into(),
                ])
                .into(),
                Page::SystemAndAccounts(Some(SystemAndAccountsPage::About)) => {
                    self.system_and_accounts.view(self)
                }
                Page::SystemAndAccounts(Some(sub_page)) => {
                    self.view_unimplemented_sub_page(sub_page)
                }
                Page::TimeAndLanguage(None) => settings::view_column(vec![
                    self.page_title(self.page),
                    column!(
                        self.sub_page_button(TimeAndLanguagePage::DateAndTime),
                        self.sub_page_button(TimeAndLanguagePage::RegionAndLanguage),
                    )
                    .spacing(16)
                    .into(),
                ])
                .into(),
                Page::TimeAndLanguage(Some(sub_page)) => self.view_unimplemented_sub_page(sub_page),
                _ => self.view_unimplemented_page(self.page),
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
    }

    fn theme(&self) -> Theme {
        self.theme
    }
}
