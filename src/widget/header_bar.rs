use crate::{theme, Element, Renderer};
use apply::Apply;
use derive_setters::*;
use iced::{self, alignment::Vertical, widget, Length};
use iced_lazy::Component;

#[derive(Setters)]
pub struct HeaderBar<Message> {
    title: String,
    nav_title: String,
    sidebar_active: bool,
    show_minimize: bool,
    show_maximize: bool,
    #[setters(strip_option)]
    on_close: Option<Message>,
    #[setters(strip_option)]
    on_drag: Option<Message>,
    #[setters(strip_option)]
    on_maximize: Option<Message>,
    #[setters(strip_option)]
    on_minimize: Option<Message>,
    #[setters(strip_option)]
    on_sidebar_toggle: Option<Message>,
}

pub fn header_bar<Message>() -> HeaderBar<Message> {
    HeaderBar {
        title: String::default(),
        nav_title: String::default(),
        sidebar_active: false,
        show_minimize: false,
        show_maximize: false,
        on_sidebar_toggle: None,
        on_close: None,
        on_drag: None,
        on_maximize: None,
        on_minimize: None,
    }
}

#[derive(Debug, Clone)]
pub enum HeaderEvent {
    Close,
    ToggleSidebar,
    Drag,
    Minimize,
    Maximize,
}

impl<Message: Clone> Component<Message, Renderer> for HeaderBar<Message> {
    type State = ();

    type Event = HeaderEvent;

    fn update(&mut self, _state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            HeaderEvent::Close => self.on_close.clone(),

            HeaderEvent::ToggleSidebar => self.on_sidebar_toggle.clone(),

            HeaderEvent::Drag => self.on_drag.clone(),

            HeaderEvent::Maximize => self.on_maximize.clone(),

            HeaderEvent::Minimize => self.on_minimize.clone(),
        }
    }

    fn view(&self, _state: &Self::State) -> Element<Self::Event> {
        let nav_button = {
            let text = widget::text(&self.nav_title)
                .style(theme::Text::Accent)
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
            .style(theme::Svg::Accent)
            .width(Length::Units(24))
            .height(Length::Fill);

            widget::row!(text, icon)
                .padding(4)
                .spacing(4)
                .apply(widget::button)
                .style(theme::Button::Secondary)
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
            let mut widgets: Vec<Element<HeaderEvent>> = Vec::with_capacity(3);

            let icon = |name, size, on_press| {
                super::icon(name, size)
                    .style(crate::theme::Svg::Accent)
                    .apply(widget::button)
                    .style(theme::Button::Text)
                    .on_press(on_press)
            };

            if self.show_minimize {
                widgets.push(icon("window-minimize-symbolic", 16, HeaderEvent::Minimize).into());
            }

            if self.show_maximize {
                widgets.push(icon("window-maximize-symbolic", 16, HeaderEvent::Maximize).into());
            }

            widgets.push(icon("window-close-symbolic", 16, HeaderEvent::Close).into());

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
            .on_release(HeaderEvent::Maximize)
            .into()
    }
}

impl<'a, Message: Clone + 'a> From<HeaderBar<Message>> for Element<'a, Message> {
    fn from(header_bar: HeaderBar<Message>) -> Self {
        iced_lazy::component(header_bar)
    }
}
