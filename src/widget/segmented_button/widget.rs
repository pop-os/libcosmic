// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::model::{Entity, Model, Selectable};
use crate::theme::{SegmentedButton as Style, THEME};
use crate::widget::{icon, Icon};
use crate::{Element, Renderer};
use derive_setters::Setters;
use iced::{
    alignment, event, keyboard, mouse, touch, Alignment, Background, Color, Command, Event, Length,
    Padding, Rectangle, Size,
};
use iced_core::mouse::ScrollDelta;
use iced_core::text::{LineHeight, Paragraph, Renderer as TextRenderer, Shaping};
use iced_core::widget::{self, operation, tree};
use iced_core::{layout, renderer, widget::Tree, Clipboard, Layout, Shell, Widget};
use iced_core::{Border, Gradient, Point, Renderer as IcedRenderer, Shadow, Text};
use slotmap::{Key, SecondaryMap};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::time::{Duration, Instant};

/// A command that focuses a segmented item stored in a widget.
pub fn focus<Message: 'static>(id: Id) -> Command<Message> {
    Command::widget(operation::focusable::focus(id.0))
}

pub enum ItemBounds {
    Button(Entity, Rectangle),
    Divider(Rectangle),
}

/// Isolates variant-specific behaviors from [`SegmentedButton`].
pub trait SegmentedVariant {
    /// Get the appearance for this variant of the widget.
    fn variant_appearance(
        theme: &crate::Theme,
        style: &crate::theme::SegmentedButton,
    ) -> super::Appearance;

    /// Calculates the bounds for visible buttons.
    fn variant_bounds<'b>(
        &'b self,
        state: &'b LocalState,
        bounds: Rectangle,
    ) -> Box<dyn Iterator<Item = ItemBounds> + 'b>;

    /// Calculates the layout of this variant.
    fn variant_layout(
        &self,
        state: &mut LocalState,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> Size;
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
    /// Scrolling switches focus between tabs.
    pub(super) scrollable_focus: bool,
    /// Show the close icon only when item is hovered.
    pub(super) show_close_icon_on_hover: bool,
    /// Padding of the whole widget.
    #[setters(into)]
    pub(super) padding: Padding,
    /// Whether to place dividers between buttons.
    pub(super) dividers: bool,
    /// Alignment of button contents.
    pub(super) button_alignment: Alignment,
    /// Padding around a button.
    pub(super) button_padding: [u16; 4],
    /// Desired height of a button.
    pub(super) button_height: u16,
    /// Spacing between icon and text in button.
    pub(super) button_spacing: u16,
    /// Maximum width of a button.
    pub(super) maximum_button_width: u16,
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
            scrollable_focus: false,
            show_close_icon_on_hover: false,
            button_alignment: Alignment::Start,
            padding: Padding::from(0.0),
            dividers: false,
            button_padding: [0, 0, 0, 0],
            button_height: 32,
            button_spacing: 0,
            minimum_button_width: u16::MIN,
            maximum_button_width: u16::MAX,
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
        let mut icon_spacing = 0.0f32;

        // Add text to measurement if text was given.
        if let Some((text, entry)) = self
            .model
            .text
            .get(button)
            .zip(state.paragraphs.entry(button))
        {
            if !text.is_empty() {
                icon_spacing = f32::from(self.button_spacing);
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
            }
        }

        // Add indent to measurement if found.
        if let Some(indent) = self.model.indent(button) {
            width = f32::from(indent).mul_add(f32::from(self.indent_spacing), width);
        }

        // Add icon to measurement if icon was given.
        if let Some(icon) = self.model.icon(button) {
            width += f32::from(icon.size) + icon_spacing;
        } else if self.model.is_active(button) {
            // Add selection icon measurements when widget is a selection widget.
            if let crate::theme::SegmentedButton::Control = self.style {
                width += 16.0 + icon_spacing;
            }
        }

        // Add close button to measurement if found.
        if self.model.is_closable(button) {
            width += f32::from(self.close_icon.size) + f32::from(self.button_spacing);
        }

        // Add button padding to the max size found
        width += f32::from(self.button_padding[0]) + f32::from(self.button_padding[2]);
        width = width.min(f32::from(self.maximum_button_width));

        (width, f32::from(self.button_height))
    }

    pub(super) fn max_button_dimensions(
        &self,
        state: &mut LocalState,
        renderer: &Renderer,
    ) -> (f32, f32) {
        let mut width = 0.0f32;
        let mut height = 0.0f32;
        let font = renderer.default_font();

        for key in self.model.order.iter().copied() {
            let (button_width, button_height) = self.button_dimensions(state, font, key);

            state.internal_layout.push((
                Size::new(button_width, button_height),
                Size::new(
                    button_width
                        - f32::from(self.button_padding[0])
                        - f32::from(self.button_padding[2]),
                    button_height,
                ),
            ));

            height = height.max(button_height);
            width = width.max(button_width);
        }

        for (size, actual) in &mut state.internal_layout {
            size.height = height;
            actual.height = height;
        }

        (width, height)
    }

    fn button_is_focused(&self, state: &LocalState, key: Entity) -> bool {
        self.on_activate.is_some() && Item::Tab(key) == state.focused_item
    }

    fn button_is_hovered(&self, state: &LocalState, key: Entity) -> bool {
        self.on_activate.is_some() && state.hovered == Item::Tab(key)
    }
}

