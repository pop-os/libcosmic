use derive_setters::Setters;
use iced::{
    alignment::{Horizontal, Vertical},
    theme,
    widget::{button, container, text, Row},
    Background, Element, Length, Renderer, Theme, Color,
};
use iced_lazy::Component;

#[derive(Setters)]
pub struct SegmentedButton<'a, Message> {
    options: Vec<&'a str>,
    #[setters(strip_option)]
    on_button_pressed: Option<Box<dyn Fn(usize) -> Message>>,
}

pub fn segmented_button<'a, Message>() -> SegmentedButton<'a, Message> {
    SegmentedButton {
        options: vec![],
        on_button_pressed: None,
    }
}

#[derive(Clone)]
pub enum SegmentedButtonEvent {
    Pressed(usize),
}

impl<'a, Message> Component<Message, Renderer> for SegmentedButton<'a, Message> {
    type State = usize;

    type Event = SegmentedButtonEvent;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            SegmentedButtonEvent::Pressed(index) => {
                *state = index;
                self.on_button_pressed
                    .as_ref()
                    .map(|on_button_pressed| (on_button_pressed)(index))
            }
        }
    }

    fn view(&self, state: &Self::State) -> Element<Self::Event> {
        let buttons: Vec<Element<'a, Self::Event>> = self
            .options
            .iter()
            .enumerate()
            .map(|(index, option)| {
                button(
                    container(text(*option))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .align_x(Horizontal::Center)
                        .align_y(Vertical::Center),
                )
                .width(Length::Units(136))
                .height(Length::Units(32))
                .style(if *state == index {
                    theme::Button::Primary.into()
                } else {
                    theme::Button::Secondary.into()
                })
                .on_press(SegmentedButtonEvent::Pressed(index))
                .into()
            })
            .collect();
        container(Row::with_children(buttons).spacing(5).width(Length::Shrink))
            .padding(2)
            .style(theme::Container::Custom(segmented_button_container_style))
            .into()
    }
}

// TODO: Allow button to receive a custom style.
pub fn _segmented_button_style(theme: &Theme) -> iced_style::button::Appearance {
    let primary = &theme.cosmic().primary;
    let accent = &theme.cosmic().accent;
    iced_style::button::Appearance {
        shadow_offset: Default::default(),
        background: Some(Background::Color(primary.base.into())),
        border_radius: 0.0,
        border_width: 0.0,
        border_color: Color::TRANSPARENT,
        text_color: accent.on.into(),
    }
}

pub fn segmented_button_container_style(theme: &Theme) -> iced_style::container::Appearance {
    let primary = &theme.cosmic().primary;
    iced_style::container::Appearance {
        text_color: Default::default(),
        background: Some(Background::Color(primary.base.into())),
        border_radius: 24.0,
        border_width: 0.0,
        border_color: primary.base.into(),
    }
}

impl<'a, Message: 'a> From<SegmentedButton<'a, Message>> for Element<'a, Message> {
    fn from(segmented_button: SegmentedButton<'a, Message>) -> Self {
        iced_lazy::component(segmented_button)
    }
}
