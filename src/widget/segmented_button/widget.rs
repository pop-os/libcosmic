// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::marker::PhantomData;

use super::model::{Entity, Model, Selectable};
use super::style::StyleSheet;

use derive_setters::Setters;
use iced::{
    alignment, event, keyboard, mouse, touch, Background, Color, Command, Element, Event, Length,
    Point, Rectangle, Size,
};
use iced_core::BorderRadius;
use iced_native::widget::{self, operation, tree, Operation};
use iced_native::{layout, renderer, widget::Tree, Clipboard, Layout, Shell, Widget};

/// State that is maintained by each individual widget.
#[derive(Default)]
struct LocalState {
    /// The first focusable key.
    first: Entity,
    /// If the widget is focused or not.
    focused: bool,
    /// The key inside the widget that is currently focused.
    focused_key: Entity,
    /// The ID of the button that is being hovered. Defaults to null.
    hovered: Entity,
}

impl operation::Focusable for LocalState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
        self.focused_key = self.first;
    }

    fn unfocus(&mut self) {
        self.focused = false;
        self.focused_key = Entity::default();
    }
}

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

/// A conjoined group of items that function together as a button.
#[derive(Setters)]
pub struct SegmentedButton<'a, Variant, SelectionMode, Message, Renderer>
where
    Renderer: iced_native::Renderer
        + iced_native::text::Renderer
        + iced_native::image::Renderer
        + iced_native::svg::Renderer,
    Renderer::Theme: StyleSheet,
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
{
    /// The model borrowed from the application create this widget.
    #[setters(skip)]
    pub(super) model: &'a Model<SelectionMode>,
    /// iced widget ID
    pub(super) id: Option<Id>,
    /// Padding around a button.
    pub(super) button_padding: [u16; 4],
    /// Desired height of a button.
    pub(super) button_height: u16,
    /// Spacing between icon and text in button.
    pub(super) button_spacing: u16,
    /// Desired font for active tabs.
    pub(super) font_active: Renderer::Font,
    /// Desired font for hovered tabs.
    pub(super) font_hovered: Renderer::Font,
    /// Desired font for inactive tabs.
    pub(super) font_inactive: Renderer::Font,
    /// Size of the font.
    pub(super) font_size: u16,
    /// Size of icon
    pub(super) icon_size: u16,
    /// Desired width of the widget.
    pub(super) width: Length,
    /// Desired height of the widget.
    pub(super) height: Length,
    /// Desired spacing between items.
    pub(super) spacing: u16,
    /// Style to draw the widget in.
    #[setters(into)]
    pub(super) style: <Renderer::Theme as StyleSheet>::Style,
    #[setters(skip)]
    /// Emits the ID of the activated widget on selection.
    pub(super) on_activate: Option<Box<dyn Fn(Entity) -> Message>>,
    #[setters(skip)]
    /// Defines the implementation of this struct
    variant: PhantomData<Variant>,
}

impl<'a, Variant, SelectionMode, Message, Renderer>
    SegmentedButton<'a, Variant, SelectionMode, Message, Renderer>
