use iced::widget::Button;
use iced_native::{Element, Widget};

use super::cosmic_widget::{CosmicWidget, Layer};

pub struct CosmicButton<'a, Message, Renderer: iced_native::Renderer>
where
    <Renderer as iced_native::Renderer>::Theme: iced_style::button::StyleSheet,
{
    button: Option<iced_native::widget::button::Button<'a, Message, Renderer>>,
    layer: Layer,
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for CosmicButton<'a, Message, Renderer>
where
    <Renderer as iced_native::Renderer>::Theme: iced_style::button::StyleSheet,
    Renderer: iced_native::Renderer,
    Message: Clone,
{
    fn width(&self) -> iced::Length {
        if let Some(button) = &self.button {
            Widget::width(button)
        } else {
            iced::Length::Shrink
        }
    }

    fn height(&self) -> iced::Length {
        if let Some(button) = &self.button {
            Widget::height(button)
        } else {
            iced::Length::Shrink
        }
    }

    fn layout(
        &self,
        renderer: &Renderer,
        limits: &iced_native::layout::Limits,
    ) -> iced_native::layout::Node {
        if let Some(button) = &self.button {
            Widget::layout(button, renderer, limits)
        } else {
            iced_native::layout::Node::new(limits.max())
        }
    }

    fn draw(
        &self,
        state: &iced_native::widget::Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_native::Renderer>::Theme,
        style: &iced_native::renderer::Style,
        layout: iced_native::Layout<'_>,
        cursor_position: iced::Point,
        viewport: &iced::Rectangle,
    ) {
        if let Some(button) = &self.button {
            Widget::draw(
                button,
                state,
                renderer,
                theme,
                style,
                layout,
                cursor_position,
                viewport,
            )
        }
    }
}

impl<'a, Message, Renderer> CosmicWidget<Message, Renderer> for CosmicButton<'a, Message, Renderer>
where
    <Renderer as iced_native::Renderer>::Theme: iced_style::button::StyleSheet,
    Renderer: iced_native::Renderer,
    Message: Clone,
{
    fn set_layer(&mut self, layer: Layer) {
        self.layer = layer;
    }
}

impl<'a, Message, Renderer> CosmicButton<'a, Message, Renderer>
where
    <Renderer as iced_native::Renderer>::Theme: iced_style::button::StyleSheet,
    Renderer: iced_native::Renderer,
    Message: Clone,
{
    #[must_use]
    /// will apply layer to the widget and all of its children
    pub fn with_cosmic_child<
        T: CosmicWidget<Message, Renderer> + Into<Element<'a, Message, Renderer>>,
    >(
        self,
        mut child: T,
    ) -> Self {
        child.set_layer(self.child_layer());
        Self {
            layer: self.layer,
            button: Some(Button::new(child.into())),
        }
    }

    #[must_use]
    /// will NOT apply layer to the widget and all of its children
    pub fn with_child<T: Widget<Message, Renderer> + Into<Element<'a, Message, Renderer>>>(
        self,
        child: T,
    ) -> Self {
        Self {
            layer: self.layer,
            button: Some(Button::new(child.into())),
        }
    }
}
