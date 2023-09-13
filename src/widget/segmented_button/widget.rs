// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::model::{Entity, Model, Selectable};
use crate::theme::SegmentedButton as Style;
use crate::widget::{icon, Icon};
use crate::{Element, Renderer};
use derive_setters::Setters;
use iced::{
    alignment, event, keyboard, mouse, touch, Background, Color, Command, Event, Length, Rectangle,
    Size,
};
use iced_core::text::{LineHeight, Renderer as TextRenderer, Shaping};
use iced_core::widget::{self, operation, tree};
use iced_core::{layout, renderer, widget::Tree, Clipboard, Layout, Shell, Widget};
use iced_core::{BorderRadius, Point, Renderer as IcedRenderer};
use std::marker::PhantomData;

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
    /// Get the appearance for this variant of the widget.
    fn variant_appearance(
        theme: &crate::Theme,
        style: &crate::theme::SegmentedButton,
    ) -> super::Appearance;

    /// Calculates the bounds for the given button by its position.
    fn variant_button_bounds(&self, bounds: Rectangle, position: usize) -> Rectangle;

    /// Calculates the layout of this variant.
    fn variant_layout(&self, renderer: &crate::Renderer, limits: &layout::Limits) -> layout::Node;
}

/// A conjoined group of items that function together as a button.
#[derive(Setters)]
pub struct SegmentedButton<'a, Variant, SelectionMode, Message>
where
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
{
    /// The model borrowed from the application create this widget.
    #[setters(skip)]
    pub(super) model: &'a Model<SelectionMode>,
    /// iced widget ID
    pub(super) id: Option<Id>,
    /// The icon used for the close button.
    pub(super) close_icon: Icon,
    /// Show the close icon only when item is hovered.
    pub(super) show_close_icon_on_hover: bool,
    /// Padding around a button.
    pub(super) button_padding: [u16; 4],
    /// Desired height of a button.
    pub(super) button_height: u16,
    /// Spacing between icon and text in button.
    pub(super) button_spacing: u16,
    /// Desired font for active tabs.
    pub(super) font_active: Option<crate::font::Font>,
    /// Desired font for hovered tabs.
    pub(super) font_hovered: Option<crate::font::Font>,
    /// Desired font for inactive tabs.
    pub(super) font_inactive: Option<crate::font::Font>,
    /// Size of the font.
    pub(super) font_size: f32,
    /// Desired width of the widget.
    pub(super) width: Length,
    /// Desired height of the widget.
    pub(super) height: Length,
    /// Desired spacing between items.
    pub(super) spacing: u16,
    /// LineHeight of the font.
    pub(super) line_height: LineHeight,
    /// Style to draw the widget in.
    #[setters(into)]
    pub(super) style: Style,
    /// Emits the ID of the item that was activated.
    #[setters(strip_option)]
    pub(super) on_activate: Option<fn(Entity) -> Message>,
    #[setters(strip_option)]
    pub(super) on_close: Option<fn(Entity) -> Message>,
    #[setters(skip)]
    /// Defines the implementation of this struct
    variant: PhantomData<Variant>,
}

