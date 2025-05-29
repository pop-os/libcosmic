/// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0
use cosmic::{
    cosmic_theme::{
        palette::{rgb::Rgb, Srgba},
        ThemeBuilder,
    },
    font::load_fonts,
    iced::{self, Length, Subscription, Task},
    iced::{
        subscription,
        widget::{self, column, horizontal_space, row, text},
        window::{self, close, drag, minimize, toggle_maximize},
    },
    iced_futures::event::listen_raw,
    keyboard_nav,
    prelude::*,
    theme::{self, Theme},
    widget::{
        button, container, header_bar, icon, nav_bar, nav_bar_toggle, scrollable, segmented_button,
        settings, warning,
    },
    Application, Element,
};
use cosmic_time::{Instant, Timeline};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    vec,
};

// XXX The use of button is removed because it assigns the same ID to multiple buttons, causing a crash when a11y is enabled...
// static BTN: Lazy<id::Id> = Lazy::new(|| id::Id::new("BTN"));

mod bluetooth;

mod demo;

use self::desktop::DesktopPage;
mod desktop;

mod editor;

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
    Editor,
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
            Editor => "Editor",
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
            Editor => "text-editor-symbolic",
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
    bluetooth: bluetooth::State,
    debug: bool,
    demo: demo::State,
    editor: editor::State,
    desktop: desktop::State,
    nav_bar: segmented_button::SingleSelectModel,
    nav_id_to_page: segmented_button::SecondaryMap<Page>,
    nav_bar_toggled_condensed: bool,
    nav_bar_toggled: bool,
    page: Page,
    show_maximize: bool,
    show_minimize: bool,
    system_and_accounts: system_and_accounts::State,
    theme: Theme,
    title: String,
    show_warning: bool,
    warning_message: String,
    scale_factor: f64,
    scale_factor_string: String,
    timeline: Rc<RefCell<Timeline>>,
}

impl Window {
    pub fn nav_bar_toggled(mut self, toggled: bool) -> Self {
        self.nav_bar_toggled = toggled;
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

    pub fn show_warning(mut self, show: bool) -> Self {
        self.show_warning = show;
        self
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Message {
    Bluetooth(bluetooth::Message),
    Close,
    CondensedViewToggle,
    Demo(demo::Message),
    Desktop(desktop::Message),
    Drag,
    Editor(editor::Message),
    InputChanged,
    KeyboardNav(keyboard_nav::Message),
    Maximize,
    Minimize,
    NavBar(segmented_button::Entity),
    Page(Page),
    ToggleNavBar,
    ToggleNavBarCondensed,
    ToggleWarning,
    FontsLoaded,
    Tick(Instant),
}

impl From<Page> for Message {
    fn from(page: Page) -> Message {
        Message::Page(page)
    }
}

impl Window {
    /// Adds a page to the model we use for the navigation bar.
    fn insert_page(&mut self, page: Page) -> segmented_button::SingleSelectEntityMut {
        self.nav_bar
            .insert()
            .text(page.title())
            .icon(icon::from_name(page.icon_name()).icon())
            .secondary(&mut self.nav_id_to_page, page)
    }

    fn page_title<Message: 'static>(&self, page: Page) -> Element<Message> {
        row!(text(page.title()).size(28), horizontal_space(),).into()
    }

    fn is_condensed(&self) -> bool {
        WINDOW_WIDTH.load(Ordering::Relaxed) < BREAK_POINT
    }

    fn page(&mut self, page: Page) {
        self.nav_bar_toggled_condensed = false;
        self.page = page;
    }

    fn parent_page_button<Message: Clone + From<Page> + 'static>(
        &self,
        sub_page: impl SubPage,
    ) -> Element<Message> {
        let page = sub_page.parent_page();
        column!(
            button::icon(icon::from_name("go-previous-symbolic").size(16))
                .label(page.title())
                .padding(0)
                .on_press(Message::from(page)),
            row!(text(sub_page.title()).size(28), horizontal_space(),),
        )
        .spacing(10)
        .into()
    }

    fn set_scale_factor(&mut self, factor: f32) {
        self.scale_factor = factor as f64;
        self.scale_factor_string = format!("{:.2}", factor);
    }

    fn sub_page_button<Message: Clone + From<Page> + 'static>(
        &self,
        sub_page: impl SubPage,
    ) -> Element<Message> {
        iced::widget::Button::new(
            container(
                settings::item_row(vec![
                    icon::from_name(sub_page.icon_name()).size(20).icon().into(),
                    column!(
                        text(sub_page.title()).size(14),
                        text(sub_page.description()).size(10),
                    )
                    .spacing(2)
                    .into(),
                    horizontal_space().into(),
                    icon::from_name("go-next-symbolic").size(20).icon().into(),
                ])
                .spacing(16),
            )
            .padding([20, 24])
            .class(theme::Container::List)
            .width(Length::Fill),
        )
        .width(Length::Fill)
        .padding(0)
        .style(theme::iced::Button::Transparent)
        .on_press(Message::from(sub_page.into_page()))
        // .id(BTN.clone())
        .into()
    }

