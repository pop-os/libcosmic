use std::marker::PhantomData;

use super::state::{Key, SharedWidgetState};
use super::style::StyleSheet;

use derive_setters::Setters;
use iced::{
    alignment, event, mouse, touch, Background, Color, Element, Event, Length, Point, Rectangle,
    Size,
};
use iced_core::BorderRadius;
use iced_native::widget::tree;
use iced_native::{layout, renderer, widget::Tree, Clipboard, Layout, Shell, Widget};

/// Isolates variant-specific behaviors from [`SegmentedButton`].
pub trait SegmentedVariant {
    type Renderer: iced_native::Renderer;

    /// Get the appearance for this variant of the widget.
    fn variant_appearance(
        theme: &<Self::Renderer as iced_native::Renderer>::Theme,
        style: &<<Self::Renderer as iced_native::Renderer>::Theme as StyleSheet>::Style,
    ) -> super::Appearance
    where
        <Self::Renderer as iced_native::Renderer>::Theme: StyleSheet;

    /// Calculates the bounds for the given button by its position.
    fn variant_button_bounds(&self, bounds: Rectangle, position: usize) -> Rectangle;

    /// Calculates the layout of this variant.
    fn variant_layout(&self, renderer: &Self::Renderer, limits: &layout::Limits) -> layout::Node;
}

#[derive(Setters)]
pub struct SegmentedButton<'a, Variant, Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: StyleSheet,
{
    /// Contains application state also used for drawing.
    #[setters(skip)]
    pub(super) state: &'a SharedWidgetState,
    /// Desired font for active tabs.
    pub(super) font_active: Renderer::Font,
    /// Desired font for hovered tabs.
    pub(super) font_hovered: Renderer::Font,
    /// Desired font for inactive tabs.
    pub(super) font_inactive: Renderer::Font,
    /// Desired width of the widget.
    pub(super) width: Length,
    /// Desired height of the widget.
    pub(super) height: Length,
    /// Padding around a button.
    pub(super) button_padding: [u16; 4],
    /// Desired height of a button.
    pub(super) button_height: u16,
    /// Desired spacing between buttons.
    pub(super) spacing: u16,
    /// Style to draw the widget in.
    #[setters(into)]
    pub(super) style: <Renderer::Theme as StyleSheet>::Style,
    #[setters(skip)]
    /// Emits the ID of the activated widget on selection.
    pub(super) on_activate: Option<Box<dyn Fn(Key) -> Message>>,
    #[setters(skip)]
    /// Defines the implementation of this struct
    variant: PhantomData<Variant>,
}

impl<'a, Variant, Message, Renderer> SegmentedButton<'a, Variant, Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: StyleSheet,
    Self: SegmentedVariant<Renderer = Renderer>,
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
            variant: PhantomData,
        }
    }

    /// Emits the ID of the activated widget on selection.
    #[must_use]
    pub fn on_activate(mut self, on_activate: impl Fn(Key) -> Message + 'static) -> Self {
        self.on_activate = Some(Box::from(on_activate));
        self
    }

    pub(super) fn measure_button(
        &self,
        renderer: &Renderer,
        text: &str,
        text_size: u16,
        bounds: Size,
    ) -> (f32, f32) {
        let (mut w, mut h) = renderer.measure(text, text_size, Default::default(), bounds);
        w += f32::from(self.button_padding[0]) + f32::from(self.button_padding[2]);
        h += f32::from(self.button_padding[1]) + f32::from(self.button_padding[3]);
        h = h.max(f32::from(self.button_height));
        (w, h)
    }

    pub(super) fn max_button_dimensions(
        &self,
        renderer: &Renderer,
        text_size: u16,
        bounds: Size,
    ) -> (f32, f32) {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for (_, content) in self.state.buttons.iter() {
            let (w, h) = self.measure_button(renderer, &content.text, text_size, bounds);
            height = height.max(h);
            width = width.max(w);
        }

        (width, height)
    }
}

impl<'a, Variant, Message, Renderer> Widget<Message, Renderer>
    for SegmentedButton<'a, Variant, Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer,
    Renderer::Theme: StyleSheet,
    Self: SegmentedVariant<Renderer = Renderer>,
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
        self.variant_layout(renderer, limits)
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
        let state = tree.state.downcast_mut::<UniqueWidgetState>();

        if bounds.contains(cursor_position) {
            for (nth, (key, _)) in self.state.buttons.iter().enumerate() {
                let bounds = self.variant_button_bounds(bounds, nth);
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
        let bounds = layout.bounds();
        if (0..self.state.buttons.len()).any(|nth| {
            self.variant_button_bounds(bounds, nth)
                .contains(cursor_position)
        }) {
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
        let appearance = Self::variant_appearance(theme, &self.style);
        let bounds = layout.bounds();
        let button_amount = self.state.buttons.len();

        // Draw the background, if a background was defined.
        if let Some(background) = appearance.background {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border_radius: appearance.border_radius,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                },
                background,
            );
        }

        // Draw each of the buttons in the widget.
        for (nth, (key, content)) in self.state.buttons.iter().enumerate() {
            let bounds = self.variant_button_bounds(bounds, nth);

            let (status_appearance, font) = if self.state.active == key {
                (appearance.active, &self.font_active)
            } else if state.hovered == key {
                (appearance.hover, &self.font_hovered)
            } else {
                (appearance.inactive, &self.font_inactive)
            };

            let x = bounds.center_x();
            let y = bounds.center_y();

            let button_appearance = if nth == 0 {
                status_appearance.first
            } else if nth + 1 == button_amount {
                status_appearance.last
            } else {
                status_appearance.middle
            };

            // Render the background of the button.
            if status_appearance.background.is_some() {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border_radius: button_appearance.border_radius,
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    status_appearance
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
                color: status_appearance.text_color,
                font: font.clone(),
                horizontal_alignment: alignment::Horizontal::Center,
                vertical_alignment: alignment::Vertical::Center,
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

impl<'a, Variant, Message, Renderer> From<SegmentedButton<'a, Variant, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer + iced_native::text::Renderer + 'a,
    Renderer::Theme: StyleSheet,
    SegmentedButton<'a, Variant, Message, Renderer>: SegmentedVariant<Renderer = Renderer>,
    Variant: 'static,
    Message: 'static + Clone,
{
    fn from(mut widget: SegmentedButton<'a, Variant, Message, Renderer>) -> Self {
        if widget.state.buttons.is_empty() {
            widget.spacing = 0;
        }

        Self::new(widget)
    }
}

/// State that is maintained by each individual widget.
#[derive(Default)]
struct UniqueWidgetState {
    /// The ID of the button that is being hovered. Defaults to null.
    hovered: Key,
}
