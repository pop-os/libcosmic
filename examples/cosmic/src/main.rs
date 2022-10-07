use cosmic::{
    widget::{
        button,
        list_item,
        list_row,
        list_section,
        list_view,
        nav_bar,
        nav_button,
        toggler,
        HeaderBar,
    },
    settings,
    iced::{self, theme, Alignment, Application, Color, Command, Element, Length, Theme},
    iced::widget::{
        checkbox,
        column,
        container,
        horizontal_space,
        pick_list,
        progress_bar,
        radio,
        row,
        slider,
        text,
        scrollable,
    },
    iced_lazy::responsive,
    iced_winit::window::drag,
    WindowMsg
};

pub fn main() -> cosmic::iced::Result {
    let mut settings = settings();
    settings.window.min_size = Some((600, 300));
    // TODO: Window resize handles not functioning yet
    // settings.window.decorations = false;
    Window::run(settings)
}


#[derive(Default)]
struct Window {
    headerbar: HeaderBar,
    page: u8,
    debug: bool,
    theme: Theme,
    slider_value: f32,
    checkbox_value: bool,
    toggler_value: bool,
    pick_list_selected: Option<&'static str>,
    exit: bool,
}

#[derive(Clone, Copy, Debug)]
enum Message {
    Page(u8),
    Debug(bool),
    ThemeChanged(Theme),
    ButtonPressed,
    SliderChanged(f32),
    CheckboxToggled(bool),
    TogglerToggled(bool),
    PickListSelected(&'static str),
    Window(WindowMsg)
}

impl From<WindowMsg> for Message {
    fn from(message: WindowMsg) -> Self {
        Self::Window(message)
    }
}

impl Application for Window {
    type Executor = iced::executor::Default;
    type Flags = ();
    type Message = Message;
    type Theme = Theme;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut window = Window::default();
        window.headerbar.title = String::from("COSMIC Design System - Iced");
        window.headerbar.nav_title = String::from("WiFi Settings");
        window.headerbar.sidebar_active = true;
        window.headerbar.show_minimize = true;
        window.headerbar.show_maximize = true;
        window.slider_value = 50.0;
        window.pick_list_selected = Some("Option 1");
        (window, Command::none())
    }

    fn title(&self) -> String {
        self.headerbar.title.clone()
    }

    fn update(&mut self, message: Message) -> iced::Command<Self::Message> {
        match message {
            Message::Page(page) => self.page = page,
            Message::Debug(debug) => self.debug = debug,
            Message::ThemeChanged(theme) => self.theme = theme,
            Message::ButtonPressed => {}
            Message::SliderChanged(value) => self.slider_value = value,
            Message::CheckboxToggled(value) => self.checkbox_value = value,
            Message::TogglerToggled(value) => self.toggler_value = value,
            Message::PickListSelected(value) => self.pick_list_selected = Some(value),
            Message::Window(msg) => match msg {
                WindowMsg::Close => self.exit = true,
                WindowMsg::ToggleSidebar => self.headerbar.sidebar_active = !self.headerbar.sidebar_active,
                WindowMsg::Drag => return drag(),
                WindowMsg::Minimize => {}
                WindowMsg::Maximize => {}
            }
        }

        iced::Command::none()
    }

