// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::model::{Entity, Model, Selectable};
use super::{InsertPosition, ReorderEvent};
use crate::iced_core::id::Internal;
use crate::theme::{SegmentedButton as Style, THEME};
use crate::widget::dnd_destination::DragId;
use crate::widget::menu::{
    self, CloseCondition, ItemHeight, ItemWidth, MenuBarState, PathHighlight, menu_roots_children,
    menu_roots_diff,
};
use crate::widget::{Icon, icon};
use crate::{Element, Renderer};
use derive_setters::Setters;
use iced::clipboard::dnd::{
    self, DndAction, DndDestinationRectangle, DndEvent, OfferEvent, SourceEvent,
};
use iced::clipboard::mime::AllowedMimeTypes;
use iced::touch::Finger;
use iced::{
    Alignment, Background, Color, Event, Length, Padding, Rectangle, Size, Task, Vector, alignment,
    event, keyboard, mouse, touch, window,
};
use iced_core::mouse::ScrollDelta;
use iced_core::text::{LineHeight, Renderer as TextRenderer, Shaping, Wrapping};
use iced_core::widget::operation::Focusable;
use iced_core::widget::{self, operation, tree};
use iced_core::{Border, Point, Renderer as IcedRenderer, Shadow, Text};
use iced_core::{Clipboard, Layout, Shell, Widget, layout, renderer, widget::Tree};
use iced_runtime::{Action, task};
use slotmap::{Key, SecondaryMap};
use std::borrow::Cow;
use std::cell::{Cell, LazyCell};
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem;
use std::time::{Duration, Instant};

thread_local! {
    // Prevents two segmented buttons from being focused at the same time.
    static LAST_FOCUS_UPDATE: LazyCell<Cell<Instant>> = LazyCell::new(|| Cell::new(Instant::now()));
}

const TAB_REORDER_LOG_TARGET: &str = "libcosmic::widget::tab_reorder";

/// A command that focuses a segmented item stored in a widget.
pub fn focus<Message: 'static>(id: Id) -> Task<Message> {
    task::effect(Action::Widget(Box::new(operation::focusable::focus(id.0))))
}

