use cosmic::{
    widget::{
        header_bar, 
        nav_bar_style
    },
    iced,
    iced::{
        Theme, 
        Application, 
        Element,
        widget::{
            container, 
            column
        }, 
    }, 
    iced_winit::{
        Command, 
        Length, 
        widget::row, 
        window::drag, 
        theme
    }, 
    nav_button, 
    iced_lazy::responsive
};

#[derive(Default)]
pub struct App {
    page: u8,
    theme: Theme,
    sidebar_toggled: bool,
    show_minimize: bool,
    show_maximize: bool,
    exit: bool,
}

impl App {
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
#[derive(Debug, Clone, Copy)]
pub enum AppMsg {
    Close,
    ToggleSidebar(bool),
    Drag,
    Minimize, 
    Maximize,
    Page(u8),
}

impl Application for App {
    type Executor = iced::executor::Default;
    type Flags = ();
    type Message = AppMsg;
    type Theme = Theme;

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            App::default()
                .sidebar_toggled(true)
                .show_maximize(true)
                .show_minimize(true), 
            Command::none()
        )
    }

    fn title(&self) -> String {
        String::from("COSMIC Settings")
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            AppMsg::Close => self.exit = true,
            AppMsg::ToggleSidebar(toggled) => self.sidebar_toggled = toggled,
            AppMsg::Drag => return drag(),
            AppMsg::Minimize => {},
            AppMsg::Maximize => {},
            AppMsg::Page(page) => self.page = page,
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<Self::Message> {
        let header = header_bar(
            self.title().as_str(),
            self.sidebar_toggled,
            self.show_minimize,
            self.show_maximize,
            |toggled| AppMsg::ToggleSidebar(toggled),
            || AppMsg::Close, 
            || AppMsg::Drag 
        ).into();
        
        let content = responsive(|size| {
            let condensed = size.width < 900.0;

            let sidebar: Element<_> = cosmic::navbar![
                nav_button!("network-wireless", "Wi-Fi", condensed)
                .on_press(AppMsg::Page(0))
                .style(if self.page == 0 { theme::Button::Primary } else { theme::Button::Text })
                ,
                nav_button!("preferences-desktop", "Desktop", condensed)
                    .on_press(AppMsg::Page(1))
                    .style(if self.page == 1 { theme::Button::Primary } else { theme::Button::Text })
                ,
                nav_button!("system-software-update", "OS Upgrade & Recovery", condensed)
                    .on_press(AppMsg::Page(2))
                    .style(if self.page == 2 { theme::Button::Primary } else { theme::Button::Text }),
            ]
            .active(self.sidebar_toggled)
            .condensed(condensed)
            .style(theme::Container::Custom(nav_bar_style))
            .into();

            let mut widgets = Vec::with_capacity(2);

            widgets.push(sidebar);

            container(row(widgets))
            .padding([8, 8])
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