impl<'a, Variant, SelectionMode, Message> Widget<Message, crate::Theme, Renderer>
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
            text_hashes: SecondaryMap::new(),
            ..LocalState::default()
        })
    }

    fn diff(&mut self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<LocalState>();
        for key in self.model.order.iter().copied() {
            if let Some(text) = self.model.text.get(key) {
                let (font, button_state) =
                    if self.model.is_active(key) || self.button_is_focused(state, key) {
                        (self.font_active, 0)
                    } else if self.button_is_hovered(state, key) {
                        (self.font_hovered, 1)
                    } else {
                        (self.font_inactive, 2)
                    };

                let mut hasher = DefaultHasher::new();
                text.hash(&mut hasher);
                button_state.hash(&mut hasher);
                let text_hash = hasher.finish();

                if let Some(prev_hash) = state.text_hashes.insert(key, text_hash) {
                    if prev_hash == text_hash {
                        continue;
                    }
                }

                let text = Text {
                    content: text,
                    size: iced::Pixels(self.font_size),
                    bounds: Size::INFINITY,
                    font: font.unwrap_or(crate::font::FONT),
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: Shaping::Advanced,
                    line_height: self.line_height,
                };

                if let Some(paragraph) = state.paragraphs.get_mut(key) {
                    paragraph.update(text);
                } else {
                    state
                        .paragraphs
                        .insert(key, crate::Paragraph::with_text(text));
                }
            }
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<LocalState>();
        let limits = limits.shrink(self.padding);
        let size = self
            .variant_layout(state, renderer, &limits)
            .expand(self.padding);
        layout::Node::new(size)
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
        state.hovered = Item::None;

        if cursor_position.is_over(bounds) {
            // Check for clicks on the previous and next tab buttons, when tabs are collapsed.
            if state.collapsed {
                // Check if the prev tab button was clicked.
                if cursor_position.is_over(prev_tab_bounds(&bounds, f32::from(self.button_height)))
                    && self.prev_tab_sensitive(state)
                {
                    state.hovered = Item::PrevButton;
                    if let Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                    | Event::Touch(touch::Event::FingerLifted { .. }) = event
                    {
                        state.buttons_offset -= 1;
                    }
                } else {
                    // Check if the next tab button was clicked.
                    if cursor_position
                        .is_over(next_tab_bounds(&bounds, f32::from(self.button_height)))
                        && self.next_tab_sensitive(state)
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
                .variant_bounds(state, bounds)
                .filter_map(|item| match item {
                    ItemBounds::Button(entity, bounds) => Some((entity, bounds)),
                    _ => None,
                })
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
                                if cursor_position
                                    .is_over(close_bounds(bounds, f32::from(self.close_icon.size)))
                                {
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

            if self.scrollable_focus {
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
            }
        }

        if state.focused {
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Tab),
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
                    key: keyboard::Key::Named(keyboard::key::Named::Enter),
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
        if self.on_activate.is_none() {
            return iced_core::mouse::Interaction::default();
        }
        let state = tree.state.downcast_ref::<LocalState>();
        let bounds = layout.bounds();

        if cursor_position.is_over(bounds) {
            let hovered_button = self
                .variant_bounds(state, bounds)
                .filter_map(|item| match item {
                    ItemBounds::Button(entity, bounds) => Some((entity, bounds)),
                    _ => None,
                })
                .find(|(_key, bounds)| cursor_position.is_over(*bounds));

            if let Some((key, _bounds)) = hovered_button {
                return if self.model.items[key].enabled {
                    iced_core::mouse::Interaction::Pointer
                } else {
                    iced_core::mouse::Interaction::Idle
                };
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
        let bounds: Rectangle = layout.bounds();
        let button_amount = self.model.items.len();

        // Modifies alpha color when `on_activate` is unset.
        let apply_alpha = |mut c: Color| {
            if self.on_activate.is_none() {
                c.a /= 2.0;
            }

            c
        };

        // Maps `apply_alpha` to background color.
        let bg_with_alpha = |mut b| {
            match &mut b {
                Background::Color(c) => {
                    *c = apply_alpha(*c);
                }

                Background::Gradient(g) => {
                    let Gradient::Linear(mut l) = g;
                    for c in &mut l.stops {
                        let Some(stop) = c else {
                            continue;
                        };
                        stop.color = apply_alpha(stop.color);
                    }
                }
            }
            b
        };

        // Draw the background, if a background was defined.
        if let Some(background) = appearance.background {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: Border {
                        radius: appearance.border_radius,
                        ..Border::default()
                    },
                    shadow: Shadow::default(),
                },
                bg_with_alpha(background),
            );
        }

        // Draw previous and next tab buttons if there is a need to paginate tabs.
        if state.collapsed {
            let mut tab_bounds = prev_tab_bounds(&bounds, f32::from(self.button_height));

            // Previous tab button
            let mut background_appearance =
                if self.on_activate.is_some() && Item::PrevButton == state.focused_item {
                    Some(appearance.focus)
                } else if self.on_activate.is_some() && Item::PrevButton == state.hovered {
                    Some(appearance.hover)
                } else {
                    None
                };

            if let Some(background_appearance) = background_appearance.take() {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: tab_bounds,
                        border: Border {
                            radius: theme.cosmic().radius_s().into(),
                            ..Default::default()
                        },
                        shadow: Shadow::default(),
                    },
                    background_appearance
                        .background
                        .map_or(Background::Color(Color::TRANSPARENT), bg_with_alpha),
                );
            }

            draw_icon::<Message>(
                renderer,
                theme,
                style,
                cursor,
                viewport,
                apply_alpha(if state.buttons_offset == 0 {
                    appearance.inactive.text_color
                } else if let Item::PrevButton = state.focused_item {
                    appearance.focus.text_color
                } else {
                    appearance.active.text_color
                }),
                Rectangle {
                    x: tab_bounds.x + 8.0,
                    y: tab_bounds.y + f32::from(self.button_height) / 4.0,
                    width: 16.0,
                    height: 16.0,
                },
                icon::from_name("go-previous-symbolic").size(16).icon(),
            );

            tab_bounds = next_tab_bounds(&bounds, f32::from(self.button_height));

            // Next tab button
            background_appearance =
                if self.on_activate.is_some() && Item::NextButton == state.focused_item {
                    Some(appearance.focus)
                } else if self.on_activate.is_some() && Item::NextButton == state.hovered {
                    Some(appearance.hover)
                } else {
                    None
                };

            if let Some(background_appearance) = background_appearance {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: tab_bounds,
                        border: Border {
                            radius: theme.cosmic().radius_s().into(),
                            ..Default::default()
                        },
                        shadow: Shadow::default(),
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
                apply_alpha(if self.next_tab_sensitive(state) {
                    appearance.active.text_color
                } else if let Item::NextButton = state.focused_item {
                    appearance.focus.text_color
                } else {
                    appearance.inactive.text_color
                }),
                Rectangle {
                    x: tab_bounds.x + 8.0,
                    y: tab_bounds.y + f32::from(self.button_height) / 4.0,
                    width: 16.0,
                    height: 16.0,
                },
                icon::from_name("go-next-symbolic").size(16).icon(),
            );
        }

        // Draw each of the items in the widget.
        let mut nth = 0;
        self.variant_bounds(state, bounds).for_each(move |item| {
            let (key, mut bounds) = match item {
                // Draw a button
                ItemBounds::Button(entity, bounds) => (entity, bounds),

                // Draw a divider between buttons
                ItemBounds::Divider(bounds) => {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds,
                            border: Border::default(),
                            shadow: Shadow::default(),
                        },
                        {
                            let theme = crate::theme::active();
                            Background::Color(theme.cosmic().small_widget_divider().into())
                        },
                    );

                    return;
                }
            };

            let center_y = bounds.center_y();

            let key_is_active = self.model.is_active(key);
            let key_is_hovered = self.button_is_hovered(state, key);
            let status_appearance = if self.button_is_focused(state, key) {
                appearance.focus
            } else if key_is_active {
                appearance.active
            } else if key_is_hovered {
                appearance.hover
            } else {
                appearance.inactive
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
                        border: Border {
                            radius: button_appearance.border_radius,
                            ..Default::default()
                        },
                        shadow: Shadow::default(),
                    },
                    status_appearance
                        .background
                        .map_or(Background::Color(Color::TRANSPARENT), bg_with_alpha),
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
                        border: Border {
                            radius: rad_0.into(),
                            ..Default::default()
                        },
                        shadow: Shadow::default(),
                    },
                    bg_with_alpha(background.into()),
                );
            }

            let original_bounds = bounds;

            bounds.x += f32::from(self.button_padding[0]);
            bounds.width -= f32::from(self.button_padding[0]) - f32::from(self.button_padding[2]);

            // Adjust bounds by indent
            if let Some(indent) = self.model.indent(key) {
                let adjustment = f32::from(indent) * f32::from(self.indent_spacing);
                bounds.x += adjustment;
                bounds.width -= adjustment;
            }

            // Align contents of the button to the requested `button_alignment`.
            {
                let actual_width = state.internal_layout[nth].1.width;

                let offset = match self.button_alignment {
                    Alignment::Start => None,
                    Alignment::Center => Some((bounds.width - actual_width) / 2.0),
                    Alignment::End => Some(bounds.width - actual_width),
                };

                if let Some(offset) = offset {
                    bounds.x += offset - f32::from(self.button_padding[0]);
                    bounds.width = actual_width;
                }
            }

            // Draw the image beside the text.
            if let Some(icon) = self.model.icon(key) {
                let mut image_bounds = bounds;
                let width = f32::from(icon.size);
                let offset = width + f32::from(self.button_spacing);
                image_bounds.y = center_y - width / 2.0;

                draw_icon::<Message>(
                    renderer,
                    theme,
                    style,
                    cursor,
                    viewport,
                    apply_alpha(status_appearance.text_color),
                    Rectangle {
                        width,
                        height: width,
                        ..image_bounds
                    },
                    icon.clone(),
                );

                bounds.x += offset;
            } else {
                // Draw the selection indicator if widget is a segmented selection, and the item is selected.
                if key_is_active {
                    if let crate::theme::SegmentedButton::Control = self.style {
                        let mut image_bounds = bounds;
                        image_bounds.y = center_y - 16.0 / 2.0;

                        draw_icon::<Message>(
                            renderer,
                            theme,
                            style,
                            cursor,
                            viewport,
                            apply_alpha(status_appearance.text_color),
                            Rectangle {
                                width: 16.0,
                                height: 16.0,
                                ..image_bounds
                            },
                            crate::widget::icon(
                                match crate::widget::common::object_select().data() {
                                    crate::iced_core::svg::Data::Bytes(bytes) => {
                                        crate::widget::icon::from_svg_bytes(bytes.as_ref())
                                    }
                                    crate::iced_core::svg::Data::Path(path) => {
                                        crate::widget::icon::from_path(path.clone())
                                    }
                                },
                            ),
                        );

                        let offset = 16.0 + f32::from(self.button_spacing);

                        bounds.x += offset;
                    }
                }
            }

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

            bounds.width = original_bounds.width
                - (bounds.x - original_bounds.x)
                - close_icon_width
                - f32::from(self.button_padding[2]);

            bounds.y = center_y;

            if self.model.text(key).is_some_and(|text| !text.is_empty()) {
                // Draw the text for this segmented button or tab.
                renderer.fill_paragraph(
                    &state.paragraphs[key],
                    bounds.position(),
                    apply_alpha(status_appearance.text_color),
                    Rectangle {
                        x: bounds.x,
                        width: bounds.width,
                        ..original_bounds
                    },
                );
            }

            // Draw a close button if set.
            if show_close_button {
                let close_button_bounds = close_bounds(original_bounds, close_icon_width);

                draw_icon::<Message>(
                    renderer,
                    theme,
                    style,
                    cursor,
                    viewport,
                    apply_alpha(status_appearance.text_color),
                    close_button_bounds,
                    self.close_icon.clone(),
                );
            }

            nth += 1;
        });
    }

    fn overlay<'b>(
        &'b mut self,
        _tree: &'b mut Tree,
        _layout: iced_core::Layout<'_>,
        _renderer: &Renderer,
    ) -> Option<iced_core::overlay::Element<'b, Message, crate::Theme, Renderer>> {
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
    pub(super) internal_layout: Vec<(Size, Size)>,
    /// The paragraphs for each text.
    paragraphs: SecondaryMap<Entity, crate::Paragraph>,
    /// Used to detect changes in text.
    text_hashes: SecondaryMap<Entity, u64>,
    /// Time since last tab activation from wheel movements.
    wheel_timestamp: Option<Instant>,
}

#[derive(Debug, Default, PartialEq)]
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
fn close_bounds(area: Rectangle<f32>, icon_size: f32) -> Rectangle<f32> {
    Rectangle {
        x: area.x + area.width - icon_size - 8.0,
        y: area.center_y() - (icon_size / 2.0),
        width: icon_size,
        height: icon_size,
    }
}

/// Calculate the bounds of the `next_tab` button.
fn next_tab_bounds(bounds: &Rectangle, button_height: f32) -> Rectangle {
    Rectangle {
        x: bounds.x + bounds.width - button_height,
        y: bounds.y,
        width: button_height,
        height: button_height,
    }
}

/// Calculate the bounds of the `prev_tab` button.
fn prev_tab_bounds(bounds: &Rectangle, button_height: f32) -> Rectangle {
    Rectangle {
        x: bounds.x,
        y: bounds.y,
        width: button_height,
        height: button_height,
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
    let layout_node = layout::Node::new(Size {
        width: bounds.width,
        height: bounds.width,
    })
    .move_to(Point {
        x: bounds.x,
        y: bounds.y,
    });

    Widget::<Message, crate::Theme, Renderer>::draw(
        Element::<Message>::from(icon.clone()).as_widget(),
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