pub enum ItemBounds {
    Button(Entity, Rectangle),
    Divider(Rectangle, bool),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DropSide {
    Before,
    After,
}

impl From<DropSide> for InsertPosition {
    fn from(side: DropSide) -> Self {
        match side {
            DropSide::Before => InsertPosition::Before,
            DropSide::After => InsertPosition::After,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DropHint {
    entity: Entity,
    side: DropSide,
}

/// Isolates variant-specific behaviors from [`SegmentedButton`].
pub trait SegmentedVariant {
    const VERTICAL: bool;

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
    pub(super) id: Id,
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
    pub(super) font_active: crate::font::Font,
    /// Desired font for hovered tabs.
    pub(super) font_hovered: crate::font::Font,
    /// Desired font for inactive tabs.
    pub(super) font_inactive: crate::font::Font,
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
    /// The context menu to display when a context is activated
    #[setters(skip)]
    pub(super) context_menu: Option<Vec<menu::Tree<Message>>>,
    /// Emits the ID of the item that was activated.
    #[setters(skip)]
    pub(super) on_activate: Option<Box<dyn Fn(Entity) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) on_close: Option<Box<dyn Fn(Entity) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) on_context: Option<Box<dyn Fn(Entity) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) on_middle_press: Option<Box<dyn Fn(Entity) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) on_dnd_drop:
        Option<Box<dyn Fn(Entity, Vec<u8>, String, DndAction) -> Message + 'static>>,
    pub(super) mimes: Vec<String>,
    #[setters(skip)]
    pub(super) on_dnd_enter: Option<Box<dyn Fn(Entity, Vec<String>) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) on_dnd_leave: Option<Box<dyn Fn(Entity) -> Message + 'static>>,
    #[setters(strip_option)]
    pub(super) drag_id: Option<DragId>,
    #[setters(skip)]
    pub(super) tab_drag: Option<TabDragSource<Message>>,
    #[setters(skip)]
    pub(super) on_drop_hint: Option<Box<dyn Fn(Option<(Entity, bool)>) -> Message + 'static>>,
    #[setters(skip)]
    pub(super) on_reorder: Option<Box<dyn Fn(ReorderEvent) -> Message + 'static>>,
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
    #[inline]
    pub fn new(model: &'a Model<SelectionMode>) -> Self {
        Self {
            model,
            id: Id::unique(),
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
            font_active: crate::font::semibold(),
            font_hovered: crate::font::semibold(),
            font_inactive: crate::font::default(),
            font_size: 14.0,
            height: Length::Shrink,
            width: Length::Fill,
            spacing: 0,
            line_height: LineHeight::default(),
            style: Style::default(),
            context_menu: None,
            on_activate: None,
            on_close: None,
            on_context: None,
            on_middle_press: None,
            on_dnd_drop: None,
            on_dnd_enter: None,
            on_dnd_leave: None,
            mimes: Vec::new(),
            variant: PhantomData,
            drag_id: None,
            tab_drag: None,
            on_drop_hint: None,
            on_reorder: None,
        }
    }

    pub fn context_menu(mut self, context_menu: Option<Vec<menu::Tree<Message>>>) -> Self
    where
        Message: Clone + 'static,
    {
        self.context_menu = context_menu.map(|menus| {
            vec![menu::Tree::with_children(
                crate::Element::from(crate::widget::row::<'static, Message>()),
                menus,
            )]
        });

        if let Some(ref mut context_menu) = self.context_menu {
            context_menu.iter_mut().for_each(menu::Tree::set_index);
        }

        self
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

    /// Emitted when a button is right-clicked.
    pub fn on_context<T>(mut self, on_context: T) -> Self
    where
        T: Fn(Entity) -> Message + 'static,
    {
        self.on_context = Some(Box::new(on_context));
        self
    }

    /// Emitted when the middle mouse button is pressed on a button.
    pub fn on_middle_press<T>(mut self, on_middle_press: T) -> Self
    where
        T: Fn(Entity) -> Message + 'static,
    {
        self.on_middle_press = Some(Box::new(on_middle_press));
        self
    }

    /// Enable drag-and-drop support for tabs using the provided payload builder.
    pub fn enable_tab_drag(
        mut self,
        payload: impl Fn(Entity) -> Option<(String, Vec<u8>)> + 'static,
    ) -> Self {
        self.tab_drag = Some(TabDragSource::new(payload));
        self
    }

    /// Receive drop hint updates during drag-and-drop.
    pub fn on_drop_hint(
        mut self,
        callback: impl Fn(Option<(Entity, bool)>) -> Message + 'static,
    ) -> Self {
        self.on_drop_hint = Some(Box::new(callback));
        self
    }

    /// Emit a message when a tab drag is dropped inside this widget.
    pub fn on_reorder(mut self, callback: impl Fn(ReorderEvent) -> Message + 'static) -> Self {
        self.on_reorder = Some(Box::new(callback));
        self
    }

    /// Set the pointer distance threshold before a drag is started.
    pub fn tab_drag_threshold(mut self, threshold: f32) -> Self {
        if let Some(tab_drag) = self.tab_drag.as_mut() {
            tab_drag.threshold = threshold.max(1.0);
        }
        self
    }

    fn reorder_event_for_drop(&self, state: &LocalState, target: Entity) -> Option<ReorderEvent> {
        let dragged = state.dragging_tab?;
        if dragged == target
            || !self.model.contains_item(dragged)
            || !self.model.contains_item(target)
        {
            return None;
        }
        let position = state
            .drop_hint
            .filter(|hint| hint.entity == target)
            .map(|hint| InsertPosition::from(hint.side))
            .unwrap_or_else(|| self.default_insert_position(dragged, target));
        Some(ReorderEvent {
            dragged,
            target,
            position,
        })
    }

    fn default_insert_position(&self, dragged: Entity, target: Entity) -> InsertPosition {
        let len = self.model.len();
        let target_pos = self
            .model
            .position(target)
            .map(|pos| pos as usize)
            .unwrap_or(len);
        let from_pos = self
            .model
            .position(dragged)
            .map(|pos| pos as usize)
            .unwrap_or(target_pos);
        if from_pos < target_pos {
            InsertPosition::After
        } else {
            InsertPosition::Before
        }
    }

    /// Check if an item is enabled.
    fn is_enabled(&self, key: Entity) -> bool {
        self.model.items.get(key).is_some_and(|item| item.enabled)
    }

    /// Handle the dnd drop event.
    pub fn on_dnd_drop<D: AllowedMimeTypes>(
        mut self,
        dnd_drop_handler: impl Fn(Entity, Option<D>, DndAction) -> Message + 'static,
    ) -> Self {
        self.on_dnd_drop = Some(Box::new(move |entity, data, mime, action| {
            dnd_drop_handler(entity, D::try_from((data, mime)).ok(), action)
        }));
        self.mimes = D::allowed().into_owned();
        self
    }

    /// Handle the dnd enter event.
    pub fn on_dnd_enter(
        mut self,
        dnd_enter_handler: impl Fn(Entity, Vec<String>) -> Message + 'static,
    ) -> Self {
        self.on_dnd_enter = Some(Box::new(dnd_enter_handler));
        self
    }

    /// Handle the dnd leave event.
    pub fn on_dnd_leave(mut self, dnd_leave_handler: impl Fn(Entity) -> Message + 'static) -> Self {
        self.on_dnd_leave = Some(Box::new(dnd_leave_handler));
        self
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
                    crate::Plain::new(Text {
                        content: text.as_ref(),
                        size: iced::Pixels(self.font_size),
                        bounds: Size::INFINITY,
                        font,
                        horizontal_alignment: alignment::Horizontal::Left,
                        vertical_alignment: alignment::Vertical::Center,
                        shaping: Shaping::Advanced,
                        wrapping: Wrapping::default(),
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
        state.focused.is_some()
            && self.on_activate.is_some()
            && Item::Tab(key) == state.focused_item
    }

    fn button_is_hovered(&self, state: &LocalState, key: Entity) -> bool {
        self.on_activate.is_some() && state.hovered == Item::Tab(key)
            || state
                .dnd_state
                .drag_offer
                .as_ref()
                .is_some_and(|id| id.data.is_some_and(|d| d == key))
    }

    fn button_is_pressed(&self, state: &LocalState, key: Entity) -> bool {
        state.pressed_item == Some(Item::Tab(key))
    }

    fn emit_drop_hint(&self, shell: &mut Shell<'_, Message>, hint: Option<DropHint>) {
        if let Some(on_hint) = self.on_drop_hint.as_ref() {
            let mapped = hint.map(|hint| (hint.entity, matches!(hint.side, DropSide::After)));
            shell.publish(on_hint(mapped));
        }
    }

    fn drop_hint_for_position(
        &self,
        state: &LocalState,
        bounds: Rectangle,
        cursor: Point,
    ) -> Option<DropHint> {
        let dragging = state.dragging_tab?;

        self.variant_bounds(state, bounds)
            .filter_map(|item| match item {
                ItemBounds::Button(entity, rect) if rect.contains(cursor) => Some((entity, rect)),
                _ => None,
            })
            .find_map(|(entity, rect)| {
                let before = if Self::VERTICAL {
                    cursor.y < rect.center_y()
                } else {
                    cursor.x < rect.center_x()
                };
                Some(DropHint {
                    entity,
                    side: if before {
                        DropSide::Before
                    } else {
                        DropSide::After
                    },
                })
            })
    }

    fn start_tab_drag(
        &self,
        state: &mut LocalState,
        entity: Entity,
        bounds: Rectangle,
        cursor: Point,
        clipboard: &mut dyn Clipboard,
    ) -> bool {
        let Some(tab_drag) = self.tab_drag.as_ref() else {
            return false;
        };

        log::trace!(
            target: TAB_REORDER_LOG_TARGET,
            "start_tab_drag requested entity={:?} cursor=({:.2},{:.2}) bounds=({:.2},{:.2},{:.2},{:.2}) threshold={}",
            entity,
            cursor.x,
            cursor.y,
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            tab_drag.threshold
        );

        let Some((mime, data)) = (tab_drag.payload)(entity) else {
            log::trace!(
                target: TAB_REORDER_LOG_TARGET,
                "start_tab_drag aborted entity={:?}: payload builder returned None",
                entity
            );
            return false;
        };

        let data_len = data.len();
        let mime_label = mime.clone();

        iced_core::clipboard::start_dnd::<crate::Theme, crate::Renderer>(
            clipboard,
            false,
            Some(iced_core::clipboard::DndSource::Widget(self.id.0.clone())),
            None,
            Box::new(SimpleDragData::new(mime, data)),
            DndAction::Move,
        );
        log::trace!(
            target: TAB_REORDER_LOG_TARGET,
            "tab drag started entity={:?} mime={} bytes={}",
            entity,
            mime_label,
            data_len
        );
        state.dragging_tab = Some(entity);
        state.tab_drag_candidate = None;
        state.pressed_item = None;
        true
    }

    /// Returns the drag id of the destination.
    ///
    /// # Panics
    /// Panics if the destination has been assigned a Set id, which is invalid.
    #[must_use]
    pub fn get_drag_id(&self) -> u128 {
        self.drag_id.map_or_else(
            || {
                u128::from(match &self.id.0.0 {
                    Internal::Unique(id) | Internal::Custom(id, _) => *id,
                    Internal::Set(_) => panic!("Invalid Id assigned to dnd destination."),
                })
            },
            |id| id.0,
        )
    }
}

impl<Variant, SelectionMode, Message> Widget<Message, crate::Theme, Renderer>
    for SegmentedButton<'_, Variant, SelectionMode, Message>
where
    Self: SegmentedVariant,
    Model<SelectionMode>: Selectable,
    SelectionMode: Default,
    Message: 'static + Clone,
{
    fn children(&self) -> Vec<Tree> {
        let mut children = Vec::new();

        // Assign the context menu's elements as this widget's children.
        if let Some(ref context_menu) = self.context_menu {
            let mut tree = Tree::empty();
            tree.state = tree::State::new(MenuBarState::default());
            tree.children = menu_roots_children(context_menu);
            children.push(tree);
        }

        children
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<LocalState>()
    }

    fn state(&self) -> tree::State {
        #[allow(clippy::default_trait_access)]
        tree::State::new(LocalState {
            menu_state: Default::default(),
            paragraphs: SecondaryMap::new(),
            text_hashes: SecondaryMap::new(),
            buttons_visible: Default::default(),
            buttons_offset: Default::default(),
            collapsed: Default::default(),
            focused: Default::default(),
            focused_item: Default::default(),
            focused_visible: false,
            hovered: Default::default(),
            known_length: Default::default(),
            middle_clicked: Default::default(),
            internal_layout: Default::default(),
            context_cursor: Point::default(),
            show_context: Default::default(),
            wheel_timestamp: Default::default(),
            dnd_state: Default::default(),
            fingers_pressed: Default::default(),
            pressed_item: None,
            tab_drag_candidate: None,
            dragging_tab: None,
            drop_hint: None,
        })
    }

    fn diff(&mut self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<LocalState>();

        for key in self.model.order.iter().copied() {
            if let Some(text) = self.model.text.get(key) {
                let font = if self.button_is_focused(state, key) {
                    self.font_active
                } else if state.show_context.is_some() || self.button_is_hovered(state, key) {
                    self.font_hovered
                } else if self.model.is_active(key) {
                    self.font_active
                } else {
                    self.font_inactive
                };

                let mut hasher = DefaultHasher::new();
                text.hash(&mut hasher);
                font.hash(&mut hasher);
                let text_hash = hasher.finish();

                if let Some(prev_hash) = state.text_hashes.insert(key, text_hash) {
                    if prev_hash == text_hash {
                        continue;
                    }
                }

                let text = Text {
                    content: text.as_ref(),
                    size: iced::Pixels(self.font_size),
                    bounds: Size::INFINITY,
                    font,
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Center,
                    shaping: Shaping::Advanced,
                    wrapping: Wrapping::None,
                    line_height: self.line_height,
                };

                if let Some(paragraph) = state.paragraphs.get_mut(key) {
                    paragraph.update(text);
                } else {
                    state.paragraphs.insert(key, crate::Plain::new(text));
                }
            }
        }

        // Diff the context menu
        if let Some(context_menu) = &mut self.context_menu {
            state.menu_state.inner.with_data_mut(|inner| {
                menu_roots_diff(context_menu, &mut inner.tree);
            });
        }

        // Unfocus if another segmented control was focused.
        if let Some(f) = state.focused.as_ref() {
            if f.updated_at != LAST_FOCUS_UPDATE.with(|f| f.get()) {
                state.unfocus();
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
        mut event: Event,
        layout: Layout<'_>,
        cursor_position: mouse::Cursor,
        _renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &iced::Rectangle,
    ) -> event::Status {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<LocalState>();
        state.hovered = Item::None;

        let my_id = self.get_drag_id();

        if let Event::Dnd(e) = &mut event {
            let entity = state
                .dnd_state
                .drag_offer
                .as_ref()
                .map(|dnd_state| dnd_state.data);
            log::trace!(
                target: TAB_REORDER_LOG_TARGET,
                "segmented button {:?} received DnD event: {:?} entity={entity:?}",
                my_id,
                e
            );
            match e {
                DndEvent::Source(SourceEvent::Cancelled | SourceEvent::Finished) => {
                    if state.dragging_tab.take().is_some() {
                        state.tab_drag_candidate = None;
                        state.drop_hint = None;
                        self.emit_drop_hint(shell, state.drop_hint);
                        log::trace!(
                            target: TAB_REORDER_LOG_TARGET,
                            "tab drag source finished id={:?}",
                            my_id
                        );
                        return event::Status::Captured;
                    }
                }
                DndEvent::Offer(
                    id,
                    OfferEvent::Enter {
                        x, y, mime_types, ..
                    },
                ) if Some(my_id) == *id => {
                    let entity = self
                        .variant_bounds(state, bounds)
                        .filter_map(|item| match item {
                            ItemBounds::Button(entity, bounds) => Some((entity, bounds)),
                            _ => None,
                        })
                        .find(|(_key, bounds)| bounds.contains(Point::new(*x as f32, *y as f32)))
                        .map(|(key, _)| key);
                    state.drop_hint = self.drop_hint_for_position(
                        state,
                        bounds,
                        Point::new(*x as f32, *y as f32),
                    );
                    self.emit_drop_hint(shell, state.drop_hint);
                    log::trace!(
                        target: TAB_REORDER_LOG_TARGET,
                        "offer enter id={my_id:?} entity={entity:?} @ ({x},{y}) mimes={mime_types:?}"
                    );

                    let on_dnd_enter =
                        self.on_dnd_enter
                            .as_ref()
                            .zip(entity)
                            .map(|(on_enter, entity)| {
                                move |_, _, mime_types| on_enter(entity, mime_types)
                            });

                    _ = state.dnd_state.on_enter::<Message>(
                        *x,
                        *y,
                        mime_types.clone(),
                        on_dnd_enter,
                        entity,
                    );
                }
                DndEvent::Offer(id, OfferEvent::LeaveDestination) if Some(my_id) != *id => {}
                DndEvent::Offer(id, OfferEvent::Leave | OfferEvent::LeaveDestination)
                    if Some(my_id) == *id =>
                {
                    state.drop_hint = None;
                    self.emit_drop_hint(shell, state.drop_hint);
                    if let Some(Some(entity)) = entity {
                        if let Some(on_dnd_leave) = self.on_dnd_leave.as_ref() {
                            shell.publish(on_dnd_leave(entity));
                        }
                    }
                    log::trace!(
                        target: TAB_REORDER_LOG_TARGET,
                        "offer leave id={my_id:?} entity={entity:?}"
                    );
                    _ = state.dnd_state.on_leave::<Message>(None);
                }
                DndEvent::Offer(_, OfferEvent::Leave | OfferEvent::LeaveDestination) => {}
                DndEvent::Offer(id, OfferEvent::Motion { x, y }) if Some(my_id) == *id => {
                    log::trace!(
                        target: TAB_REORDER_LOG_TARGET,
                        "offer motion id={my_id:?} cursor=({x},{y}) current_entity={entity:?}"
                    );
                    let new = self
                        .variant_bounds(state, bounds)
                        .filter_map(|item| match item {
                            ItemBounds::Button(entity, bounds) => Some((entity, bounds)),
                            _ => None,
                        })
                        .find(|(_key, bounds)| bounds.contains(Point::new(*x as f32, *y as f32)))
                        .map(|(key, _)| key);
                    if let Some(new_entity) = new {
                        state.dnd_state.on_motion::<Message>(
                            *x,
                            *y,
                            None::<fn(_, _) -> Message>,
                            None::<fn(_, _, _) -> Message>,
                            Some(new_entity),
                        );
                        state.drop_hint = self.drop_hint_for_position(
                            state,
                            bounds,
                            Point::new(*x as f32, *y as f32),
                        );
                        self.emit_drop_hint(shell, state.drop_hint);
                        if Some(Some(new_entity)) != entity {
                            let prev_action = state
                                .dnd_state
                                .drag_offer
                                .as_ref()
                                .map(|dnd| dnd.selected_action);
                            if let Some(on_dnd_enter) = self.on_dnd_enter.as_ref() {
                                shell.publish(on_dnd_enter(new_entity, Vec::new()));
                            }
                            if let Some(dnd) = state.dnd_state.drag_offer.as_mut() {
                                dnd.data = Some(new_entity);
                                if let Some(prev_action) = prev_action {
                                    dnd.selected_action = prev_action;
                                }
                            }
                        }
                    } else if entity.is_some() {
                        log::trace!(
                            target: TAB_REORDER_LOG_TARGET,
                            "offer motion leaving id={my_id:?}"
                        );
                        state.drop_hint = None;
                        self.emit_drop_hint(shell, state.drop_hint);
                        state.dnd_state.on_motion::<Message>(
                            *x,
                            *y,
                            None::<fn(_, _) -> Message>,
                            None::<fn(_, _, _) -> Message>,
                            None,
                        );
                        if let Some(on_dnd_leave) = self.on_dnd_leave.as_ref() {
                            if let Some(Some(entity)) = entity {
                                shell.publish(on_dnd_leave(entity));
                            }
                        }
                    }
                }
                DndEvent::Offer(id, OfferEvent::Drop) if Some(my_id) == *id => {
                    log::trace!(
                        target: TAB_REORDER_LOG_TARGET,
                        "offer drop id={my_id:?} entity={entity:?}"
                    );
                    _ = state
                        .dnd_state
                        .on_drop::<Message>(None::<fn(_, _) -> Message>);
                }
                DndEvent::Offer(id, OfferEvent::SelectedAction(action)) if Some(my_id) == *id => {
                    if state.dnd_state.drag_offer.is_some() {
                        log::trace!(
                            target: TAB_REORDER_LOG_TARGET,
                            "offer selected action id={my_id:?} action={action:?} entity={entity:?}"
                        );
                        _ = state
                            .dnd_state
                            .on_action_selected::<Message>(*action, None::<fn(_) -> Message>);
                    }
                }
                DndEvent::Offer(id, OfferEvent::Data { data, mime_type }) if Some(my_id) == *id => {
                    log::trace!(
                        target: TAB_REORDER_LOG_TARGET,
                        "offer data id={my_id:?} entity={entity:?} mime={mime_type:?}"
                    );
                    let drop_entity = entity
                        .flatten()
                        .or_else(|| state.drop_hint.map(|hint| hint.entity));
                    let allow_reorder = state
                        .dnd_state
                        .drag_offer
                        .as_ref()
                        .is_some_and(|offer| offer.selected_action.contains(DndAction::Move));
                    let pending_reorder = if allow_reorder && self.on_reorder.is_some() {
                        drop_entity.and_then(|target| self.reorder_event_for_drop(state, target))
                    } else {
                        None
                    };
                    if let Some(entity) = drop_entity {
                        let on_drop = self.on_dnd_drop.as_ref();
                        let on_drop = on_drop.map(|on_drop| {
                            |mime, data, action, _, _| on_drop(entity, data, mime, action)
                        });

                        let (maybe_msg, ret) = state.dnd_state.on_data_received(
                            mem::take(mime_type),
                            mem::take(data),
                            None::<fn(_, _) -> Message>,
                            on_drop,
                        );
                        if let Some(msg) = maybe_msg {
                            log::trace!(
                                target: TAB_REORDER_LOG_TARGET,
                                "publishing drop message entity={entity:?}"
                            );
                            shell.publish(msg);
                        }
                        state.drop_hint = None;
                        self.emit_drop_hint(shell, state.drop_hint);
                        if let Some(event) = pending_reorder {
                            if let Some(on_reorder) = self.on_reorder.as_ref() {
                                shell.publish(on_reorder(event));
                            }
                        }
                        return ret;
                    } else {
                        log::trace!(
                            target: TAB_REORDER_LOG_TARGET,
                            "data received without entity id={my_id:?}"
                        );
                        state.drop_hint = None;
                        self.emit_drop_hint(shell, state.drop_hint);
                        if let Some(event) = pending_reorder {
                            if let Some(on_reorder) = self.on_reorder.as_ref() {
                                shell.publish(on_reorder(event));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if cursor_position.is_over(bounds) {
            let fingers_pressed = state.fingers_pressed.len();

            match event {
                Event::Touch(touch::Event::FingerPressed { id, .. }) => {
                    state.fingers_pressed.insert(id);
                }

                Event::Touch(touch::Event::FingerLifted { id, .. }) => {
                    state.fingers_pressed.remove(&id);
                }

                _ => (),
            }

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

                        let close_button_bounds =
                            close_bounds(bounds, f32::from(self.close_icon.size));
                        let over_close_button = self.model.items[key].closable
                            && cursor_position.is_over(close_button_bounds);

                        // If marked as closable, show a close icon.
                        if self.model.items[key].closable {
                            // Emit close message if the close button is pressed.
                            if let Some(on_close) = self.on_close.as_ref() {
                                if over_close_button
                                    && (left_button_released(&event)
                                        || (touch_lifted(&event) && fingers_pressed == 1))
                                {
                                    shell.publish(on_close(key));
                                    return event::Status::Captured;
                                }

                                if self.on_middle_press.is_none() {
                                    // Emit close message if the tab is middle clicked.
                                    if let Event::Mouse(mouse::Event::ButtonReleased(
                                        mouse::Button::Middle,
                                    )) = event
                                    {
                                        if state.middle_clicked == Some(Item::Tab(key)) {
                                            shell.publish(on_close(key));
                                            return event::Status::Captured;
                                        }

                                        state.middle_clicked = None;
                                    }
                                }
                            }
                        }

                        if self.tab_drag.is_some()
                            && matches!(
                                event,
                                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                            )
                            && !over_close_button
                        {
                            if let Some(position) = cursor_position.position() {
                                state.tab_drag_candidate = Some(TabDragCandidate {
                                    entity: key,
                                    bounds,
                                    origin: position,
                                });
                                if let Some(tab_drag) = self.tab_drag.as_ref() {
                                    log::trace!(
                                        target: TAB_REORDER_LOG_TARGET,
                                        "tab drag candidate entity={:?} origin=({:.2},{:.2}) bounds=({:.2},{:.2},{:.2},{:.2}) threshold={}",
                                        key,
                                        position.x,
                                        position.y,
                                        bounds.x,
                                        bounds.y,
                                        bounds.width,
                                        bounds.height,
                                        tab_drag.threshold
                                    );
                                }
                            }
                        }

                        if is_lifted(&event) {
                            state.unfocus();
                        }

                        if let Some(on_activate) = self.on_activate.as_ref() {
                            if is_pressed(&event) {
                                state.pressed_item = Some(Item::Tab(key));
                            } else if is_lifted(&event) {
                                if self.button_is_pressed(state, key) {
                                    shell.publish(on_activate(key));
                                    state.set_focused();
                                    state.focused_item = Item::Tab(key);
                                    state.pressed_item = None;
                                    return event::Status::Captured;
                                }
                            }
                        }

                        // Present a context menu on a right click event.
                        if self.context_menu.is_some() {
                            if let Some(on_context) = self.on_context.as_ref() {
                                if right_button_released(&event)
                                    || (touch_lifted(&event) && fingers_pressed == 2)
                                {
                                    state.show_context = Some(key);
                                    state.context_cursor =
                                        cursor_position.position().unwrap_or_default();

                                    state.menu_state.inner.with_data_mut(|data| {
                                        data.open = true;
                                        data.view_cursor = cursor_position;
                                    });

                                    shell.publish(on_context(key));
                                    return event::Status::Captured;
                                }
                            }
                        }

                        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Middle)) =
                            event
                        {
                            state.middle_clicked = Some(Item::Tab(key));
                            if let Some(on_middle_press) = self.on_middle_press.as_ref() {
                                shell.publish(on_middle_press(key));
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
                        if state.wheel_timestamp.is_none_or(|previous| {
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
                                        state.set_focused();
                                        state.focused_item = Item::Tab(key);
                                        return event::Status::Captured;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if state.is_focused() {
            // Unfocus on clicks outside of the boundaries of the segmented button.
            if is_pressed(&event) {
                state.unfocus();
                state.pressed_item = None;
                return event::Status::Ignored;
            }
        } else if is_lifted(&event) {
            state.pressed_item = None;
        }

        if let (Some(tab_drag), Some(candidate)) =
            (self.tab_drag.as_ref(), state.tab_drag_candidate)
        {
            if let Event::Mouse(mouse::Event::CursorMoved { .. }) = event {
                if let Some(position) = cursor_position.position() {
                    if position.distance(candidate.origin) >= tab_drag.threshold {
                        if let Some(candidate) = state.tab_drag_candidate.take() {
                            log::trace!(
                                target: TAB_REORDER_LOG_TARGET,
                                "tab drag threshold met entity={:?} distance={:.2} threshold={}",
                                candidate.entity,
                                position.distance(candidate.origin),
                                tab_drag.threshold
                            );
                            if self.start_tab_drag(
                                state,
                                candidate.entity,
                                candidate.bounds,
                                position,
                                clipboard,
                            ) {
                                return event::Status::Captured;
                            }
                        }
                    }
                }
            }
        }

        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        ) {
            state.tab_drag_candidate = None;
        }

        if state.is_focused() {
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Tab),
                modifiers,
                ..
            }) = event
            {
                state.focused_visible = true;
                return if modifiers == keyboard::Modifiers::SHIFT {
                    self.focus_previous(state)
                } else if modifiers.is_empty() {
                    self.focus_next(state)
                } else {
                    event::Status::Ignored
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
        operation: &mut dyn iced_core::widget::Operation<()>,
    ) {
        let state = tree.state.downcast_mut::<LocalState>();
        operation.focusable(state, Some(&self.id.0));
        operation.custom(state, Some(&self.id.0));

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
        let show_drop_hint = state.dragging_tab.is_some();
        let drop_hint = if show_drop_hint {
            state.drop_hint
        } else {
            None
        };

        // Draw the background, if a background was defined.
        if let Some(background) = appearance.background {
            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: appearance.border,
                    shadow: Shadow::default(),
                },
                background,
            );
        }

        // Draw previous and next tab buttons if there is a need to paginate tabs.
        if state.collapsed {
            let mut tab_bounds = prev_tab_bounds(&bounds, f32::from(self.button_height));

            // Previous tab button
            let mut background_appearance =
                if self.on_activate.is_some() && Item::PrevButton == state.focused_item {
                    Some(appearance.active)
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
                } else {
                    appearance.active.text_color
                },
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
                    Some(appearance.active)
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
                if self.next_tab_sensitive(state) {
                    appearance.active.text_color
                } else if let Item::NextButton = state.focused_item {
                    appearance.active.text_color
                } else {
                    appearance.inactive.text_color
                },
                Rectangle {
                    x: tab_bounds.x + 8.0,
                    y: tab_bounds.y + f32::from(self.button_height) / 4.0,
                    width: 16.0,
                    height: 16.0,
                },
                icon::from_name("go-next-symbolic").size(16).icon(),
            );
        }

        let rad_0 = THEME.lock().unwrap().cosmic().corner_radii.radius_0;

        let divider_background = Background::Color(
            crate::theme::active()
                .cosmic()
                .primary_component_divider()
                .into(),
        );

        // Draw each of the items in the widget.
        let mut nth = 0;
        let drop_hint_marker = drop_hint;
        let show_drop_hint_marker = show_drop_hint;
        self.variant_bounds(state, bounds).for_each(move |item| {
            let (key, mut bounds) = match item {
                // Draw a button
                ItemBounds::Button(entity, bounds) => (entity, bounds),

                // Draw a divider between buttons
                ItemBounds::Divider(bounds, accented) => {
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds,
                            border: Border::default(),
                            shadow: Shadow::default(),
                        },
                        {
                            let theme = crate::theme::active();
                            if accented {
                                Background::Color(theme.cosmic().small_widget_divider().into())
                            } else {
                                Background::Color(theme.cosmic().primary_container_divider().into())
                            }
                        },
                    );

                    return;
                }
            };

            let original_bounds = bounds;
            let center_y = bounds.center_y();

            if show_drop_hint_marker {
                if matches!(
                    drop_hint_marker,
                    Some(DropHint {
                        entity,
                        side: DropSide::Before
                    }) if entity == key
                ) {
                    draw_drop_indicator(
                        renderer,
                        original_bounds,
                        DropSide::Before,
                        Self::VERTICAL,
                        appearance.active.text_color,
                    );
                }
            }

            let menu_open = || {
                state.show_context == Some(key)
                    && !tree.children.is_empty()
                    && tree.children[0]
                        .state
                        .downcast_ref::<MenuBarState>()
                        .inner
                        .with_data(|data| data.open)
            };

            let key_is_active = self.model.is_active(key);
            let key_is_focused = state.focused_visible && self.button_is_focused(state, key);
            let key_is_hovered = self.button_is_hovered(state, key);
            let status_appearance = if self.button_is_pressed(state, key) {
                appearance.pressed
            } else if key_is_hovered || menu_open() {
                appearance.hover
            } else if key_is_active {
                appearance.active
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

            // Draw the active hint on tabs
            if appearance.active_width > 0.0 {
                let active_width = if key_is_active {
                    appearance.active_width
                } else {
                    1.0
                };

                renderer.fill_quad(
                    renderer::Quad {
                        bounds: if Self::VERTICAL {
                            Rectangle {
                                x: bounds.x + bounds.width - active_width,
                                width: active_width,
                                ..bounds
                            }
                        } else {
                            Rectangle {
                                y: bounds.y + bounds.height - active_width,
                                height: active_width,
                                ..bounds
                            }
                        },
                        border: Border {
                            radius: rad_0.into(),
                            ..Default::default()
                        },
                        shadow: Shadow::default(),
                    },
                    appearance.active.text_color,
                );
            }

            bounds.x += f32::from(self.button_padding[0]);
            bounds.width -= f32::from(self.button_padding[0]) - f32::from(self.button_padding[2]);
            let mut indent_padding = 0.0;

            // Adjust bounds by indent
            if let Some(indent) = self.model.indent(key) {
                if indent > 0 {
                    let adjustment = f32::from(indent) * f32::from(self.indent_spacing);
                    bounds.x += adjustment;
                    bounds.width -= adjustment;

                    // Draw indent line
                    if let crate::theme::SegmentedButton::FileNav = self.style {
                        if indent > 1 {
                            indent_padding = 7.0;

                            for level in 1..indent {
                                renderer.fill_quad(
                                    renderer::Quad {
                                        bounds: Rectangle {
                                            x: (level as f32)
                                                .mul_add(-(self.indent_spacing as f32), bounds.x)
                                                + indent_padding,
                                            width: 1.0,
                                            ..bounds
                                        },
                                        border: Border {
                                            radius: rad_0.into(),
                                            ..Default::default()
                                        },
                                        shadow: Shadow::default(),
                                    },
                                    divider_background,
                                );
                            }

                            indent_padding += 4.0;
                        }
                    }
                }
            }

            // Render the background of the button.
            if key_is_focused || status_appearance.background.is_some() {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x - f32::from(self.button_padding[0]) + indent_padding,
                            width: bounds.width + f32::from(self.button_padding[0])
                                - f32::from(self.button_padding[2])
                                - indent_padding,
                            ..bounds
                        },
                        border: if key_is_focused {
                            Border {
                                width: 1.0,
                                color: appearance.active.text_color,
                                radius: button_appearance.border.radius,
                            }
                        } else {
                            button_appearance.border
                        },
                        shadow: Shadow::default(),
                    },
                    status_appearance
                        .background
                        .unwrap_or(Background::Color(Color::TRANSPARENT)),
                );
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
                    status_appearance.text_color,
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
                        image_bounds.y = center_y - 8.0;

                        draw_icon::<Message>(
                            renderer,
                            theme,
                            style,
                            cursor,
                            viewport,
                            status_appearance.text_color,
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
                    state.paragraphs[key].raw(),
                    bounds.position(),
                    status_appearance.text_color,
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
                    status_appearance.text_color,
                    close_button_bounds,
                    self.close_icon.clone(),
                );
            }

            if show_drop_hint_marker {
                if matches!(
                    drop_hint_marker,
                    Some(DropHint {
                        entity,
                        side: DropSide::After
                    }) if entity == key
                ) {
                    draw_drop_indicator(
                        renderer,
                        original_bounds,
                        DropSide::After,
                        Self::VERTICAL,
                        appearance.active.text_color,
                    );
                }
            }

            nth += 1;
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: iced_core::Layout<'_>,
        _renderer: &Renderer,
        translation: Vector,
    ) -> Option<iced_core::overlay::Element<'b, Message, crate::Theme, Renderer>> {
        let state = tree.state.downcast_ref::<LocalState>();
        let menu_state = state.menu_state.clone();

        let entity = state.show_context?;

        let mut bounds =
            self.variant_bounds(state, layout.bounds())
                .find_map(|item| match item {
                    ItemBounds::Button(e, bounds) if e == entity => Some(bounds),
                    _ => None,
                })?;

        let context_menu = self.context_menu.as_mut()?;

        if !menu_state.inner.with_data(|data| data.open) {
            // If the menu is not open, we don't need to show it.
            return None;
        }
        bounds.x = state.context_cursor.x;
        bounds.y = state.context_cursor.y;

        Some(
            crate::widget::menu::Menu {
                tree: menu_state,
                menu_roots: std::borrow::Cow::Owned(context_menu.clone()),
                bounds_expand: 16,
                menu_overlays_parent: true,
                close_condition: CloseCondition {
                    leave: false,
                    click_outside: true,
                    click_inside: true,
                },
                item_width: ItemWidth::Uniform(240),
                item_height: ItemHeight::Dynamic(40),
                bar_bounds: bounds,
                main_offset: -bounds.height as i32,
                cross_offset: 0,
                root_bounds_list: vec![bounds],
                path_highlight: Some(PathHighlight::MenuActive),
                style: std::borrow::Cow::Borrowed(&crate::theme::menu_bar::MenuBarStyle::Default),
                position: Point::new(translation.x, translation.y),
                is_overlay: true,
                window_id: window::Id::NONE,
                depth: 0,
                on_surface_action: None,
            }
            .overlay(),
        )
    }

    fn drag_destinations(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        let local_state = tree.state.downcast_ref::<LocalState>();
        let my_id = self.get_drag_id();
        let mut pushed = false;

        for item in self.variant_bounds(local_state, layout.bounds()) {
            if let ItemBounds::Button(_entity, rect) = item {
                pushed = true;
                log::trace!(
                    target: TAB_REORDER_LOG_TARGET,
                    "register drag destination id={:?} bounds=({:.2},{:.2},{:.2},{:.2}) mimes={:?}",
                    my_id,
                    rect.x,
                    rect.y,
                    rect.width,
                    rect.height,
                    self.mimes
                );
                dnd_rectangles.push(DndDestinationRectangle {
                    id: my_id,
                    rectangle: dnd::Rectangle {
                        x: f64::from(rect.x),
                        y: f64::from(rect.y),
                        width: f64::from(rect.width),
                        height: f64::from(rect.height),
                    },
                    mime_types: self.mimes.clone().into_iter().map(Cow::Owned).collect(),
                    actions: DndAction::Copy | DndAction::Move,
                    preferred: DndAction::Move,
                });
            }
        }

        if !pushed {
            let bounds = layout.bounds();
            log::trace!(
                target: TAB_REORDER_LOG_TARGET,
                "register drag destination id={:?} bounds=({:.2},{:.2},{:.2},{:.2}) mimes={:?}",
                my_id,
                bounds.x,
                bounds.y,
                bounds.width,
                bounds.height,
                self.mimes
            );
            dnd_rectangles.push(DndDestinationRectangle {
                id: my_id,
                rectangle: dnd::Rectangle {
                    x: f64::from(bounds.x),
                    y: f64::from(bounds.y),
                    width: f64::from(bounds.width),
                    height: f64::from(bounds.height),
                },
                mime_types: self.mimes.clone().into_iter().map(Cow::Owned).collect(),
                actions: DndAction::Copy | DndAction::Move,
                preferred: DndAction::Move,
            });
        }
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

struct TabDragSource<Message> {
    payload: Box<dyn Fn(Entity) -> Option<(String, Vec<u8>)>>,
    threshold: f32,
    _marker: PhantomData<Message>,
}

impl<Message> TabDragSource<Message> {
    fn new(payload: impl Fn(Entity) -> Option<(String, Vec<u8>)> + 'static) -> Self {
        Self {
            payload: Box::new(payload),
            threshold: 8.0,
            _marker: PhantomData,
        }
    }
}

struct SimpleDragData {
    mime: String,
    bytes: Vec<u8>,
}

impl SimpleDragData {
    fn new(mime: String, bytes: Vec<u8>) -> Self {
        Self { mime, bytes }
    }
}

impl iced::clipboard::mime::AsMimeTypes for SimpleDragData {
    fn available(&self) -> Cow<'static, [String]> {
        Cow::Owned(vec![self.mime.clone()])
    }

    fn as_bytes(&self, mime_type: &str) -> Option<Cow<'static, [u8]>> {
        if mime_type == self.mime {
            Some(Cow::Owned(self.bytes.clone()))
        } else {
            None
        }
    }
}

#[derive(Clone, Copy)]
struct TabDragCandidate {
    entity: Entity,
    bounds: Rectangle,
    origin: Point,
}

#[derive(Debug, Clone, Copy)]
struct Focus {
    updated_at: Instant,
    now: Instant,
}

/// State that is maintained by each individual widget.
pub struct LocalState {
    /// Menu state
    pub(crate) menu_state: MenuBarState,
    /// Defines how many buttons to show at a time.
    pub(super) buttons_visible: usize,
    /// Button visibility offset, when collapsed.
    pub(super) buttons_offset: usize,
    /// Whether buttons need to be collapsed to preserve minimum width
    pub(super) collapsed: bool,
    /// Visibility of focus state
    focused_visible: bool,
    /// If the widget is focused or not.
    focused: Option<Focus>,
    /// The key inside the widget that is currently focused.
    focused_item: Item,
    /// The ID of the button that is being hovered. Defaults to null.
    hovered: Item,
    /// The ID of the button that was middle-clicked, but not yet released.
    middle_clicked: Option<Item>,
    /// Last known length of the model.
    pub(super) known_length: usize,
    /// Dimensions of internal buttons when shrinking
    pub(super) internal_layout: Vec<(Size, Size)>,
    /// The paragraphs for each text.
    paragraphs: SecondaryMap<Entity, crate::Plain>,
    /// Used to detect changes in text.
    text_hashes: SecondaryMap<Entity, u64>,
    /// Location of cursor when context menu was opened.
    context_cursor: Point,
    /// Track whether an item is currently showing a context menu.
    show_context: Option<Entity>,
    /// Time since last tab activation from wheel movements.
    wheel_timestamp: Option<Instant>,
    /// Dnd state
    pub dnd_state: crate::widget::dnd_destination::State<Option<Entity>>,
    /// Tracks multi-touch events
    fingers_pressed: HashSet<Finger>,
    /// The currently pressed item
    pressed_item: Option<Item>,
    /// Pending tab drag candidate data
    tab_drag_candidate: Option<TabDragCandidate>,
    /// Currently dragging tab entity
    dragging_tab: Option<Entity>,
    /// Current drop hint for drag-and-drop indicator
    drop_hint: Option<DropHint>,
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

impl LocalState {
    fn set_focused(&mut self) {
        let now = Instant::now();
        LAST_FOCUS_UPDATE.with(|x| x.set(now));

        self.focused = Some(Focus {
            updated_at: now,
            now,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::segmented_button::{self, Appearance as SegAppearance};
    use iced::Size;
    use slotmap::SecondaryMap;
    use std::collections::HashSet;

    #[derive(Clone, Debug)]
    enum TestMessage {}

    struct TestVariant;

    impl<SelectionMode, Message> SegmentedVariant
        for SegmentedButton<'_, TestVariant, SelectionMode, Message>
    where
        Model<SelectionMode>: Selectable,
        SelectionMode: Default,
    {
        const VERTICAL: bool = false;

        fn variant_appearance(
            _theme: &crate::Theme,
            _style: &crate::theme::SegmentedButton,
        ) -> SegAppearance {
            SegAppearance::default()
        }

        fn variant_bounds<'b>(
            &'b self,
            _state: &'b LocalState,
            bounds: Rectangle,
        ) -> Box<dyn Iterator<Item = ItemBounds> + 'b> {
            let len = self.model.order.len();
            if len == 0 {
                return Box::new(std::iter::empty());
            }
            let width = bounds.width / len as f32;
            Box::new(
                self.model
                    .order
                    .iter()
                    .copied()
                    .enumerate()
                    .map(move |(idx, entity)| {
                        let rect = Rectangle {
                            x: bounds.x + (idx as f32) * width,
                            y: bounds.y,
                            width,
                            height: bounds.height,
                        };
                        ItemBounds::Button(entity, rect)
                    }),
            )
        }

        fn variant_layout(
            &self,
            _state: &mut LocalState,
            _renderer: &crate::Renderer,
            _limits: &layout::Limits,
        ) -> Size {
            Size::ZERO
        }
    }

    fn sample_model() -> (
        segmented_button::SingleSelectModel,
        Vec<segmented_button::Entity>,
    ) {
        let mut entities = Vec::new();
        let model = segmented_button::Model::builder()
            .insert(|b| b.text("One").with_id(|id| entities.push(id)))
            .insert(|b| b.text("Two").with_id(|id| entities.push(id)))
            .insert(|b| b.text("Three").with_id(|id| entities.push(id)))
            .build();
        (model, entities)
    }

    fn test_state(dragging: segmented_button::Entity, len: usize) -> LocalState {
        let mut state = LocalState {
            menu_state: MenuBarState::default(),
            paragraphs: SecondaryMap::new(),
            text_hashes: SecondaryMap::new(),
            buttons_visible: 0,
            buttons_offset: 0,
            collapsed: false,
            focused: None,
            focused_item: Item::default(),
            focused_visible: false,
            hovered: Item::default(),
            known_length: 0,
            middle_clicked: None,
            internal_layout: Vec::new(),
            context_cursor: Point::ORIGIN,
            show_context: None,
            wheel_timestamp: None,
            dnd_state: crate::widget::dnd_destination::State::<Option<Entity>>::new(),
            fingers_pressed: HashSet::new(),
            pressed_item: None,
            tab_drag_candidate: None,
            dragging_tab: Some(dragging),
            drop_hint: None,
        };
        state.buttons_visible = len;
        state.known_length = len;
        state
    }

    #[test]
    fn drop_hint_reports_before_and_after() {
        let (model, ids) = sample_model();
        let button =
            SegmentedButton::<TestVariant, segmented_button::SingleSelect, TestMessage>::new(
                &model,
            );
        let state = test_state(ids[0], model.order.len());
        let bounds = Rectangle {
            x: 0.0,
            y: 0.0,
            width: 300.0,
            height: 30.0,
        };
        let before = button
            .drop_hint_for_position(&state, bounds, Point::new(10.0, 15.0))
            .expect("hint");
        assert_eq!(before.entity, ids[0]);
        assert!(matches!(before.side, DropSide::Before));

        let after = button
            .drop_hint_for_position(&state, bounds, Point::new(290.0, 15.0))
            .expect("hint");
        assert_eq!(after.entity, ids[2]);
        assert!(matches!(after.side, DropSide::After));
    }
}

impl operation::Focusable for LocalState {
    fn is_focused(&self) -> bool {
        self.focused
            .is_some_and(|f| f.updated_at == LAST_FOCUS_UPDATE.with(|f| f.get()))
    }

    fn focus(&mut self) {
        self.set_focused();
        self.focused_visible = true;
        self.focused_item = Item::Set;
    }

    fn unfocus(&mut self) {
        self.focused = None;
        self.focused_item = Item::None;
        self.focused_visible = false;
        self.show_context = None;
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
    #[inline]
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

fn draw_drop_indicator(
    renderer: &mut Renderer,
    bounds: Rectangle,
    side: DropSide,
    vertical: bool,
    color: Color,
) {
    let thickness = 4.0;
    let quad_bounds = if vertical {
        let y = match side {
            DropSide::Before => bounds.y - thickness / 2.0,
            DropSide::After => bounds.y + bounds.height - thickness / 2.0,
        };

        Rectangle {
            x: bounds.x,
            y,
            width: bounds.width,
            height: thickness,
        }
    } else {
        let x = match side {
            DropSide::Before => bounds.x - thickness / 2.0,
            DropSide::After => bounds.x + bounds.width - thickness / 2.0,
        };

        Rectangle {
            x,
            y: bounds.y,
            width: thickness,
            height: bounds.height,
        }
    };

    renderer.fill_quad(
        renderer::Quad {
            bounds: quad_bounds,
            border: Border {
                radius: 2.0.into(),
                ..Default::default()
            },
            shadow: Shadow::default(),
        },
        Background::Color(color),
    );
}

fn left_button_released(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left,))
    )
}

fn right_button_released(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right,))
    )
}

fn is_pressed(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. })
    )
}

fn is_lifted(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left,))
            | Event::Touch(touch::Event::FingerLifted { .. })
    )
}

fn touch_lifted(event: &Event) -> bool {
    matches!(event, Event::Touch(touch::Event::FingerLifted { .. }))
}