impl<'a, Variant, SelectionMode, Message> SegmentedButton<'a, Variant, SelectionMode, Message>
where
    Self: SegmentedVariant,
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
{
    #[must_use]
    pub fn new(model: &'a Model<SelectionMode>) -> Self {
        Self {
            model,
            id: None,
            close_icon: icon::from_name("window-close-symbolic").size(16).icon(),
            show_close_icon_on_hover: false,
            button_padding: [4, 4, 4, 4],
            button_height: 32,
            button_spacing: 4,
            font_active: None,
            font_hovered: None,
            font_inactive: None,
            font_size: 14.0,
            height: Length::Shrink,
            width: Length::Fill,
            spacing: 0,
            line_height: LineHeight::default(),
            style: Style::default(),
            on_activate: None,
            on_close: None,
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

    pub(super) fn max_button_dimensions(&self, renderer: &Renderer, bounds: Size) -> (f32, f32) {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        let font = renderer.default_font();

        for key in self.model.order.iter().copied() {
            let mut button_width = 0.0f32;
            let mut button_height = 0.0f32;

            // Add text to measurement if text was given.
            if let Some(text) = self.model.text(key) {
                let Size { width, height } = renderer.measure(
                    text,
                    self.font_size,
                    self.line_height,
                    font,
                    bounds,
                    Shaping::Advanced,
                );

                button_width = width;
                button_height = height;
            }

            // Add icon to measurement if icon was given.
            if let Some(icon) = self.model.icon(key) {
                button_height = button_height.max(f32::from(icon.size));
                button_width += f32::from(icon.size) + f32::from(self.button_spacing);
            }

            // Add close button to measurement if found.
            if self.model.is_closable(key) {
                button_height = button_height.max(f32::from(self.close_icon.size));
                button_width +=
                    f32::from(self.close_icon.size) + f32::from(self.button_spacing) + 8.0;
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

impl<'a, Variant, SelectionMode, Message> Widget<Message, Renderer>
    for SegmentedButton<'a, Variant, SelectionMode, Message>
where
    Self: SegmentedVariant,
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
        cursor_position: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) -> event::Status {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<LocalState>();

        if cursor_position.is_over(bounds) {
            for (nth, key) in self.model.order.iter().copied().enumerate() {
                let bounds = self.variant_button_bounds(bounds, nth);
                if cursor_position.is_over(bounds) {
                    if self.model.items[key].enabled {
                        // Record that the mouse is hovering over this button.
                        state.hovered = key;

                        // If marked as closable, show a close icon.
                        if self.model.items[key].closable {
                            if let Some(on_close) = self.on_close.as_ref() {
                                if cursor_position.is_over(close_bounds(
                                    bounds,
                                    f32::from(self.close_icon.size),
                                    self.button_padding,
                                )) {
                                    if let Event::Mouse(mouse::Event::ButtonReleased(
                                        mouse::Button::Left,
                                    ))
                                    | Event::Touch(touch::Event::FingerLifted { .. }) = event
                                    {
                                        shell.publish(on_close(key));
                                        return event::Status::Captured;
                                    }
                                }
                            }
                        }

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
        _renderer: &Renderer,
        operation: &mut dyn iced_core::widget::Operation<
            iced_core::widget::OperationOutputWrapper<Message>,
        >,
    ) {
        let state = tree.state.downcast_mut::<LocalState>();
        operation.focusable(state, self.id.as_ref().map(|id| &id.0));
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> iced_core::mouse::Interaction {
        let bounds = layout.bounds();

        if cursor_position.is_over(bounds) {
            for (nth, key) in self.model.order.iter().copied().enumerate() {
                if cursor_position.is_over(self.variant_button_bounds(bounds, nth)) {
                    return if self.model.items[key].enabled {
                        iced_core::mouse::Interaction::Pointer
                    } else {
                        iced_core::mouse::Interaction::Idle
                    };
                }
            }
        }

        iced_core::mouse::Interaction::Idle
    }

    #[allow(clippy::too_many_lines)]
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
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

            let key_is_active = self.model.is_active(key);
            let key_is_hovered = state.hovered == key;

            let (status_appearance, font) = if state.focused_key == key {
                (appearance.focus, &self.font_active)
            } else if key_is_active {
                (appearance.active, &self.font_active)
            } else if key_is_hovered {
                (appearance.hover, &self.font_hovered)
            } else {
                (appearance.inactive, &self.font_inactive)
            };
            let font = font.unwrap_or_else(|| renderer.default_font());

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

            let original_bounds = bounds;

            let y = bounds.center_y();

            // Draw the image beside the text.
            let horizontal_alignment = if let Some(icon) = self.model.icon(key) {
                bounds.x += f32::from(self.button_padding[0]);
                bounds.y += f32::from(self.button_padding[1]);
                bounds.width -=
                    f32::from(self.button_padding[0]) - f32::from(self.button_padding[2]);
                bounds.height -=
                    f32::from(self.button_padding[1]) - f32::from(self.button_padding[3]);

                let width = f32::from(icon.size);
                let offset = width + f32::from(self.button_spacing);
                bounds.y = y - width / 2.0;

                let mut layout_node = layout::Node::new(Size {
                    width,
                    height: width - offset,
                });
                layout_node.move_to(Point {
                    x: bounds.x + offset,
                    y: bounds.y,
                });

                Widget::<Message, Renderer>::draw(
                    &Element::<Message>::from(icon.clone()),
                    &Tree::empty(),
                    renderer,
                    theme,
                    style,
                    Layout::new(&layout_node),
                    cursor,
                    viewport,
                );

                alignment::Horizontal::Left
            } else {
                bounds.x = bounds.center_x();
                alignment::Horizontal::Center
            };

            if let Some(text) = self.model.text(key) {
                bounds.y = y;

                // Draw the text in this button.
                renderer.fill_text(iced_core::text::Text {
                    content: text,
                    size: self.font_size,
                    bounds,
                    color: status_appearance.text_color,
                    font,
                    horizontal_alignment,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: Shaping::Advanced,
                    line_height: self.line_height,
                });
            }

            let show_close_button =
                (key_is_active || !self.show_close_icon_on_hover || key_is_hovered)
                    && self.model.is_closable(key);

            // Draw a close button if this is set.
            if show_close_button {
                let width = f32::from(self.close_icon.size);
                let icon_bounds = close_bounds(original_bounds, width, self.button_padding);
                let mut layout_node = layout::Node::new(Size {
                    width: icon_bounds.width,
                    height: icon_bounds.height,
                });
                layout_node.move_to(Point {
                    x: icon_bounds.x,
                    y: icon_bounds.y,
                });

                Widget::<Message, Renderer>::draw(
                    &Element::<Message>::from(self.close_icon.clone()),
                    &Tree::empty(),
                    renderer,
                    theme,
                    style,
                    Layout::new(&layout_node),
                    cursor,
                    viewport,
                );
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        _tree: &'b mut Tree,
        _layout: iced_core::Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<iced_core::overlay::Element<'b, Message, Renderer>> {
        None
    }
}

impl<'a, Variant, SelectionMode, Message> From<SegmentedButton<'a, Variant, SelectionMode, Message>>
    for Element<'a, Message>
where
    SegmentedButton<'a, Variant, SelectionMode, Message>: SegmentedVariant,
    Variant: 'static,
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
    Message: 'static + Clone,
{
    fn from(mut widget: SegmentedButton<'a, Variant, SelectionMode, Message>) -> Self {
        if widget.model.items.is_empty() {
            widget.spacing = 0;
        }

        Self::new(widget)
    }
}

/// A command that focuses a segmented item stored in a widget.
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

/// Calculates the bounds of the close button within the area of an item.
fn close_bounds(area: Rectangle<f32>, icon_size: f32, button_padding: [u16; 4]) -> Rectangle<f32> {
    let unpadded_height = area.height - f32::from(button_padding[1]) - f32::from(button_padding[3]);

    Rectangle {
        x: area.x + area.width - icon_size - 8.0,
        y: area.y + (unpadded_height / 2.0) - (icon_size / 2.0),
        width: icon_size,
        height: icon_size,
    }
}
