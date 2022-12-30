// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A widget providing a conjoined set of linear buttons for choosing between.
//!
//! ## Example
//!
//! Add the state and a message variant in your application for handling selections.
//!
//! ```no_run
//! use iced_core::Length;
//! use cosmic::theme;
//! use cosmic::widget::segmented_button;
//!
//! enum AppMessage {
//!     Selected(segmented_button::Key)
//! }
//!
//! struct App {
//!     ...
//!     state: segmented_button::State<u16>(),
//!     ...
//! }
//! ```
//!
//! Then add choices to the state, while activating the first.
//!
//! ```no_run
//! let first_key = application.state.insert("Choice A", 0);
//! application.state.insert("Choice B", 1);
//! application.state.insert("Choice C", 2);
//! application.state.activate(first_key);
//! ```
//!
//! Then use it in the view method to create segmented button widgets.
//!
//! ```no_run
//! let widget = segmentend_button(&application.state)
//!     .style(theme::SegmentedButton::Selection)
//!     .height(Length::Units(32))
//!     .on_activate(AppMessage::Selected);
//! ```

/// COSMIC configurations of [`SegmentedButton`].
pub mod cosmic;

mod state;
mod style;

pub use self::state::{ButtonContent, Key, SecondaryState, State, WidgetState};
pub use self::style::{Appearance, ButtonAppearance, StyleSheet};

use crate::widget::Orientation;

use derive_setters::Setters;
use iced::{
    alignment::{Horizontal, Vertical},
    event, mouse, touch, Background, Color, Element, Event, Length, Point, Rectangle, Size,
};
use iced_core::BorderRadius;
use iced_native::widget::tree;
use iced_native::{layout, renderer, widget::Tree, Clipboard, Layout, Shell, Widget};

/// Creates a widget that presents multiple conjoined buttons.
#[must_use]
pub fn segmented_button<Message, Renderer, Data>(
    state: &State<Data>,
) -> SegmentedButton<Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    SegmentedButton::new(&state.inner)
}

/// State that is maintained by the widget internally.
#[derive(Default)]
struct PrivateWidgetState {
    /// The ID of the button that is being hovered. Defaults to null.
    hovered: Key,
}

/// A widget providing a conjoined set of linear buttons for choosing between.
///
/// The data for the widget comes from a [`State`] that is maintained the application.
#[derive(Setters)]
pub struct SegmentedButton<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Contains application state also used for drawing.
    #[setters(skip)]
    state: &'a WidgetState,
    /// Desired font for active tabs.
    font_active: Renderer::Font,
    /// Desired font for hovered tabs.
    font_hovered: Renderer::Font,
    /// Desired font for inactive tabs.
    font_inactive: Renderer::Font,
    /// Orientation of the buttons.
    orientation: Orientation,
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

impl<'a, Message, Renderer> SegmentedButton<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    #[must_use]
    pub fn new(state: &'a WidgetState) -> Self {
        Self {
            state,
            font_active: Renderer::Font::default(),
            font_hovered: Renderer::Font::default(),
            font_inactive: Renderer::Font::default(),
            orientation: Orientation::Horizontal,
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
    fn button_bounds(
        &self,
        bounds: Rectangle,
    ) -> stack_dst::ValueA<dyn FnMut() -> Rectangle, [usize; 4]> {
        let button_amount = self.state.buttons.len() as f32;
        match self.orientation {
            Orientation::Horizontal => {
                let width = bounds.width / button_amount;
                let mut bounds = bounds;
                bounds.width = width;

                let closure = move || {
                    let clone = bounds;
                    bounds.x += width;
                    clone
                };

                stack_dst::ValueA::new_stable(closure, |p| p as _)
                    .ok()
                    .unwrap()
            }

            Orientation::Vertical => {
                let height = bounds.height / button_amount;
                let mut bounds = bounds;
                bounds.height = height;

                let closure = move || {
                    let clone = bounds;
                    bounds.y += height;
                    clone
                };

                stack_dst::ValueA::new_stable(closure, |p| p as _)
                    .ok()
                    .unwrap()
            }
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
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        let mut limits = limits.width(self.width);
        let text_size = renderer.default_size();

        match self.orientation {
            Orientation::Horizontal => {
                for (_, content) in self.state.buttons.iter() {
                    let (w, h) =
                        self.measure_button(renderer, &content.text, text_size, limits.max());
                    width += w + f32::from(self.spacing * 2);
                    height = height.max(h);
                }

                limits = limits.height(Length::Units(height as u16));
            }
            Orientation::Vertical => {
                for (_, content) in self.state.buttons.iter() {
                    let (w, h) =
                        self.measure_button(renderer, &content.text, text_size, limits.max());
                    height += h + f32::from(self.spacing * 2);
                    width = width.max(w);
                }
                limits = limits.height(Length::Units(height as u16));
            }
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
        let mut bounds_generator = self.button_bounds(bounds);
        let state = tree.state.downcast_mut::<PrivateWidgetState>();

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
        let appearance = theme.appearance(&self.style, self.orientation);
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
