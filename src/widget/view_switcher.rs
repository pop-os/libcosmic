use derive_setters::Setters;
use iced::{
    alignment::{Horizontal, Vertical},
    theme,
    widget::{button, container, text, Row},
    Background, Element, Length, Renderer, Theme, Alignment,
};
use iced_lazy::Component;

#[derive(Setters)]
pub struct ViewSwitcher<'a, Message> {
    options: Vec<&'a str>,
    #[setters(strip_option)]
    on_view_changed: Option<Box<dyn Fn(usize) -> Message + 'a>>
}

pub fn view_switcher<'a, Message>() -> ViewSwitcher<'a, Message> {
    ViewSwitcher { options: vec![], on_view_changed: None }
}

#[derive(Clone)]
pub enum ViewSwitcherEvent {
    ViewSelected(usize),
}

#[derive(Default)]
pub struct ViewSwitcherState {
    selected_view: usize,
}

impl<'a, Message> Component<Message, Renderer> for ViewSwitcher<'a, Message> {
    type State = ViewSwitcherState;

    type Event = ViewSwitcherEvent;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            ViewSwitcherEvent::ViewSelected(index) => state.selected_view = index,
        }
        self.on_view_changed.as_ref().map(|on_view_changed| (on_view_changed)(state.selected_view))
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event, Renderer> {
        let mut options: Vec<Element<'a, Self::Event, Renderer>> = vec![];

        for (index, option) in self.options.iter().enumerate() {
            options.push(
                button(
                    container(text(option))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                .width(Length::Fill)
                .height(Length::Units(32))
                .style(if state.selected_view == index {
                    theme::Button::Secondary.into()
                } else {
                    theme::Button::Primary.into()
                })
                .on_press(ViewSwitcherEvent::ViewSelected(index))
                .into(),
            );
        }
        container(Row::with_children(options).align_items(Alignment::Fill))
            .width(Length::Fill)
            .padding(2)
            .style(theme::Container::Custom(view_switcher_container_style))
            .into()
    }
}

pub fn view_switcher_container_style(theme: &Theme) -> iced_style::container::Appearance {
    let accent = &theme.cosmic().accent;
    iced_style::container::Appearance {
        text_color: Default::default(),
        background: Some(Background::Color(accent.base.into())),
        border_radius: 24.0,
        border_width: 2.0,
        border_color: accent.base.into(),
    }
}

impl<'a, Message: 'a> From<ViewSwitcher<'a, Message>> for Element<'a, Message> {
    fn from(view_switcher: ViewSwitcher<'a, Message>) -> Self {
        iced_lazy::component(view_switcher)
    }
}