    fn toggle_warning(&mut self) {
        self.show_warning = !self.show_warning
    }

    fn view_unimplemented_page<Message: 'static>(&self, page: Page) -> Element<Message> {
        settings::view_column(vec![
            self.page_title(page),
            text("We haven't created that panel yet, and/or it is using a similar idea as current Pop! designs.").into(),
        ]).into()
    }

    fn view_unimplemented_sub_page<'a, Message: Clone + From<Page> + 'static>(
        &'a self,
        sub_page: impl SubPage,
    ) -> Element<'a, Message> {
        settings::view_column(vec![
            self.parent_page_button(sub_page),
            text("We haven't created that panel yet, and/or it is using a similar idea as current Pop! designs.").into(),
        ]).into()
    }
}

impl Application for Window {
    type Executor = cosmic::executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Task<Self::Message>) {
        let mut window = Window::default()
            .nav_bar_toggled(true)
            .show_maximize(true)
            .show_minimize(true);

        window.title = String::from("COSMIC Design System - Iced");
        window.set_scale_factor(1.0);
        window.warning_message = String::from("You were not supposed to touch that.");

        window.insert_page(Page::Demo);
        window.insert_page(Page::Editor);
        window.insert_page(Page::WiFi);
        window.insert_page(Page::Networking(None));
        window.insert_page(Page::Bluetooth);
        window.insert_page(Page::Desktop(None)).activate();
        window.insert_page(Page::InputDevices(None));
        window.insert_page(Page::Displays);
        window.insert_page(Page::PowerAndBattery);
        window.insert_page(Page::Sound);
        window.insert_page(Page::PrintersAndScanners);
        window.insert_page(Page::PrivacyAndSecurity);
        window.insert_page(Page::SystemAndAccounts(None));
        window.insert_page(Page::TimeAndLanguage(None));
        window.insert_page(Page::Accessibility);
        window.insert_page(Page::Applications);
        window.demo.timeline = window.timeline.clone();

        (window, load_fonts().map(|_| Message::FontsLoaded))
    }

    fn title(&self) -> String {
        self.title.clone()
    }

    fn subscription(&self) -> Subscription<Message> {
        let window_break = listen_raw(|event, _| match event {
            cosmic::iced::Event::Window(window::Event::Resized { width, height: _ }) => {
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
        });

        Subscription::batch(vec![
            window_break.map(|_| Message::CondensedViewToggle),
            keyboard_nav::subscription().map(Message::KeyboardNav),
            self.timeline
                .borrow()
                .as_subscription()
                .map(|(_, instant)| Self::Message::Tick(instant)),
        ])
    }

    fn update(&mut self, message: Message) -> iced::Task<Self::Message> {
        let mut ret = Task::none();
        match message {
            Message::NavBar(key) => {
                if let Some(page) = self.nav_id_to_page.get(key).copied() {
                    self.nav_bar.activate(key);
                    self.page(page);
                }
            }
            Message::Page(page) => self.page(page),
            Message::Bluetooth(message) => {
                self.bluetooth.update(message);
            }
            Message::Demo(message) => match self.demo.update(message) {
                Some(demo::Output::Debug(debug)) => self.debug = debug,
                Some(demo::Output::ScalingFactor(factor)) => self.set_scale_factor(factor),
                Some(demo::Output::ThemeChanged(theme)) => {
                    self.theme = match theme {
                        demo::ThemeVariant::Light => Theme::light(),
                        demo::ThemeVariant::Dark => Theme::dark(),
                        demo::ThemeVariant::HighContrastDark => Theme::dark_hc(),
                        demo::ThemeVariant::HighContrastLight => Theme::light_hc(),
                        demo::ThemeVariant::Custom => Theme::custom(Arc::new(
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
                        demo::ThemeVariant::System => cosmic::theme::system_preference(),
                    };
                }
                Some(demo::Output::ToggleWarning) => self.toggle_warning(),
                None => (),
            },
            Message::Editor(message) => self.editor.update(message),
            Message::Desktop(message) => match self.desktop.update(message) {
                Some(desktop::Output::Page(page)) => self.page(page),
                None => (),
            },
            Message::ToggleNavBar => self.nav_bar_toggled = !self.nav_bar_toggled,
            Message::ToggleNavBarCondensed => {
                self.nav_bar_toggled_condensed = !self.nav_bar_toggled_condensed
            }
            Message::Drag => return drag(self.core().main_window_id().unwrap()),
            Message::Close => return close(self.core().main_window_id().unwrap()),
            Message::Minimize => return minimize(self.core().main_window_id().unwrap(), true),
            Message::Maximize => return toggle_maximize(self.core().main_window_id().unwrap()),

            Message::InputChanged => {}

            Message::CondensedViewToggle => {}
            Message::KeyboardNav(message) => match message {
                keyboard_nav::Message::FocusNext => ret = widget::focus_next(),
                keyboard_nav::Message::FocusPrevious => ret = widget::focus_previous(),
                _ => (),
            },
            Message::ToggleWarning => self.toggle_warning(),
            Message::FontsLoaded => {} // Message::Tick(instant) => self.timeline.borrow_mut().now(instant),            Message::Tick(instant) => self.timeline.borrow_mut().now(instant),
            Message::Tick(instant) => self.timeline.borrow_mut().now(instant),
        }
        ret
    }

    fn view(&self) -> Element<Message> {
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
            let mut nav_bar = nav_bar(&self.nav_bar, Message::NavBar).into_container();

            if !self.is_condensed() {
                nav_bar = nav_bar.max_width(300);
            }

            let nav_bar: Element<_> = nav_bar.into();
            widgets.push(nav_bar.debug(self.debug));
        }

        if !(self.is_condensed() && nav_bar_toggled) {
            let content: Element<_> = match self.page {
                Page::Demo => self.demo.view(self).map(Message::Demo),
                Page::Editor => self.editor.view(self).map(Message::Editor),
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
                Page::Networking(Some(sub_page)) => self.view_unimplemented_sub_page(sub_page),
                Page::Bluetooth => self.bluetooth.view(self).map(Message::Bluetooth),
                Page::Desktop(desktop_page_opt) => self
                    .desktop
                    .view(self, desktop_page_opt)
                    .map(Message::Desktop),
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
                scrollable(container(content.debug(self.debug)).align_x(iced::Alignment::Center))
                    .width(Length::Fill)
                    .into(),
            );
        }

        let content = container(row(widgets))
            .padding([0, 8, 8, 8])
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::Container::Background)
            .into();
        let warning = warning(&self.warning_message)
            .on_close(Message::ToggleWarning)
            .into();
        if self.show_warning {
            column![
                header,
                container(column(vec![
                    warning,
                    iced::widget::vertical_space()
                        .width(Length::Fixed(12.0))
                        .into(),
                    content,
                ]))
                .style(theme::Container::Background)
            ]
            .into()
        } else {
            column(vec![header, content]).into()
        }
    }

    fn scale_factor(&self) -> f64 {
        self.scale_factor
    }

    fn theme(&self) -> Theme {
        self.theme.clone()
    }
}
