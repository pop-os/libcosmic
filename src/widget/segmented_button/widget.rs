// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::model::{Entity, Model, Selectable};
use crate::theme::{SegmentedButton as Style, THEME};
use crate::widget::{icon, Icon};
use crate::{Element, Renderer};
use derive_setters::Setters;
use iced::{
    alignment, event, keyboard, mouse, touch, Background, Color, Command, Event, Length, Rectangle,
    Size,
};
use iced_core::mouse::ScrollDelta;
use iced_core::text::{LineHeight, Paragraph, Renderer as TextRenderer, Shaping};
use iced_core::widget::{self, operation, tree};
use iced_core::{layout, renderer, widget::Tree, Clipboard, Layout, Shell, Widget};
use iced_core::{Point, Renderer as IcedRenderer, Text};
use slotmap::{Key, SecondaryMap};
use std::marker::PhantomData;
use std::time::{Duration, Instant};

/// A command that focuses a segmented item stored in a widget.
pub fn focus<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::focusable::focus(id.0))
}

/// Isolates variant-specific behaviors from [`SegmentedButton`].
pub trait SegmentedVariant {
    /// Get the appearance for this variant of the widget.
    fn variant_appearance(
        theme: &crate::Theme,
        style: &crate::theme::SegmentedButton,
    ) -> super::Appearance;

    /// Calculates the bounds for visible buttons.
    fn variant_button_bounds(
        &self,
        state: &LocalState,
        bounds: Rectangle,
    ) -> impl Iterator<Item = (Entity, Rectangle)>;

