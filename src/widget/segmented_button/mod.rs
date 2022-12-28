/// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

mod state;
mod style;

pub use self::state::{ButtonContent, Key, SecondaryState, State, WidgetState};
pub use self::style::{Appearance, ButtonAppearance, StyleSheet};

use derive_setters::Setters;
use iced::{
    alignment::{Horizontal, Vertical},
    event, mouse, touch, Background, Color, Element, Event, Length, Point, Rectangle, Size,
};
use iced_core::BorderRadius;
use iced_native::widget::tree;
use iced_native::{layout, renderer, widget::Tree, Clipboard, Layout, Shell, Widget};

/// State that is maintained by the widget internally.
#[derive(Default)]
struct PrivateWidgetState {
    /// The ID of the button that is being hovered. Defaults to null.
    hovered: Key,
}

/// A linear set of options for choosing between.
#[derive(Setters)]
pub struct SegmentedButton<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    state: &'a WidgetState,
    width: Length,
    height: Length,
    spacing: u16,
    #[setters(into)]
    style: <Renderer::Theme as StyleSheet>::Style,
    #[setters(skip)]
    on_activate: Option<Box<dyn Fn(Key) -> Message>>,
}

impl<'a, Message, Renderer> SegmentedButton<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    #[must_use]
    pub fn new(state: &'a WidgetState) -> Self {
        Self {
            state,
            height: Length::Units(48),
            width: Length::Fill,
            spacing: 0,
            style: <Renderer::Theme as StyleSheet>::Style::default(),
            on_activate: None,
        }
    }

    #[must_use]
    pub fn on_activate(mut self, on_activate: impl Fn(Key) -> Message + 'static) -> Self {
        self.on_activate = Some(Box::from(on_activate));
        self
    }
}

#[must_use]
pub fn segmented_button<Message, Renderer, Data>(
    state: &State<Data>,
) -> SegmentedButton<Message, Renderer>
where
    Renderer: iced_native::Renderer,
    Renderer::Theme: StyleSheet,
{
    SegmentedButton::new(&state.inner)
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for SegmentedButton<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: StyleSheet,
    Message: 'static + Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PrivateWidgetState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(PrivateWidgetState::default())
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let bounds = limits.max();

        let size = renderer.default_size();

        let mut width = 0.0;
        let height = bounds.height;

        for (_, content) in self.state.buttons.iter() {
            let (w, _) = renderer.measure(&content.text, size, Default::default(), bounds);
            width += w + f32::from(self.spacing * 2);
        }

        layout::Node::new(limits.resolve(Size::new(width, height)))
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<PrivateWidgetState>();

        if bounds.contains(cursor_position) {
            let button_width = bounds.width / self.state.buttons.len() as f32;
            for (num, (key, _)) in self.state.buttons.iter().enumerate() {
                let mut bounds = bounds;
                bounds.width = button_width;
                bounds.x += num as f32 * button_width;

                if bounds.contains(cursor_position) {
                    // Record that the mouse is hovering over this button.
                    state.hovered = key;

                    if let Some(on_activate) = self.on_activate.as_ref() {
                        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                        | Event::Touch(touch::Event::FingerLifted { .. }) = event
                        {
                            shell.publish(on_activate(key));
                            return event::Status::Captured;
                        }
                    }
                }
            }
        } else {
            state.hovered = Key::default();
        }

        event::Status::Ignored
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor_position: iced::Point,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> iced_native::mouse::Interaction {
        if layout.bounds().contains(cursor_position) {
            iced_native::mouse::Interaction::Pointer
        } else {
            iced_native::mouse::Interaction::Idle
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &<Renderer as iced_native::Renderer>::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor_position: iced::Point,
        _viewport: &iced::Rectangle,
    ) {
        let state = tree.state.downcast_ref::<PrivateWidgetState>();
        let appearance = theme.appearance(&self.style);
        let bounds = layout.bounds();
        let button_amount = self.state.buttons.len();
        let button_width = bounds.width / button_amount as f32;

        if let Some(background) = appearance.background {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border_radius: appearance.border_radius,
                    border_width: appearance.border_width,
                    border_color: appearance.border_color,
                },
                background,
            );
        }

        for (num, (key, content)) in self.state.buttons.iter().enumerate() {
            let mut bounds = bounds;
            bounds.width = button_width;
            bounds.x += num as f32 * button_width;

            let button_appearance = if self.state.active == key {
                appearance.button_active
            } else if state.hovered == key {
                appearance.button_hover
            } else {
                appearance.button_inactive
            };

            let x = bounds.center_x();
            let y = bounds.center_y();

            // Render the background of the button.
            if button_appearance.background.is_some() {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border_radius: if num == 0 {
                            button_appearance.border_radius_first
                        } else if num + 1 == button_amount {
                            button_appearance.border_radius_last
                        } else {
                            button_appearance.border_radius_middle
                        },
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    button_appearance
                        .background
                        .unwrap_or(Background::Color(Color::TRANSPARENT)),
                );
            }

            // Render the bottom border.
            if let Some((width, background)) = button_appearance.border_bottom {
                let mut bounds = bounds;
                bounds.y = bounds.y + bounds.height - width;
                bounds.height = width;

                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border_radius: BorderRadius::from(0.0),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    background,
                );
            }

            // Render the text.
            renderer.fill_text(iced_native::text::Text {
                content: &content.text,
                size: f32::from(renderer.default_size()),
                bounds: Rectangle { x, y, ..bounds },
                color: button_appearance.text_color,
                font: Default::default(),
                horizontal_alignment: Horizontal::Center,
                vertical_alignment: Vertical::Center,
            });
        }
    }

    fn overlay<'b>(
        &'b self,
        _tree: &'b mut Tree,
        _layout: iced_native::Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<iced_native::overlay::Element<'b, Message, Renderer>> {
        None
    }
}

impl<'a, Message, Renderer> From<SegmentedButton<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer + 'a,
    Renderer::Theme: StyleSheet,
    Message: 'static + Clone,
{
    fn from(widget: SegmentedButton<'a, Message, Renderer>) -> Self {
        Self::new(widget)
    }
}
