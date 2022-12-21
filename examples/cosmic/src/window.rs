/// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use cosmic::{
    iced_native::window,
    iced::widget::{column, container, horizontal_space, row, text},
    iced::{self, Application, Command, Length},
    iced_lazy::responsive,
    iced_winit::window::{close, drag, toggle_maximize, minimize},
    theme::{self, Theme},
    widget::{icon, list, nav_bar, nav_button, header_bar, settings, scrollable, spin_button::{SpinButtonModel, SpinMessage}},
    Element,
    ElementExt,
};
use std::vec;

mod bluetooth;

mod demo;

use self::desktop::DesktopPage;
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
                    Page::Desktop(desktop_page_opt) => self.view_desktop(desktop_page_opt),
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
