use apply::Apply;
use iced::{self, Element, Length, widget, alignment::Vertical, theme, Renderer};
use iced_lazy::Component;

pub struct HeaderBar<Message>
{
    title: String,
    nav_title: String,
    sidebar_active: bool,
    show_minimize: bool,
    show_maximize: bool,
    on_sidebar_active: Box<dyn Fn(bool) -> Message>,
    on_close: Box<dyn Fn() -> Message>,
    on_drag: Box<dyn Fn() -> Message>,
}

pub fn header_bar<Message>(
    title: &str,
    toggled: bool,
    show_minimize: bool,
    show_maximize: bool,
    on_sidebar_active: impl Fn(bool) -> Message + 'static,
    on_close: impl Fn() -> Message + 'static,
    on_drag: impl Fn() -> Message + 'static,
) -> HeaderBar<Message> {
    HeaderBar::new(
        title, 
        toggled, 
        show_minimize,
        show_maximize,
        on_sidebar_active, 
        on_close, 
        on_drag
    )
}

#[derive(Debug, Clone)]
pub enum HeaderEvent {
    Close,
    ToggleSidebar,
    Drag,
    Minimize, 
    Maximize
}

impl<Message> HeaderBar<Message> {
    pub fn new(
        title: &str,
        toggled: bool,
        show_minimize: bool,
        show_maximize: bool,
        on_sidebar_active: impl Fn(bool) -> Message + 'static,
        on_close: impl Fn() -> Message + 'static,
        on_drag: impl Fn() -> Message + 'static,
    ) -> Self {
        Self {
            title: String::from(title),
            nav_title: String::new(),
            sidebar_active: toggled,
            show_minimize,
            show_maximize,
            on_sidebar_active: Box::new(on_sidebar_active),
            on_close: Box::new(on_close),
            on_drag: Box::new(on_drag),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn nav_title(mut self, nav_title: &str) -> Self {
        self.nav_title = nav_title.to_string();
        self
    }

    pub fn sidebar_active(mut self, sidebar_active: bool) -> Self {
        self.sidebar_active = sidebar_active;
        self
    }

    pub fn show_minimize(mut self, show_minimize: bool) -> Self {
        self.show_minimize = show_minimize;
        self
    }

    pub fn show_maximize(mut self, show_maximize: bool) -> Self {
        self.show_maximize = show_maximize;
        self
    }
}

impl<Message: Clone> Component<Message, Renderer> for HeaderBar<Message>  
{
    type State = ();

    type Event = HeaderEvent;

    fn update(
        &mut self,
        _state: &mut Self::State,
        event: Self::Event,
    ) -> Option<Message> {
        match event {
            HeaderEvent::Close => Some((self.on_close)()),
            HeaderEvent::ToggleSidebar => {
                self.sidebar_active = !self.sidebar_active;
                Some((self.on_sidebar_active)(self.sidebar_active))
            },
            HeaderEvent::Drag => Some((self.on_drag)()),
            HeaderEvent::Minimize => None,
            HeaderEvent::Maximize => None,
        }
    }

    fn view(&self, _state: &Self::State) -> Element<Self::Event> 
    {
        let nav_button = {
            let text = widget::text(&self.nav_title)
                .vertical_alignment(Vertical::Center)
                .width(Length::Shrink)
                .height(Length::Fill);

            let icon = super::icon(
                if self.sidebar_active {
                    "go-previous-symbolic"
                } else {
                    "go-next-symbolic"
                },
                24,
            )
            .width(Length::Units(24))
            .height(Length::Fill);

            widget::row!(text, icon)
                .padding(4)
                .spacing(4)
                .apply(widget::button)
                .style(theme::Button::Primary)
                .on_press(HeaderEvent::ToggleSidebar)
                .apply(widget::container)
                .center_y()
                .height(Length::Fill)
                .into()
        };

        let content = widget::container(widget::text(&self.title))
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        
        let window_controls = {
            let mut widgets: Vec<Element<HeaderEvent, _>> = Vec::with_capacity(3);

            let icon = |name, size, on_press| {
                super::icon(name, size)
                    .apply(widget::button)
                    .style(theme::Button::Primary)
                    .on_press(on_press)
            };

            if self.show_minimize {
                widgets.push(
                    icon("window-minimize-symbolic", 16, HeaderEvent::Minimize).into()
                );
            }

            if self.show_maximize {
                widgets.push(
                    icon("window-maximize-symbolic", 16, HeaderEvent::Maximize).into()
                );
            }

            widgets.push(
                icon("window-close-symbolic", 16, HeaderEvent::Close).into()
            );

            widget::row(widgets)
                .spacing(8)
                .apply(widget::container)
                .height(Length::Fill)
                .center_y()
                .into()
        };

        widget::row(vec![nav_button, content, window_controls])
            .height(Length::Units(50))
            .padding(10)
            .apply(widget::event_container)
            .center_y()
            .on_press(HeaderEvent::Drag)
            .into()
    }
}

impl<'a, Message: Clone + 'a> From<HeaderBar<Message>> for Element<'a, Message>
{
    fn from(header_bar: HeaderBar<Message>) -> Self {
        iced_lazy::component(header_bar)
    }
}