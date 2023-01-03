// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::state::{Key, SharedWidgetState, State};
use super::style::StyleSheet;
use super::UniqueWidgetState;

use derive_setters::Setters;
use iced::{
    alignment::{Horizontal, Vertical},
    event, mouse, touch, Background, Color, Element, Event, Length, Point, Rectangle, Size,
};
use iced_core::BorderRadius;
use iced_native::widget::tree;
use iced_native::{layout, renderer, widget::Tree, Clipboard, Layout, Shell, Widget};

/// Creates a [`HorizontalSegmentedButton`].
#[must_use]
pub fn horizontal_segmented_button<Message, Renderer, Data>(
    state: &State<Data>,
) -> HorizontalSegmentedButton<Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    HorizontalSegmentedButton::new(&state.inner)
}

/// A widget providing a conjoined set of horizontally-arranged buttons for choosing between.
///
/// The data for the widget comes from a [`State`] that is maintained the application.
#[derive(Setters)]
pub struct HorizontalSegmentedButton<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Contains application state also used for drawing.
    #[setters(skip)]
    state: &'a SharedWidgetState,
    /// Desired font for active tabs.
    font_active: Renderer::Font,
    /// Desired font for hovered tabs.
    font_hovered: Renderer::Font,
    /// Desired font for inactive tabs.
    font_inactive: Renderer::Font,
    /// Desired width of the widget.
    width: Length,
    /// Desired height of the widget.
    height: Length,
    /// Padding around a button.
    button_padding: [u16; 4],
    /// Desired height of a button.
    button_height: u16,
    /// Desired spacing between buttons.
    spacing: u16,
    /// Style to draw the widget in.
    #[setters(into)]
    style: <Renderer::Theme as StyleSheet>::Style,
    /// Emits the ID of the activated widget on selection.
    #[setters(skip)]
    on_activate: Option<Box<dyn Fn(Key) -> Message>>,
}

impl<'a, Message, Renderer> HorizontalSegmentedButton<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    #[must_use]
    pub fn new(state: &'a SharedWidgetState) -> Self {
        Self {
            state,
            font_active: Renderer::Font::default(),
            font_hovered: Renderer::Font::default(),
            font_inactive: Renderer::Font::default(),
            height: Length::Shrink,
            width: Length::Fill,
            button_padding: [4, 4, 4, 4],
            button_height: 32,
            spacing: 0,
            style: <Renderer::Theme as StyleSheet>::Style::default(),
            on_activate: None,
        }
    }

    /// Emits the ID of the activated widget on selection.
    #[must_use]
    pub fn on_activate(mut self, on_activate: impl Fn(Key) -> Message + 'static) -> Self {
        self.on_activate = Some(Box::from(on_activate));
        self
    }

    /// Creates a closure for generating the layout bounds of the buttons.
    fn button_bounds(&self, bounds: Rectangle) -> impl FnMut() -> Rectangle {
        let button_amount = self.state.buttons.len();
        let width = bounds.width / button_amount as f32;
        let spacing = self.spacing as f32;
        let half = spacing / 2.0;
        let mut bounds = bounds;
        bounds.width = width;
        let mut counter = 1;

        move || {
            let mut clone = bounds;
            if counter == 1 {
                clone.width -= half;
            } else if counter == button_amount {
                clone.x += spacing;
                clone.width -= spacing;
            } else {
                clone.x += half;
                clone.width -= half;
            }

            bounds.x += width;
            counter += 1;
            clone
        }
    }

    fn measure_button(
        &self,
        renderer: &Renderer,
        text: &str,
        text_size: u16,
        bounds: Size,
    ) -> (f32, f32) {
        let (mut w, mut h) = renderer.measure(text, text_size, Default::default(), bounds);
        w += self.button_padding[0] as f32 + self.button_padding[2] as f32;
        h += self.button_padding[1] as f32 + self.button_padding[3] as f32;
        h = h.max(self.button_height as f32);
        (w, h)
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer>
    for HorizontalSegmentedButton<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: StyleSheet,
    Message: 'static + Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<UniqueWidgetState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(UniqueWidgetState::default())
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        let limits = limits.width(self.width);
        let text_size = renderer.default_size();

        for (_, content) in self.state.buttons.iter() {
            let (w, h) = self.measure_button(renderer, &content.text, text_size, limits.max());
            width += w + f32::from(self.spacing * 2);
            height = height.max(h);
        }

        let size = limits
            .height(Length::Units(height as u16))
            .resolve(Size::new(width, height));

        layout::Node::new(size)
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
        let mut bounds_generator = self.button_bounds(bounds);
        let state = tree.state.downcast_mut::<UniqueWidgetState>();

        if bounds.contains(cursor_position) {
            for (key, _) in self.state.buttons.iter() {
                let bounds = bounds_generator();
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
        let mut generator = self.button_bounds(layout.bounds());

        if (0..self.state.buttons.len()).any(move |_| generator().contains(cursor_position)) {
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
        let state = tree.state.downcast_ref::<UniqueWidgetState>();
        let appearance = theme.horizontal(&self.style);
        let bounds = layout.bounds();
        let button_amount = self.state.buttons.len();

        let mut bounds_generator = self.button_bounds(bounds);

        // Draw the background, if a background was defined.
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

        // Draw each of the buttons in the widget.
        for (num, (key, content)) in self.state.buttons.iter().enumerate() {
            let bounds = bounds_generator();

            let (button_appearance, font) = if self.state.active == key {
                (appearance.button_active, &self.font_active)
            } else if state.hovered == key {
                (appearance.button_hover, &self.font_hovered)
            } else {
                (appearance.button_inactive, &self.font_inactive)
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

            // Draw the bottom border defined for this button.
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

            // Draw the text in this button.
            renderer.fill_text(iced_native::text::Text {
                content: &content.text,
                size: f32::from(renderer.default_size()),
                bounds: Rectangle { x, y, ..bounds },
                color: button_appearance.text_color,
                font: font.clone(),
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

impl<'a, Message, Renderer> From<HorizontalSegmentedButton<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer + 'a,
    Renderer::Theme: StyleSheet,
    Message: 'static + Clone,
{
    fn from(widget: HorizontalSegmentedButton<'a, Message, Renderer>) -> Self {
        Self::new(widget)
    }
}