    /// Calculates the layout of this variant.
    fn variant_layout(
        &self,
        state: &mut LocalState,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node;
}

/// A conjoined group of items that function together as a button.
#[derive(Setters)]
#[must_use]
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
    /// Minimum width of a button.
    pub(super) minimum_button_width: u16,
    /// Spacing for each indent.
    pub(super) indent_spacing: u16,
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
    #[setters(skip)]
    pub(super) on_activate: Option<Box<dyn Fn(Entity) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) on_close: Option<Box<dyn Fn(Entity) -> Message + 'static>>,
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
    pub fn new(model: &'a Model<SelectionMode>) -> Self {
        Self {
            model,
            id: None,
            close_icon: icon::from_name("window-close-symbolic").size(16).icon(),
            show_close_icon_on_hover: false,
            button_padding: [4, 4, 4, 4],
            button_height: 32,
            button_spacing: 4,
            minimum_button_width: 150,
            indent_spacing: 16,
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

    /// Emitted when a tab is pressed.
    pub fn on_activate<T>(mut self, on_activate: T) -> Self
    where
        T: Fn(Entity) -> Message + 'static,
    {
        self.on_activate = Some(Box::new(on_activate));
        self
    }

    /// Emitted when a tab close button is pressed.
    pub fn on_close<T>(mut self, on_close: T) -> Self
    where
        T: Fn(Entity) -> Message + 'static,
    {
        self.on_close = Some(Box::new(on_close));
        self
    }

    /// Check if an item is enabled.
    fn is_enabled(&self, key: Entity) -> bool {
        self.model.items.get(key).map_or(false, |item| item.enabled)
    }

    /// Item the previous item in the widget.
    fn focus_previous(&mut self, state: &mut LocalState) -> event::Status {
        match state.focused_item {
            Item::Tab(entity) => {
                let mut keys = self.iterate_visible_tabs(state).rev();

                while let Some(key) = keys.next() {
                    if key == entity {
                        for key in keys {
                            // Skip disabled buttons.
                            if !self.is_enabled(key) {
                                continue;
                            }

                            state.focused_item = Item::Tab(key);
                            return event::Status::Captured;
                        }

                        break;
                    }
                }

                if self.prev_tab_sensitive(state) {
                    state.focused_item = Item::PrevButton;
                    return event::Status::Captured;
                }
            }

            Item::NextButton => {
                if let Some(last) = self.last_tab(state) {
                    state.focused_item = Item::Tab(last);
                    return event::Status::Captured;
                }
            }

            Item::None => {
                if self.next_tab_sensitive(state) {
                    state.focused_item = Item::NextButton;
                    return event::Status::Captured;
                } else if let Some(last) = self.last_tab(state) {
                    state.focused_item = Item::Tab(last);
                    return event::Status::Captured;
                }
            }

            Item::PrevButton | Item::Set => (),
        }

        state.focused_item = Item::None;
        event::Status::Ignored
    }

    /// Item the next item in the widget.
    fn focus_next(&mut self, state: &mut LocalState) -> event::Status {
        match state.focused_item {
            Item::Tab(entity) => {
                let mut keys = self.iterate_visible_tabs(state);
                while let Some(key) = keys.next() {
                    if key == entity {
                        for key in keys {
                            // Skip disabled buttons.
                            if !self.is_enabled(key) {
                                continue;
                            }

                            state.focused_item = Item::Tab(key);
                            return event::Status::Captured;
                        }

                        break;
                    }
                }

                if self.next_tab_sensitive(state) {
                    state.focused_item = Item::NextButton;
                    return event::Status::Captured;
                }
            }

            Item::PrevButton => {
                if let Some(first) = self.first_tab(state) {
                    state.focused_item = Item::Tab(first);
                    return event::Status::Captured;
                }
            }

            Item::None => {
                if self.prev_tab_sensitive(state) {
                    state.focused_item = Item::PrevButton;
                    return event::Status::Captured;
                } else if let Some(first) = self.first_tab(state) {
                    state.focused_item = Item::Tab(first);
                    return event::Status::Captured;
                }
            }

            Item::NextButton | Item::Set => (),
        }

        state.focused_item = Item::None;
        event::Status::Ignored
    }

    fn iterate_visible_tabs<'b>(
        &'b self,
        state: &LocalState,
    ) -> impl DoubleEndedIterator<Item = Entity> + 'b {
        self.model
            .order
            .iter()
            .copied()
            .skip(state.buttons_offset)
            .take(state.buttons_visible)
    }

    fn first_tab(&self, state: &LocalState) -> Option<Entity> {
        self.model.order.get(state.buttons_offset).copied()
    }

    fn last_tab(&self, state: &LocalState) -> Option<Entity> {
        self.model
            .order
            .get(state.buttons_offset + state.buttons_visible)
            .copied()
    }

    #[allow(clippy::unused_self)]
    fn prev_tab_sensitive(&self, state: &LocalState) -> bool {
        state.buttons_offset > 0
    }

    fn next_tab_sensitive(&self, state: &LocalState) -> bool {
        state.buttons_offset < self.model.order.len() - state.buttons_visible
    }

    pub(super) fn button_dimensions(
        &self,
        state: &mut LocalState,
        font: crate::font::Font,
        button: Entity,
    ) -> (f32, f32) {
        let mut width = 0.0f32;
        let mut height = 0.0f32;

        // Add text to measurement if text was given.
        if let Some((text, entry)) = self
            .model
            .text
            .get(button)
            .zip(state.paragraphs.entry(button))
        {
            let paragraph = entry.or_insert_with(|| {
                crate::Paragraph::with_text(Text {
                    content: text,
                    size: iced::Pixels(self.font_size),
                    bounds: Size::INFINITY,
                    font,
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: Shaping::Advanced,
                    line_height: self.line_height,
                })
            });

            let size = paragraph.min_bounds();
            width += size.width;
            height += size.height;
        }

        // Add indent to measurement if found.
        if let Some(indent) = self.model.indent(button) {
            width = f32::from(indent).mul_add(f32::from(self.indent_spacing), width);
        }

        // Add icon to measurement if icon was given.
        if let Some(icon) = self.model.icon(button) {
            height = height.max(f32::from(icon.size));
            width += f32::from(icon.size) + f32::from(self.button_spacing);
        }

        // Add close button to measurement if found.
        if self.model.is_closable(button) {
            height = height.max(f32::from(self.close_icon.size));
            width += f32::from(self.close_icon.size) + f32::from(self.button_spacing) + 8.0;
        }

        // Add button padding to the max size found
        width += f32::from(self.button_padding[0]) + f32::from(self.button_padding[2]);
        height += f32::from(self.button_padding[1]) + f32::from(self.button_padding[3]);
        height = height.max(f32::from(self.button_height));

        (width, height)
    }

    pub(super) fn max_button_dimensions(
        &self,
        state: &mut LocalState,
        renderer: &Renderer,
        _bounds: Size,
    ) -> (f32, f32) {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        let font = renderer.default_font();

        for key in self.model.order.iter().copied() {
            let (button_width, button_height) = self.button_dimensions(state, font, key);

            height = height.max(button_height);
            width = width.max(button_width);
        }

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
            paragraphs: SecondaryMap::new(),
            ..LocalState::default()
        })
    }

    fn diff(&mut self, tree: &mut Tree) {
        for e in self.model.order.iter().copied() {
            if let Some(text) = self.model.text.get(e) {
                let text = Text {
                    content: text,
                    size: iced::Pixels(self.font_size),
                    bounds: Size::INFINITY,
                    font: self.font_active.unwrap_or(crate::font::FONT),
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: Shaping::Advanced,
                    line_height: self.line_height,
                };
                if let Some(paragraph) = tree
                    .state
                    .downcast_mut::<LocalState>()
                    .paragraphs
                    .get_mut(e)
                {
                    paragraph.update(text);
                } else {
                    tree.state
                        .downcast_mut::<LocalState>()
                        .paragraphs
                        .insert(e, crate::Paragraph::with_text(text));
                }
            }
        }
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        self.height
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.variant_layout(tree.state.downcast_mut::<LocalState>(), renderer, limits)
    }

    #[allow(clippy::too_many_lines)]
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
            // Check for clicks on the previous and next tab buttons, when tabs are collapsed.
            if state.collapsed {
                // Check if the prev tab button was clicked.
                if cursor_position.is_over(Rectangle {
                    x: bounds.x,
                    y: bounds.y,
                    width: f32::from(self.button_height),
                    height: f32::from(self.button_height),
                }) && self.prev_tab_sensitive(state)
                {
                    state.hovered = Item::PrevButton;
                    if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                    | Event::Touch(touch::Event::FingerLifted { .. }) = event
                    {
                        state.buttons_offset -= 1;
                    }
                } else {
                    // Check if the next tab button was clicked.
                    if cursor_position.is_over(Rectangle {
                        x: bounds.x + bounds.width - f32::from(self.button_height),
                        y: bounds.y,
                        width: f32::from(self.button_height),
                        height: f32::from(self.button_height),
                    }) && self.next_tab_sensitive(state)
                    {
                        state.hovered = Item::NextButton;

                        if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                        | Event::Touch(touch::Event::FingerLifted { .. }) = event
                        {
                            state.buttons_offset += 1;
                        }
                    }
                }
            }

            for (key, bounds) in self
                .variant_button_bounds(state, bounds)
                .collect::<Vec<_>>()
            {
                if cursor_position.is_over(bounds) {
                    if self.model.items[key].enabled {
                        // Record that the mouse is hovering over this button.
                        state.hovered = Item::Tab(key);

                        // If marked as closable, show a close icon.
                        if self.model.items[key].closable {
                            // Emit close message if the close button is pressed.
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

                                // Emit close message if the tab is middle clicked.
                                if let Event::Mouse(mouse::Event::ButtonReleased(
                                    mouse::Button::Middle,
                                )) = event
                                {
                                    shell.publish(on_close(key));
                                    return event::Status::Captured;
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

            if let Some(on_activate) = self.on_activate.as_ref() {
                if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
                    let current = Instant::now();

                    // Permit successive scroll wheel events only after a given delay.
                    if state.wheel_timestamp.map_or(true, |previous| {
                        current.duration_since(previous) > Duration::from_millis(250)
                    }) {
                        state.wheel_timestamp = Some(current);

                        match delta {
                            ScrollDelta::Lines { y, .. } | ScrollDelta::Pixels { y, .. } => {
                                let mut activate_key = None;

                                if y < 0.0 {
                                    let mut prev_key = Entity::null();

                                    for key in self.model.order.iter().copied() {
                                        if self.model.is_active(key) && !prev_key.is_null() {
                                            activate_key = Some(prev_key);
                                        }

                                        if self.model.is_enabled(key) {
                                            prev_key = key;
                                        }
                                    }
                                } else if y > 0.0 {
                                    let mut buttons = self.model.order.iter().copied();
                                    while let Some(key) = buttons.next() {
                                        if self.model.is_active(key) {
                                            for key in buttons {
                                                if self.model.is_enabled(key) {
                                                    activate_key = Some(key);
                                                    break;
                                                }
                                            }
                                            break;
                                        }
                                    }
                                }

                                if let Some(key) = activate_key {
                                    shell.publish(on_activate(key));
                                    return event::Status::Captured;
                                }
                            }
                        }
                    }
                }
            }
        } else {
            state.hovered = Item::None;
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
                    match state.focused_item {
                        Item::Tab(entity) => {
                            shell.publish(on_activate(entity));
                        }

                        Item::PrevButton => {
                            if self.prev_tab_sensitive(state) {
                                state.buttons_offset -= 1;

                                // If the change would cause it to be insensitive, focus the first tab.
                                if !self.prev_tab_sensitive(state) {
                                    if let Some(first) = self.first_tab(state) {
                                        state.focused_item = Item::Tab(first);
                                    }
                                }
                            }
                        }

                        Item::NextButton => {
                            if self.next_tab_sensitive(state) {
                                state.buttons_offset += 1;

                                // If the change would cause it to be insensitive, focus the last tab.
                                if !self.next_tab_sensitive(state) {
                                    if let Some(last) = self.last_tab(state) {
                                        state.focused_item = Item::Tab(last);
                                    }
                                }
                            }
                        }

                        Item::None | Item::Set => (),
                    }

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

        if let Item::Set = state.focused_item {
            if self.prev_tab_sensitive(state) {
                state.focused_item = Item::PrevButton;
            } else if let Some(first) = self.first_tab(state) {
                state.focused_item = Item::Tab(first);
            }
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        _viewport: &iced::Rectangle,
        _renderer: &Renderer,
    ) -> iced_core::mouse::Interaction {
        let state = tree.state.downcast_ref::<LocalState>();
        let bounds = layout.bounds();

        if cursor_position.is_over(bounds) {
            for (key, bounds) in self.variant_button_bounds(state, bounds) {
                if cursor_position.is_over(bounds) {
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
        let cosmic_theme = theme.cosmic();
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

        // Draw previous and next tab buttons if there is a need to paginate tabs.
        if state.collapsed {
            // Previous tab button
            let mut background_appearance = if Item::PrevButton == state.focused_item {
                Some(appearance.focus)
            } else if Item::PrevButton == state.hovered {
                Some(appearance.hover)
            } else {
                None
            };

            if let Some(background_appearance) = background_appearance.take() {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y: bounds.y,
                            width: f32::from(self.button_height),
                            height: bounds.height,
                        },
                        border_radius: cosmic_theme.radius_s().into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    background_appearance
                        .background
                        .unwrap_or(Background::Color(Color::TRANSPARENT)),
                );
            }

            draw_icon::<Message>(
                renderer,
                theme,
                style,
                cursor,
                viewport,
                if state.buttons_offset == 0 {
                    appearance.inactive.text_color
                } else if let Item::PrevButton = state.focused_item {
                    appearance.focus.text_color
                } else {
                    appearance.active.text_color
                },
                Rectangle {
                    x: bounds.x + f32::from(self.button_height) / 4.0,
                    y: bounds.y + f32::from(self.button_height) / 4.0,
                    width: 16.0,
                    height: 16.0,
                },
                icon::from_name("go-previous-symbolic").size(16).icon(),
            );

            // Next tab button
            background_appearance = if Item::NextButton == state.focused_item {
                Some(appearance.focus)
            } else if Item::NextButton == state.hovered {
                Some(appearance.hover)
            } else {
                None
            };

            if let Some(background_appearance) = background_appearance {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + bounds.width - f32::from(self.button_height),
                            y: bounds.y,
                            width: f32::from(self.button_height),
                            height: bounds.height,
                        },
                        border_radius: cosmic_theme.radius_s().into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    background_appearance
                        .background
                        .unwrap_or(Background::Color(Color::TRANSPARENT)),
                );
            }

            draw_icon::<Message>(
                renderer,
                theme,
                style,
                cursor,
                viewport,
                if self.next_tab_sensitive(state) {
                    appearance.active.text_color
                } else if let Item::NextButton = state.focused_item {
                    appearance.focus.text_color
                } else {
                    appearance.inactive.text_color
                },
                Rectangle {
                    x: bounds.x + bounds.width - f32::from(self.button_height)
                        + f32::from(self.button_height) / 4.0,
                    y: bounds.y + f32::from(self.button_height) / 4.0,
                    width: 16.0,
                    height: 16.0,
                },
                icon::from_name("go-next-symbolic").size(16).icon(),
            );
        }

        // Draw each of the items in the widget.
        for (nth, (key, mut bounds)) in self.variant_button_bounds(state, bounds).enumerate() {
            let key_is_active = self.model.is_active(key);
            let key_is_hovered = state.hovered == Item::Tab(key);

            let (status_appearance, font) = if Item::Tab(key) == state.focused_item {
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

                let rad_0 = THEME.with(|t| t.borrow().cosmic().corner_radii.radius_0);
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border_radius: rad_0.into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    background,
                );
            }

            let original_bounds = bounds;

            let y = bounds.center_y();

            // Adjust bounds by indent
            if let Some(indent) = self.model.indent(key) {
                let adjustment = f32::from(indent) * f32::from(self.indent_spacing);
                bounds.x += adjustment;
                bounds.width -= adjustment;
            }

            // Draw the image beside the text.
            let horizontal_alignment = if let Some(icon) = self.model.icon(key) {
                bounds.x += f32::from(self.button_padding[0]);

                let mut image_bounds = bounds;
                let width = f32::from(icon.size);
                let offset = width + f32::from(self.button_spacing);
                image_bounds.y += f32::from(self.button_padding[1]);
                image_bounds.y = y - width / 2.0;

                draw_icon::<Message>(
                    renderer,
                    theme,
                    style,
                    cursor,
                    viewport,
                    status_appearance.text_color,
                    Rectangle {
                        width,
                        height: width,
                        ..image_bounds
                    },
                    icon.clone(),
                );

                bounds.x += offset;
                bounds.width -= offset;

                alignment::Horizontal::Left
            } else {
                bounds.x = bounds.center_x();
                alignment::Horizontal::Center
            };

            // Whether to show the close button on this tab.
            let show_close_button =
                (key_is_active || !self.show_close_icon_on_hover || key_is_hovered)
                    && self.model.is_closable(key);

            // Width of the icon used by the close button, which we will subtract from the text bounds.
            let close_icon_width = if show_close_button {
                f32::from(self.close_icon.size)
            } else {
                0.0
            };

            if let Some(text) = self.model.text(key) {
                bounds.y = y;

                // Draw the text for this segmented button or tab.
                renderer.fill_text(
                    iced_core::text::Text {
                        content: text,
                        size: iced::Pixels(self.font_size),
                        bounds: bounds.size(),
                        font,
                        horizontal_alignment,
                        vertical_alignment: alignment::Vertical::Center,
                        shaping: Shaping::Advanced,
                        line_height: self.line_height,
                    },
                    bounds.position(),
                    status_appearance.text_color,
                    Rectangle {
                        width: {
                            let width = bounds.width - close_icon_width;
                            // TODO: determine cause of differences here.
                            if self.model.icon(key).is_some() {
                                width - f32::from(self.button_spacing)
                            } else {
                                width - 12.0
                            }
                        },
                        ..original_bounds
                    },
                );
            }

            // Draw a close button if set.
            if show_close_button {
                let close_button_bounds =
                    close_bounds(original_bounds, close_icon_width, self.button_padding);

                draw_icon::<Message>(
                    renderer,
                    theme,
                    style,
                    cursor,
                    viewport,
                    status_appearance.text_color,
                    close_button_bounds,
                    self.close_icon.clone(),
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

/// State that is maintained by each individual widget.
#[derive(Default)]
pub struct LocalState {
    /// Defines how many buttons to show at a time.
    pub(super) buttons_visible: usize,
    /// Button visibility offset, when collapsed.
    pub(super) buttons_offset: usize,
    /// Whether buttons need to be collapsed to preserve minimum width
    pub(super) collapsed: bool,
    /// If the widget is focused or not.
    focused: bool,
    /// The key inside the widget that is currently focused.
    focused_item: Item,
    /// The ID of the button that is being hovered. Defaults to null.
    hovered: Item,
    /// Last known length of the model.
    pub(super) known_length: usize,
    /// Dimensions of internal buttons when shrinking
    pub(super) internal_layout: Vec<Size>,
    /// The paragraphs for each text.
    paragraphs: SecondaryMap<Entity, crate::Paragraph>,
    /// Time since last tab activation from wheel movements.
    wheel_timestamp: Option<Instant>,
}

#[derive(Default, PartialEq)]
enum Item {
    NextButton,
    #[default]
    None,
    PrevButton,
    Set,
    Tab(Entity),
}

impl operation::Focusable for LocalState {
    fn is_focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self) {
        self.focused = true;
        self.focused_item = Item::Set;
    }

    fn unfocus(&mut self) {
        self.focused = false;
        self.focused_item = Item::None;
    }
}

/// The iced identifier of a segmented button.
#[derive(Debug, Clone, PartialEq)]
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

#[allow(clippy::too_many_arguments)]
fn draw_icon<Message: 'static>(
    renderer: &mut Renderer,
    theme: &crate::Theme,
    style: &renderer::Style,
    cursor: mouse::Cursor,
    viewport: &Rectangle,
    color: Color,
    bounds: Rectangle,
    icon: Icon,
) {
    let mut layout_node = layout::Node::new(Size {
        width: bounds.width,
        height: bounds.width,
    });

    layout_node.move_to(Point {
        x: bounds.x,
        y: bounds.y,
    });

    Widget::<Message, Renderer>::draw(
        Element::<Message>::from(icon).as_widget(),
        &Tree::empty(),
        renderer,
        theme,
        &renderer::Style {
            icon_color: color,
            text_color: color,
            scale_factor: style.scale_factor,
        },
        Layout::new(&layout_node),
        cursor,
        viewport,
    );
}
