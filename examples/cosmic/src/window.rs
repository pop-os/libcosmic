use cosmic::widget::{expander, nav_bar, nav_bar_item};
use cosmic::{
    iced::widget::{
        checkbox, column, container, horizontal_space, pick_list, progress_bar, radio, row, slider,
        text,
    },
    iced::{self, theme, Alignment, Application, Color, Command, Element, Length, Theme},
    iced_lazy::responsive,
    iced_winit::window::{drag, maximize, minimize},
    list_view, list_view_item, list_view_row, list_view_section, scrollable,
    widget::{button, header_bar, list_box, list_row, list_view::*, toggler},
};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Window {
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
        (window, Command::none())
    }

    fn title(&self) -> String {
        String::from("COSMIC Design System - Iced")
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
            Message::Minimize => return minimize(),
            Message::Maximize => return maximize(),
            Message::RowSelected(row) => println!("Selected row {row}"),
        }

        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let mut header: Element<Message, _> = header_bar()
            .title(self.title())
            //            .nav_title(String::from("Settings"))
            .sidebar_active(self.sidebar_toggled)
            .show_minimize(self.show_minimize)
            .show_maximize(self.show_maximize)
            .on_close(Message::Close)
            .on_drag(Message::Drag)
            .on_maximize(Message::Maximize)
            .on_minimize(Message::Minimize)
            .on_sidebar_toggle(Message::ToggleSidebar)
            .into();

        if self.debug {
            header = header.explain(Color::WHITE);
        }

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

            let content: Element<_> = list_view!(
                list_view_section!(
                    "Debug",
                    list_view_item!("Debug theme", choose_theme),
                    list_view_item!(
                        "Debug layout",
                        toggler(String::from("Debug layout"), self.debug, Message::Debug,)
                    )
                ),
                list_view_section!(
                    "Buttons",
                    list_view_row!(
                        button!("Primary")
                            .style(theme::Button::Primary)
                            .on_press(Message::ButtonPressed),
                        button!("Secondary")
                            .style(theme::Button::Secondary)
                            .on_press(Message::ButtonPressed),
                        button!("Positive")
                            .style(theme::Button::Positive)
                            .on_press(Message::ButtonPressed),
                        button!("Destructive")
                            .style(theme::Button::Destructive)
                            .on_press(Message::ButtonPressed),
                        button!("Text")
                            .style(theme::Button::Text)
                            .on_press(Message::ButtonPressed),
                    ),
                    list_view_row!(
                        button!("Primary").style(theme::Button::Primary),
                        button!("Secondary").style(theme::Button::Secondary),
                        button!("Positive").style(theme::Button::Positive),
                        button!("Destructive").style(theme::Button::Destructive),
                        button!("Text").style(theme::Button::Text),
                    ),
                ),
                list_view_section!(
                    "Controls",
                    list_view_item!(
                        "Toggler",
                        toggler(None, self.toggler_value, Message::TogglerToggled)
                    ),
                    list_view_item!(
                        "Pick List (TODO)",
                        pick_list(
                            vec!["Option 1", "Option 2", "Option 3", "Option 4",],
                            self.pick_list_selected,
                            Message::PickListSelected
                        )
                        .padding([8, 0, 8, 16])
                    ),
                    list_view_item!(
                        "Slider",
                        slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                            .width(Length::Units(250))
                    ),
                    list_view_item!(
                        "Progress",
                        progress_bar(0.0..=100.0, self.slider_value)
                            .width(Length::Units(250))
                            .height(Length::Units(4))
                    ),
                    checkbox("Checkbox", self.checkbox_value, Message::CheckboxToggled),
                ),
                list_view_section!(
                    "Expander",
                    expander()
                        .title("Label")
                        .subtitle("Caption")
                        .icon(String::from("edit-paste"))
                        .on_row_selected(Box::new(Message::RowSelected))
                        .rows(vec![
                            list_row()
                                .title("Label")
                                .subtitle("Caption")
                                .icon(String::from("help-about")),
                            list_row().subtitle("Caption").title("Label"),
                            list_row().title("Label")
                        ])
                ),
                list_view_section!(
                    "List Box",
                    list_box()
                        .style(theme::Container::Custom(list_section_style))
                        .children(vec![
                            cosmic::list_box_row!("Title").into(),
                            cosmic::list_box_row!("Title", "Subtitle").into(),
                            cosmic::list_box_row!("Title", "", "edit-paste").into(),
                            cosmic::list_box_row!("", "Subtitle", "edit-paste").into(),
                            cosmic::list_box_row!("Title", "Subtitle", "edit-paste").into()
                        ])
                        .render()
                ),
            )
            .into();

            let mut widgets = Vec::with_capacity(2);

            widgets.push(if self.debug {
                sidebar.explain(Color::WHITE)
            } else {
                sidebar
            });

            widgets.push(
                scrollable!(row![
                    horizontal_space(Length::Fill),
                    if self.debug {
                        content.explain(Color::WHITE)
                    } else {
                        content
                    },
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