where
    Renderer: iced_native::Renderer
        + iced_native::text::Renderer
        + iced_native::image::Renderer
        + iced_native::svg::Renderer,
    Renderer::Theme: StyleSheet,
    Self: SegmentedVariant<Renderer = Renderer>,
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
{
    #[must_use]
    pub fn new(model: &'a Model<SelectionMode>) -> Self {
        Self {
            model,
            id: None,
            button_padding: [4, 4, 4, 4],
            button_height: 32,
            button_spacing: 4,
            font_active: Renderer::Font::default(),
            font_hovered: Renderer::Font::default(),
            font_inactive: Renderer::Font::default(),
            font_size: 17,
            icon_size: 16,
            height: Length::Shrink,
            width: Length::Fill,
            spacing: 0,
            style: <Renderer::Theme as StyleSheet>::Style::default(),
            on_activate: None,
            variant: PhantomData,
        }
    }

    /// Check if an item is enabled.
    fn is_enabled(&self, key: Entity) -> bool {
        self.model.items.get(key).map_or(false, |item| item.enabled)
    }

    /// Focus the previous item in the widget.
    fn focus_previous(&mut self, state: &mut LocalState) -> event::Status {
        let mut keys = self.model.order.iter().copied().rev();

        while let Some(key) = keys.next() {
            if key == state.focused_key {
                for key in keys {
                    // Skip disabled buttons.
                    if !self.is_enabled(key) {
                        continue;
                    }

                    state.focused_key = key;
                    return event::Status::Captured;
                }

                break;
            }
        }

        state.focused_key = Entity::default();
        event::Status::Ignored
    }

    /// Focus the next item in the widget.
    fn focus_next(&mut self, state: &mut LocalState) -> event::Status {
        let mut keys = self.model.order.iter().copied();

        while let Some(key) = keys.next() {
            if key == state.focused_key {
                for key in keys {
                    // Skip disabled buttons.
                    if !self.is_enabled(key) {
                        continue;
                    }

                    state.focused_key = key;
                    return event::Status::Captured;
                }

                break;
            }
        }

        state.focused_key = Entity::default();
        event::Status::Ignored
    }

    /// Emits the ID of the activated widget on selection.
    #[must_use]
    pub fn on_activate(mut self, on_activate: impl Fn(Entity) -> Message + 'static) -> Self {
        self.on_activate = Some(Box::from(on_activate));
        self
    }

    pub(super) fn max_button_dimensions(&self, renderer: &Renderer, bounds: Size) -> (f32, f32) {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        for key in self.model.order.iter().copied() {
            let mut button_width = 0.0f32;
            let mut button_height = 0.0f32;

            // Add text to measurement if text was given.
            if let Some(text) = self.model.text(key) {
                let (w, h) = renderer.measure(text, self.font_size, Default::default(), bounds);

                button_width = w;
                button_height = h;
            }

            // Add icon to measurement if icon was given.
            if self.model.icon(key).is_some() {
                button_width += f32::from(self.icon_size) + f32::from(self.button_spacing);
                button_height = f32::from(self.icon_size);
            }

            height = height.max(button_height);
            width = width.max(button_width);
        }

        // Add button padding to the max size found
        width += f32::from(self.button_padding[0]) + f32::from(self.button_padding[2]);
        height += f32::from(self.button_padding[1]) + f32::from(self.button_padding[3]);
        height = height.max(f32::from(self.button_height));

        (width, height)
    }
}

impl<'a, Variant, SelectionMode, Message, Renderer> Widget<Message, Renderer>
    for SegmentedButton<'a, Variant, SelectionMode, Message, Renderer>