    fn view(&self) -> Element<Message> {
        let mut header = self.headerbar.render();
        if self.debug {
            header = header.explain(Color::WHITE);
        }

        // TODO: Adding responsive makes this regenerate on every size change, and regeneration
        // involves allocations for many different items. Ideally, we could only make the nav bar
        // responsive and leave the content to be sized normally.
        let content = responsive(|size| {
            let condensed = size.width < 900.0;

            let sidebar: Option<Element<_>> = if self.headerbar.sidebar_active {
                Some(nav_bar!(
                    //TODO: Support symbolic icons
                    nav_button!("network-wireless", "Wi-Fi", condensed)
                        .on_press(Message::Page(0))
                        .style(if self.page == 0 { theme::Button::Primary } else { theme::Button::Text })
                    ,
                    nav_button!("preferences-desktop", "Desktop", condensed)
                        .on_press(Message::Page(1))
                        .style(if self.page == 1 { theme::Button::Primary } else { theme::Button::Text })
                    ,
                    nav_button!("system-software-update", "OS Upgrade & Recovery", condensed)
                        .on_press(Message::Page(2))
                        .style(if self.page == 2 { theme::Button::Primary } else { theme::Button::Text })
                )
                .max_width(if condensed {
                    56
                } else {
                    300
                })
                .into())
            } else {
                None
            };

            let choose_theme = [Theme::Light, Theme::Dark].iter().fold(
                row![text("Debug theme:")].spacing(10).align_items(Alignment::Center),
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
                list_section!(
                    "Debug",
                    choose_theme,
                    toggler(
                        String::from("Debug layout"),
                        self.debug,
                        Message::Debug,
                    )
                ),
                list_section!(
                    "Buttons",
                    list_row!(
                        button!("Primary")
                            .style(theme::Button::Primary)
                            .on_press(Message::ButtonPressed)
                        ,
                        button!("Secondary")
                            .style(theme::Button::Secondary)
                            .on_press(Message::ButtonPressed)
                        ,
                        button!("Positive")
                            .style(theme::Button::Positive)
                            .on_press(Message::ButtonPressed)
                        ,
                        button!("Destructive")
                            .style(theme::Button::Destructive)
                            .on_press(Message::ButtonPressed)
                        ,
                        button!("Text")
                            .style(theme::Button::Text)
                            .on_press(Message::ButtonPressed)
                        ,
                    ),
                    list_row!(
                        button!("Primary")
                            .style(theme::Button::Primary)
                        ,
                        button!("Secondary")
                            .style(theme::Button::Secondary)
                        ,
                        button!("Positive")
                            .style(theme::Button::Positive)
                        ,
                        button!("Destructive")
                            .style(theme::Button::Destructive)
                        ,
                        button!("Text")
                            .style(theme::Button::Text)
                        ,
                    ),
                ),
                list_section!(
                    "Controls",
                    list_item!(
                        "Toggler",
                        toggler(None, self.toggler_value, Message::TogglerToggled)
                    ),
                    list_item!(
                        "Pick List (TODO)",
                        pick_list(
                            vec![
                                "Option 1",
                                "Option 2",
                                "Option 3",
                                "Option 4",
                            ],
                            self.pick_list_selected,
                            Message::PickListSelected
                        )
                            .padding([8, 0, 8, 16])
                    ),
                    list_item!(
                        "Slider",
                        slider(0.0..=100.0, self.slider_value, Message::SliderChanged)
                            .width(Length::Units(250))
                    ),
                    list_item!(
                        "Progress",
                        progress_bar(0.0..=100.0, self.slider_value)
                            .width(Length::Units(250))
                            .height(Length::Units(4))
                    ),
                    checkbox("Checkbox", self.checkbox_value, Message::CheckboxToggled),
                )
            )
            .into();

            let mut widgets = Vec::with_capacity(2);

            if let Some(sidebar) = sidebar {
                widgets.push(if self.debug { sidebar.explain(Color::WHITE) } else { sidebar });
            }

            widgets.push(
                scrollable(row![
                    horizontal_space(Length::Fill),
                    if self.debug { content.explain(Color::WHITE) } else { content },
                    horizontal_space(Length::Fill),
                ])
                .scrollbar_width(12)
                .scroller_width(6)
                .into()
            );

            container(row(widgets))
            .padding([16, 8])
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        }).into();

        column(vec![header, content]).into()
    }

    fn should_exit(&self) -> bool {
        self.exit
    }

    fn theme(&self) -> Theme {
        self.theme
    }
}