where
    Renderer: iced_native::Renderer
        + iced_native::text::Renderer
        + iced_native::image::Renderer
        + iced_native::svg::Renderer,
    Renderer::Theme: StyleSheet,
    Self: SegmentedVariant<Renderer = Renderer>,
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
    Message: 'static + Clone,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<LocalState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(LocalState {
            first: self.model.order.iter().copied().next().unwrap_or_default(),
            ..LocalState::default()
        })
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
        let state = tree.state.downcast_mut::<LocalState>();

        if bounds.contains(cursor_position) {
            for (nth, key) in self.model.order.iter().copied().enumerate() {
                let bounds = self.variant_button_bounds(bounds, nth);
                if bounds.contains(cursor_position) {
                    if self.model.items[key].enabled {
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

                    break;
                }
            }
        } else {
            state.hovered = Entity::default();
        }

        if state.focused {
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key_code: keyboard::KeyCode::Tab,
                modifiers,
                ..
            }) = event
            {
                return if modifiers.shift() {
                    self.focus_previous(state)
                } else {
                    self.focus_next(state)
                };
            }

            if let Some(on_activate) = self.on_activate.as_ref() {
                if let Event::Keyboard(keyboard::Event::KeyReleased {
                    key_code: keyboard::KeyCode::Enter,
                    ..
                }) = event
                {
                    shell.publish(on_activate(state.focused_key));
                    return event::Status::Captured;
                }
            }
        }

        event::Status::Ignored
    }

    fn operate(
        &self,
        tree: &mut Tree,
        _layout: Layout<'_>,
        operation: &mut dyn Operation<Message>,
    ) {
        let state = tree.state.downcast_mut::<LocalState>();
        operation.focusable(state, self.id.as_ref().map(|id| &id.0));
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

        if bounds.contains(cursor_position) {
            for (nth, key) in self.model.order.iter().copied().enumerate() {
                if self
                    .variant_button_bounds(bounds, nth)
                    .contains(cursor_position)
                {
                    return if self.model.items[key].enabled {
                        iced_native::mouse::Interaction::Pointer
                    } else {
                        iced_native::mouse::Interaction::Idle
                    };
                }
            }
        }

        iced_native::mouse::Interaction::Idle
    }

    #[allow(clippy::too_many_lines)]
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
        let state = tree.state.downcast_ref::<LocalState>();
        let appearance = Self::variant_appearance(theme, &self.style);
        let bounds = layout.bounds();
        let button_amount = self.model.items.len();

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

        // Draw each of the items in the widget.
        for (nth, key) in self.model.order.iter().copied().enumerate() {
            let mut bounds = self.variant_button_bounds(bounds, nth);

            let (status_appearance, font) = if state.focused_key == key {
                (appearance.focus, &self.font_active)
            } else if self.model.is_active(key) {
                (appearance.active, &self.font_active)
            } else if state.hovered == key {
                (appearance.hover, &self.font_hovered)
            } else {
                (appearance.inactive, &self.font_inactive)
            };

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

            let y = bounds.center_y();

            // Draw the image beside the text.
            let horizontal_alignment = if let Some(icon) = self.model.icon(key) {
                bounds.x += f32::from(self.button_padding[0]);
                bounds.y += f32::from(self.button_padding[1]);
                bounds.width -=
                    f32::from(self.button_padding[0]) - f32::from(self.button_padding[2]);
                bounds.height -=
                    f32::from(self.button_padding[1]) - f32::from(self.button_padding[3]);

                let width = f32::from(self.icon_size);
                let offset = width + f32::from(self.button_spacing);
                bounds.y = y - width / 2.0;

                let icon_bounds = Rectangle {
                    width,
                    height: width,
                    ..bounds
                };

                bounds.x += offset;
                bounds.width -= offset;
                match icon.load(self.icon_size, None, false) {
                    crate::widget::icon::Handle::Image(_handle) => {
                        unimplemented!()
                    }
                    crate::widget::icon::Handle::Svg(handle) => {
                        iced_native::svg::Renderer::draw(
                            renderer,
                            handle,
                            Some(status_appearance.text_color),
                            icon_bounds,
                        );
                    }
                }

                alignment::Horizontal::Left
            } else {
                bounds.x = bounds.center_x();
                alignment::Horizontal::Center
            };

            if let Some(text) = self.model.text(key) {
                bounds.y = y;

                // Draw the text in this button.
                renderer.fill_text(iced_native::text::Text {
                    content: text,
                    size: f32::from(self.font_size),
                    bounds,
                    color: status_appearance.text_color,
                    font: font.clone(),
                    horizontal_alignment,
                    vertical_alignment: alignment::Vertical::Center,
                });
            }
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

impl<'a, Variant, SelectionMode, Message, Renderer>
    From<SegmentedButton<'a, Variant, SelectionMode, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: iced_native::Renderer
        + iced_native::text::Renderer
        + iced_native::image::Renderer
        + iced_native::svg::Renderer
        + 'a,
    Renderer::Theme: StyleSheet,
    SegmentedButton<'a, Variant, SelectionMode, Message, Renderer>:
        SegmentedVariant<Renderer = Renderer>,
    Variant: 'static,
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
    Message: 'static + Clone,
{
    fn from(mut widget: SegmentedButton<'a, Variant, SelectionMode, Message, Renderer>) -> Self {
        if widget.model.items.is_empty() {
            widget.spacing = 0;
        }

        Self::new(widget)
    }
}

/// A command that focuses a segmented item stored in a widget.
#[must_use]
pub fn focus<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::focusable::focus(id.0))
}

/// The iced identifier of a segmented button.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Id(widget::Id);

impl Id {
    /// Creates a custom [`Id`].
    pub fn new(id: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        Self(widget::Id::new(id))
    }

    /// Creates a unique [`Id`].
    ///
    /// This function produces a different [`Id`] every time it is called.
    #[must_use]
    pub fn unique() -> Self {
        Self(widget::Id::unique())
    }
}

impl From<Id> for widget::Id {
    fn from(id: Id) -> Self {
        id.0
    }
}
