// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::{Element, Renderer};
use iced::{Alignment, Pixels, alignment};
use iced_core::event::Event;
use iced_core::layout::{self, Layout};
use iced_core::mouse::{self, Cursor};
use iced_core::widget::Operation;
use iced_core::widget::tree::{self, Tree};
#[cfg(feature = "wgpu")]
use iced_core::{Background, Border, Shadow};
use iced_core::{
    Clipboard, Length, Padding, Point, Rectangle, Renderer as _, Shell, Size, Vector, Widget,
    overlay, renderer, window,
};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::time::{Duration, Instant};

const DEFAULT_ANIMATION_DURATION: Duration = Duration::from_millis(180);
const DEFAULT_DRAG_LIFT: f32 = 10.0;
const DEFAULT_DRAG_THRESHOLD: f32 = 6.0;
const SHADOW_BLUR_RADIUS: f32 = 20.0;
const POSITION_EPSILON: f32 = 0.5;

#[derive(Debug, Clone)]
struct SlotAnimation {
    from: Point,
    to: Point,
    started_at: Option<Instant>,
}

impl SlotAnimation {
    fn new(position: Point) -> Self {
        Self {
            from: position,
            to: position,
            started_at: None,
        }
    }

    fn current_position(&self, duration: Duration) -> Point {
        let Some(started_at) = self.started_at else {
            return self.to;
        };

        let duration_secs = duration.as_secs_f32();
        if duration_secs <= f32::EPSILON {
            return self.to;
        }

        let progress = (started_at.elapsed().as_secs_f32() / duration_secs).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - progress).powi(3);

        Point::new(
            self.from.x + (self.to.x - self.from.x) * eased,
            self.from.y + (self.to.y - self.from.y) * eased,
        )
    }

    fn retarget(&mut self, new_target: Point, duration: Duration) {
        if approx_eq_point(self.to, new_target) {
            if !self.is_animating(duration) {
                self.from = new_target;
                self.to = new_target;
                self.started_at = None;
            }
            return;
        }

        self.from = self.current_position(duration);
        self.to = new_target;
        self.started_at = Some(Instant::now());
    }

    fn is_animating(&self, duration: Duration) -> bool {
        self.started_at
            .is_some_and(|started_at| started_at.elapsed() < duration)
    }

    fn finish_if_done(&mut self, duration: Duration) {
        if self
            .started_at
            .is_some_and(|started_at| started_at.elapsed() >= duration)
        {
            self.from = self.to;
            self.started_at = None;
        }
    }
}

#[derive(Debug, Clone)]
struct PendingDragState<Key>
where
    Key: Clone + Eq + Hash + 'static,
{
    key: Key,
    item_index: usize,
    original_index: usize,
    press_local: Point,
    pointer_offset: Vector,
}

#[derive(Debug, Clone)]
struct DragState<Key>
where
    Key: Clone + Eq + Hash + 'static,
{
    key: Key,
    item_index: usize,
    original_index: usize,
    current_index: usize,
    cursor_local: Point,
    pointer_offset: Vector,
}

#[derive(Debug, Clone)]
struct State<Key>
where
    Key: Clone + Eq + Hash + 'static,
{
    keys: Vec<Key>,
    slot_positions: HashMap<Key, SlotAnimation>,
    pending_drag: Option<PendingDragState<Key>>,
    drag: Option<DragState<Key>>,
    wrap_width: f32,
    initialized: bool,
}

impl<Key> Default for State<Key>
where
    Key: Clone + Eq + Hash + 'static,
{
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            slot_positions: HashMap::new(),
            pending_drag: None,
            drag: None,
            wrap_width: f32::INFINITY,
            initialized: false,
        }
    }
}

impl<Key> State<Key>
where
    Key: Clone + Eq + Hash + 'static,
{
    fn retain_keys(&mut self, keys: &[Key]) {
        let keep: HashSet<_> = keys.iter().cloned().collect();
        self.slot_positions.retain(|key, _| keep.contains(key));

        if self
            .pending_drag
            .as_ref()
            .is_some_and(|drag| !keep.contains(&drag.key))
        {
            self.pending_drag = None;
        }

        if self
            .drag
            .as_ref()
            .is_some_and(|drag| !keep.contains(&drag.key))
        {
            self.drag = None;
        }
    }

    fn finish_animations(&mut self, duration: Duration) {
        self.slot_positions
            .values_mut()
            .for_each(|slot| slot.finish_if_done(duration));
    }

    fn is_animating(&self, duration: Duration) -> bool {
        self.slot_positions
            .values()
            .any(|slot| slot.is_animating(duration))
    }

    fn current_slot_position(&self, key: &Key, fallback: Point, duration: Duration) -> Point {
        self.slot_positions
            .get(key)
            .map(|slot| slot.current_position(duration))
            .unwrap_or(fallback)
    }

    fn retarget_slot(&mut self, key: &Key, target: Point, duration: Duration) {
        self.slot_positions
            .entry(key.clone())
            .or_insert_with(|| SlotAnimation::new(target))
            .retarget(target, duration);
    }

    fn snap_slot(&mut self, key: &Key, target: Point) {
        self.slot_positions
            .insert(key.clone(), SlotAnimation::new(target));
    }

    fn apply_layout_position(&mut self, key: &Key, target: Point, duration: Duration) {
        if self.initialized {
            self.retarget_slot(key, target, duration);
        } else {
            self.snap_slot(key, target);
        }
    }
}

#[derive(Debug, Clone)]
struct LocalSlot<Key>
where
    Key: Clone + Eq + Hash + 'static,
{
    key: Key,
    locked: bool,
    bounds: Rectangle,
}

/// A horizontal flex row with drag-to-reorder behavior.
#[must_use]
pub struct ReorderableFlexRow<'a, Key, Message>
where
    Key: Clone + Eq + Hash + 'static,
{
    spacing: f32,
    padding: Padding,
    width: Length,
    height: Length,
    align: Alignment,
    clip: bool,
    drag_lift: f32,
    animation_duration: Duration,
    on_reorder: Box<dyn Fn(Vec<Key>) -> Message + 'a>,
    keys: Vec<Key>,
    locked: Vec<bool>,
    children: Vec<Element<'a, Message>>,
}

impl<'a, Key, Message> ReorderableFlexRow<'a, Key, Message>
where
    Key: Clone + Eq + Hash + 'static,
{
    pub fn new(on_reorder: impl Fn(Vec<Key>) -> Message + 'a) -> Self {
        Self {
            spacing: 0.0,
            padding: Padding::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
            align: Alignment::Start,
            clip: false,
            drag_lift: DEFAULT_DRAG_LIFT,
            animation_duration: DEFAULT_ANIMATION_DURATION,
            on_reorder: Box::new(on_reorder),
            keys: Vec::new(),
            locked: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize, on_reorder: impl Fn(Vec<Key>) -> Message + 'a) -> Self {
        let mut row = Self::new(on_reorder);
        row.keys = Vec::with_capacity(capacity);
        row.locked = Vec::with_capacity(capacity);
        row.children = Vec::with_capacity(capacity);
        row
    }

    pub fn spacing(mut self, amount: impl Into<Pixels>) -> Self {
        self.spacing = amount.into().0;
        self
    }

    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn align_y(mut self, align: impl Into<alignment::Vertical>) -> Self {
        self.align = Alignment::from(align.into());
        self
    }

    /// Leave disabled for dragged item to visibly lift above the row.
    pub fn clip(mut self, clip: bool) -> Self {
        self.clip = clip;
        self
    }

    pub fn drag_lift(mut self, lift: f32) -> Self {
        self.drag_lift = lift.max(0.0);
        self
    }

    pub fn animation_duration(mut self, duration: Duration) -> Self {
        self.animation_duration = duration;
        self
    }

    pub fn push(self, key: Key, child: impl Into<Element<'a, Message>>) -> Self {
        self.push_with_lock(key, false, child)
    }

    /// Item stays fixed, never participates in reordering.
    pub fn push_locked(self, key: Key, child: impl Into<Element<'a, Message>>) -> Self {
        self.push_with_lock(key, true, child)
    }

    fn push_with_lock(
        mut self,
        key: Key,
        locked: bool,
        child: impl Into<Element<'a, Message>>,
    ) -> Self {
        let child = child.into();
        let child_size = child.as_widget().size_hint();

        if !child_size.is_void() {
            self.width = self.width.enclose(child_size.width);
            self.height = self.height.enclose(child_size.height);
            self.keys.push(key);
            self.locked.push(locked);
            self.children.push(child);
        }

        self
    }

    pub fn extend(self, items: impl IntoIterator<Item = (Key, Element<'a, Message>)>) -> Self {
        items
            .into_iter()
            .fold(self, |row, (key, child)| row.push(key, child))
    }

    pub fn extend_locked(
        self,
        items: impl IntoIterator<Item = (Key, Element<'a, Message>)>,
    ) -> Self {
        items
            .into_iter()
            .fold(self, |row, (key, child)| row.push_locked(key, child))
    }

    fn item_local_slots_from_layout(
        &self,
        bounds: Rectangle,
        child_layouts: &[Layout<'_>],
    ) -> Vec<LocalSlot<Key>> {
        self.keys
            .iter()
            .zip(self.locked.iter())
            .zip(child_layouts.iter())
            .map(|((key, locked), child_layout)| {
                let child_bounds = child_layout.bounds();
                LocalSlot {
                    key: key.clone(),
                    locked: *locked,
                    bounds: Rectangle {
                        x: child_bounds.x - bounds.x,
                        y: child_bounds.y - bounds.y,
                        width: child_bounds.width,
                        height: child_bounds.height,
                    },
                }
            })
            .collect()
    }

    fn sync_slot_positions(&self, state: &mut State<Key>, slots: &[LocalSlot<Key>]) {
        let ordered_keys: Vec<Key> = slots.iter().map(|slot| slot.key.clone()).collect();
        state.retain_keys(&ordered_keys);

        let size_by_key: HashMap<Key, Size> = slots
            .iter()
            .map(|slot| {
                (
                    slot.key.clone(),
                    Size::new(slot.bounds.width, slot.bounds.height),
                )
            })
            .collect();
        let locked_by_key: HashMap<Key, bool> = slots
            .iter()
            .map(|slot| (slot.key.clone(), slot.locked))
            .collect();

        let Some(drag_snapshot) = state.drag.as_ref().map(|drag| {
            (
                drag.key.clone(),
                drag.cursor_local,
                drag.pointer_offset,
                drag.item_index,
            )
        }) else {
            for slot in slots {
                state.apply_layout_position(
                    &slot.key,
                    Point::new(slot.bounds.x, slot.bounds.y),
                    self.animation_duration,
                );
            }
            return;
        };

        let (drag_key, cursor_local, pointer_offset, drag_item_index) = drag_snapshot;

        let Some(dragged_slot) = slots.iter().find(|slot| slot.key == drag_key) else {
            state.drag = None;
            for slot in slots {
                state.apply_layout_position(
                    &slot.key,
                    Point::new(slot.bounds.x, slot.bounds.y),
                    self.animation_duration,
                );
            }
            return;
        };

        if dragged_slot.locked {
            state.drag = None;
            return;
        }

        let dragged_center = Point::new(
            cursor_local.x - pointer_offset.x + dragged_slot.bounds.width / 2.0,
            cursor_local.y - pointer_offset.y + dragged_slot.bounds.height / 2.0,
        );
        let target_index = target_index_for_drag(slots, &drag_key, dragged_center);
        let prior_index = state.drag.as_ref().map(|drag| drag.current_index);

        if let Some(drag) = state.drag.as_mut() {
            drag.current_index = target_index;
            drag.item_index = drag_item_index;
        }

        if prior_index == Some(target_index) {
            return;
        }

        let reordered_keys =
            reordered_keys_for_drag(&ordered_keys, &locked_by_key, &drag_key, target_index);
        let (target_slots, _) = compute_wrapped_slots(
            &reordered_keys,
            &locked_by_key,
            &size_by_key,
            state.wrap_width,
            self.padding,
            self.spacing,
            self.align,
        );
        let target_positions: HashMap<Key, Point> = target_slots
            .iter()
            .map(|slot| (slot.key.clone(), Point::new(slot.bounds.x, slot.bounds.y)))
            .collect();

        for slot in slots {
            let target = target_positions
                .get(&slot.key)
                .copied()
                .unwrap_or(Point::new(slot.bounds.x, slot.bounds.y));
            state.retarget_slot(&slot.key, target, self.animation_duration);
        }
    }
}

impl<'a, Key, Message> Widget<Message, crate::Theme, Renderer>
    for ReorderableFlexRow<'a, Key, Message>
where
    Key: Clone + Eq + Hash + 'static,
    Message: 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Key>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            keys: self.keys.clone(),
            ..State::default()
        })
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&mut self, tree: &mut Tree) {
        let Tree {
            state, children, ..
        } = tree;
        let state = state.downcast_mut::<State<Key>>();
        let previous_keys = state.keys.clone();
        let previous_children = std::mem::take(children);
        let mut previous_by_key = HashMap::with_capacity(previous_keys.len());

        for (key, child_tree) in previous_keys.into_iter().zip(previous_children) {
            previous_by_key.insert(key, child_tree);
        }

        children.reserve(self.children.len());

        for (key, child) in self.keys.iter().cloned().zip(self.children.iter_mut()) {
            let mut child_tree = previous_by_key
                .remove(&key)
                .unwrap_or_else(|| Tree::new(child.as_widget()));
            child.as_widget_mut().diff(&mut child_tree);
            children.push(child_tree);
        }

        if state.keys != self.keys {
            state.keys.clone_from(&self.keys);
        }
        state.retain_keys(&self.keys);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits
            .width(self.width)
            .height(self.height)
            .shrink(self.padding);
        let child_limits = limits.loose();
        let wrap_width = limits.max().width;

        let mut nodes = Vec::with_capacity(self.children.len());
        let mut size_by_key = HashMap::with_capacity(self.children.len());
        let locked_by_key: HashMap<Key, bool> = self
            .keys
            .iter()
            .cloned()
            .zip(self.locked.iter().copied())
            .collect();

        for ((key, child), child_tree) in self
            .keys
            .iter()
            .zip(self.children.iter_mut())
            .zip(tree.children.iter_mut())
        {
            let node = child
                .as_widget_mut()
                .layout(child_tree, renderer, &child_limits);
            size_by_key.insert(key.clone(), node.size());
            nodes.push(node);
        }

        let (slots, intrinsic_size) = compute_wrapped_slots(
            &self.keys,
            &locked_by_key,
            &size_by_key,
            wrap_width,
            self.padding,
            self.spacing,
            self.align,
        );

        for (node, slot) in nodes.iter_mut().zip(&slots) {
            node.move_to_mut(Point::new(slot.bounds.x, slot.bounds.y));
        }

        let size = limits.resolve(self.width, self.height, intrinsic_size);
        let node = layout::Node::with_children(size.expand(self.padding), nodes);
        let state = tree.state.downcast_mut::<State<Key>>();
        state.wrap_width = wrap_width;
        self.sync_slot_positions(state, &slots);

        node
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.children
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((child, state), child_layout)| {
                    child.as_widget_mut().operate(
                        state,
                        child_layout.with_virtual_offset(layout.virtual_offset()),
                        renderer,
                        operation,
                    );
                });
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State<Key>>();
        let child_layouts: Vec<_> = layout.children().collect();

        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            state.initialized = true;
            state.finish_animations(self.animation_duration);
            if state.drag.is_some() || state.is_animating(self.animation_duration) {
                shell.request_redraw();
            }
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if state.drag.is_none()
                    && state.pending_drag.is_none()
                    && let Some(cursor_pos) = cursor.position()
                    && let Some((index, child_layout)) = child_layouts
                        .iter()
                        .enumerate()
                        .find(|(_, child_layout)| child_layout.bounds().contains(cursor_pos))
                    && !self.locked.get(index).copied().unwrap_or(false)
                    && let Some(reorder_index) = reorderable_index_for_child(&self.locked, index)
                {
                    let child_bounds = child_layout.bounds();
                    state.pending_drag = Some(PendingDragState {
                        key: self.keys[index].clone(),
                        item_index: index,
                        original_index: reorder_index,
                        press_local: Point::new(cursor_pos.x - bounds.x, cursor_pos.y - bounds.y),
                        pointer_offset: Vector::new(
                            cursor_pos.x - child_bounds.x,
                            cursor_pos.y - child_bounds.y,
                        ),
                    });
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(pending) = state.pending_drag.clone()
                    && let Some(cursor_pos) = cursor.position()
                {
                    let cursor_local = Point::new(cursor_pos.x - bounds.x, cursor_pos.y - bounds.y);
                    let delta = Vector::new(
                        cursor_local.x - pending.press_local.x,
                        cursor_local.y - pending.press_local.y,
                    );
                    let distance = (delta.x.powi(2) + delta.y.powi(2)).sqrt();

                    if distance >= DEFAULT_DRAG_THRESHOLD {
                        if let (Some(child), Some(child_tree), Some(child_layout)) = (
                            self.children.get_mut(pending.item_index),
                            tree.children.get_mut(pending.item_index),
                            child_layouts.get(pending.item_index),
                        ) {
                            let cursor_left = Event::Mouse(mouse::Event::CursorLeft);
                            child.as_widget_mut().update(
                                child_tree,
                                &cursor_left,
                                child_layout.with_virtual_offset(layout.virtual_offset()),
                                cursor,
                                renderer,
                                clipboard,
                                shell,
                                viewport,
                            );
                        }

                        state.pending_drag = None;
                        state.drag = Some(DragState {
                            key: pending.key,
                            item_index: pending.item_index,
                            original_index: pending.original_index,
                            current_index: pending.original_index,
                            cursor_local,
                            pointer_offset: pending.pointer_offset,
                        });
                        let slots = self.item_local_slots_from_layout(bounds, &child_layouts);
                        self.sync_slot_positions(state, &slots);
                        shell.capture_event();
                        shell.request_redraw();
                        return;
                    }
                }

                if let Some(drag) = state.drag.as_mut()
                    && let Some(cursor_pos) = cursor.position()
                {
                    drag.cursor_local =
                        Point::new(cursor_pos.x - bounds.x, cursor_pos.y - bounds.y);
                    let slots = self.item_local_slots_from_layout(bounds, &child_layouts);
                    self.sync_slot_positions(state, &slots);
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                state.pending_drag = None;

                if state.drag.is_some() {
                    let slots = self.item_local_slots_from_layout(bounds, &child_layouts);
                    self.sync_slot_positions(state, &slots);
                }
                if let Some(drag) = state.drag.take() {
                    if drag.current_index != drag.original_index {
                        let locked_by_key: HashMap<Key, bool> = self
                            .keys
                            .iter()
                            .cloned()
                            .zip(self.locked.iter().copied())
                            .collect();
                        let new_order = reordered_keys_for_drag(
                            &self.keys,
                            &locked_by_key,
                            &drag.key,
                            drag.current_index,
                        );
                        shell.publish((self.on_reorder)(new_order));
                    }
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
            }
            _ => {}
        }

        if state.drag.is_some() {
            return;
        }

        for ((item, tree), child_layout) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(child_layouts.into_iter())
        {
            item.as_widget_mut().update(
                tree,
                event,
                child_layout.with_virtual_offset(layout.virtual_offset()),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State<Key>>();

        if state.drag.is_some() {
            return mouse::Interaction::Grabbing;
        }

        if let Some(cursor_pos) = cursor.position()
            && self
                .locked
                .iter()
                .zip(layout.children())
                .any(|(locked, child_layout)| {
                    !*locked && child_layout.bounds().contains(cursor_pos)
                })
        {
            return mouse::Interaction::Grab;
        }

        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, tree), child_layout)| {
                child.as_widget().mouse_interaction(
                    tree,
                    child_layout.with_virtual_offset(layout.virtual_offset()),
                    cursor,
                    viewport,
                    renderer,
                )
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Key>>();
        let bounds = layout.bounds();

        let Some(clipped_viewport) = bounds.intersection(viewport) else {
            return;
        };

        let viewport = if self.clip {
            &clipped_viewport
        } else {
            viewport
        };
        let drag_key = state.drag.as_ref().map(|drag| &drag.key);
        let drag_item = state.drag.as_ref().and_then(|drag| {
            self.keys
                .iter()
                .zip(&self.children)
                .zip(&tree.children)
                .zip(layout.children())
                .find_map(|(((key, child), state), child_layout)| {
                    (key == &drag.key).then_some((key, child, state, child_layout, drag))
                })
        });

        for (((key, child), child_tree), child_layout) in self
            .keys
            .iter()
            .zip(&self.children)
            .zip(&tree.children)
            .zip(layout.children())
        {
            if drag_key.is_some_and(|drag_key| drag_key == key) {
                continue;
            }

            let child_layout = child_layout.with_virtual_offset(layout.virtual_offset());
            let child_bounds = child_layout.bounds();
            let base_local = Point::new(child_bounds.x - bounds.x, child_bounds.y - bounds.y);
            let target_local =
                state.current_slot_position(key, base_local, self.animation_duration);
            let translation =
                Vector::new(target_local.x - base_local.x, target_local.y - base_local.y);
            let translated_bounds = translate_rect(child_bounds, translation);

            if translated_bounds.intersects(viewport) {
                renderer.with_translation(translation, |renderer| {
                    child.as_widget().draw(
                        child_tree,
                        renderer,
                        theme,
                        style,
                        child_layout,
                        cursor,
                        viewport,
                    );
                });
            }
        }

        if let Some((_key, child, child_tree, child_layout, drag)) = drag_item {
            let child_layout = child_layout.with_virtual_offset(layout.virtual_offset());
            let child_bounds = child_layout.bounds();
            let base_local = Point::new(child_bounds.x - bounds.x, child_bounds.y - bounds.y);
            let drag_local = Point::new(
                drag.cursor_local.x - drag.pointer_offset.x,
                drag.cursor_local.y - drag.pointer_offset.y - self.drag_lift,
            );
            let translation = Vector::new(drag_local.x - base_local.x, drag_local.y - base_local.y);

            #[cfg(feature = "wgpu")]
            {
                let translated_bounds = translate_rect(child_bounds, translation);
                draw_drag_backdrop(renderer, theme, translated_bounds);
            }

            renderer.with_translation(translation, |renderer| {
                child.as_widget().draw(
                    child_tree,
                    renderer,
                    theme,
                    style,
                    child_layout,
                    cursor,
                    viewport,
                );
            });
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, Renderer>> {
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }

    fn drag_destinations(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        dnd_rectangles: &mut iced_core::clipboard::DndDestinationRectangles,
    ) {
        for ((item, child_layout), child_state) in self
            .children
            .iter()
            .zip(layout.children())
            .zip(state.children.iter())
        {
            item.as_widget().drag_destinations(
                child_state,
                child_layout.with_virtual_offset(layout.virtual_offset()),
                renderer,
                dnd_rectangles,
            );
        }
    }
}

impl<'a, Key, Message> From<ReorderableFlexRow<'a, Key, Message>> for Element<'a, Message>
where
    Key: Clone + Eq + Hash + 'static,
    Message: 'a,
{
    fn from(row: ReorderableFlexRow<'a, Key, Message>) -> Self {
        Element::new(row)
    }
}

/// Create a horizontal flex row with drag-to-reorder behavior.
pub fn reorderable_flex_row<'a, Key, Message>(
    on_reorder: impl Fn(Vec<Key>) -> Message + 'a,
) -> ReorderableFlexRow<'a, Key, Message>
where
    Key: Clone + Eq + Hash + 'static,
{
    ReorderableFlexRow::new(on_reorder)
}

fn compute_wrapped_slots<Key>(
    ordered_keys: &[Key],
    locked_by_key: &HashMap<Key, bool>,
    size_by_key: &HashMap<Key, Size>,
    wrap_width: f32,
    padding: Padding,
    spacing: f32,
    align: Alignment,
) -> (Vec<LocalSlot<Key>>, Size)
where
    Key: Clone + Eq + Hash + 'static,
{
    let wrap_width = if wrap_width.is_finite() {
        wrap_width.max(0.0)
    } else {
        f32::INFINITY
    };

    let mut slots = Vec::with_capacity(ordered_keys.len());
    let mut intrinsic_size = Size::ZERO;
    let mut row_start = 0;
    let mut row_height = 0.0;
    let mut x = 0.0;
    let mut y = 0.0;

    let align_factor = match align {
        Alignment::Start => 0.0,
        Alignment::Center => 2.0,
        Alignment::End => 1.0,
    };

    let align_row =
        |range: std::ops::Range<usize>, row_height: f32, slots: &mut [LocalSlot<Key>]| {
            if align_factor == 0.0 {
                return;
            }

            for slot in &mut slots[range] {
                slot.bounds.y += (row_height - slot.bounds.height) / align_factor;
            }
        };

    for (index, key) in ordered_keys.iter().enumerate() {
        let size = size_by_key.get(key).copied().unwrap_or(Size::ZERO);

        if x != 0.0 && x + size.width > wrap_width {
            intrinsic_size.width = intrinsic_size.width.max(x - spacing);
            align_row(row_start..index, row_height, &mut slots);
            y += row_height + spacing;
            x = 0.0;
            row_start = index;
            row_height = 0.0;
        }

        row_height = row_height.max(size.height);

        slots.push(LocalSlot {
            key: key.clone(),
            locked: locked_by_key.get(key).copied().unwrap_or(false),
            bounds: Rectangle {
                x: x + padding.left,
                y: y + padding.top,
                width: size.width,
                height: size.height,
            },
        });

        x += size.width + spacing;
    }

    if x != 0.0 {
        intrinsic_size.width = intrinsic_size.width.max(x - spacing);
    }

    intrinsic_size.height = y + row_height;
    align_row(row_start..slots.len(), row_height, &mut slots);

    (slots, intrinsic_size)
}

fn reordered_keys_for_drag<Key>(
    ordered_keys: &[Key],
    locked_by_key: &HashMap<Key, bool>,
    dragged_key: &Key,
    target_index: usize,
) -> Vec<Key>
where
    Key: Clone + Eq + Hash + 'static,
{
    let movable_keys: Vec<Key> = ordered_keys
        .iter()
        .filter(|key| !locked_by_key.get(*key).copied().unwrap_or(false))
        .cloned()
        .collect();
    let mut remaining: Vec<Key> = movable_keys
        .iter()
        .filter(|key| *key != dragged_key)
        .cloned()
        .collect();

    remaining.insert(target_index.min(remaining.len()), dragged_key.clone());
    merge_locked_and_reordered_keys(ordered_keys, locked_by_key, &remaining)
}

/// Picks insertion index among movable items using row-aware midpoint rule.
///
/// Walks laid-out slots in reading order, counting how many non-dragged movable
/// items the cursor has moved past. Skips locked slots. O(n) single pass, no
/// allocations.
fn target_index_for_drag<Key>(
    slots: &[LocalSlot<Key>],
    dragged_key: &Key,
    dragged_center: Point,
) -> usize
where
    Key: Clone + Eq + Hash + 'static,
{
    let mut target = 0;
    let mut passed_movable: usize = 0;
    let mut found_target = false;

    let mut i = 0;
    while i < slots.len() {
        let slot = &slots[i];

        if slot.locked || &slot.key == dragged_key {
            i += 1;
            continue;
        }

        let row_top = slot.bounds.y;
        let row_bottom = row_top + slot.bounds.height;

        if !found_target && dragged_center.y < row_top {
            target = passed_movable;
            found_target = true;
            break;
        }

        if dragged_center.y > row_bottom {
            passed_movable += 1;
            i += 1;
            continue;
        }

        let center_x = slot.bounds.x + slot.bounds.width / 2.0;
        if dragged_center.x < center_x {
            target = passed_movable;
            found_target = true;
            break;
        }

        passed_movable += 1;
        i += 1;
    }

    if !found_target {
        target = passed_movable;
    }

    target
}

fn reorderable_index_for_child(locked: &[bool], item_index: usize) -> Option<usize> {
    (!locked.get(item_index).copied().unwrap_or(false)).then(|| {
        locked[..item_index]
            .iter()
            .filter(|is_locked| !**is_locked)
            .count()
    })
}

fn merge_locked_and_reordered_keys<Key>(
    ordered_keys: &[Key],
    locked_by_key: &HashMap<Key, bool>,
    reordered_movable_keys: &[Key],
) -> Vec<Key>
where
    Key: Clone + Eq + Hash + 'static,
{
    let mut movable = reordered_movable_keys.iter();

    ordered_keys
        .iter()
        .map(|key| {
            if locked_by_key.get(key).copied().unwrap_or(false) {
                key.clone()
            } else {
                movable.next().cloned().unwrap_or_else(|| key.clone())
            }
        })
        .collect()
}

fn approx_eq_point(a: Point, b: Point) -> bool {
    (a.x - b.x).abs() <= POSITION_EPSILON && (a.y - b.y).abs() <= POSITION_EPSILON
}

fn translate_rect(bounds: Rectangle, translation: Vector) -> Rectangle {
    Rectangle {
        x: bounds.x + translation.x,
        y: bounds.y + translation.y,
        width: bounds.width,
        height: bounds.height,
    }
}

#[cfg(feature = "wgpu")]
fn draw_drag_backdrop(renderer: &mut Renderer, theme: &crate::Theme, bounds: Rectangle) {
    let cosmic = theme.cosmic();
    let backdrop = iced::Color {
        a: 0.08,
        ..iced::Color::from(cosmic.bg_component_color())
    };

    renderer.fill_quad(
        renderer::Quad {
            bounds,
            border: Border {
                radius: cosmic.corner_radii.radius_m.into(),
                ..Border::default()
            },
            shadow: Shadow {
                color: cosmic.shade.into(),
                offset: Vector::new(0.0, 8.0),
                blur_radius: SHADOW_BLUR_RADIUS,
            },
            snap: true,
        },
        Background::Color(backdrop),
    );
}

#[cfg(test)]
mod tests {
    use super::{compute_wrapped_slots, reordered_keys_for_drag, target_index_for_drag};
    use iced::{Alignment, Padding, Point, Size};
    use std::collections::HashMap;

    fn size_map(keys: &[usize], width: f32, height: f32) -> HashMap<usize, Size> {
        keys.iter()
            .copied()
            .map(|key| (key, Size::new(width, height)))
            .collect()
    }

    fn locked_map(keys: &[usize], locked_keys: &[usize]) -> HashMap<usize, bool> {
        keys.iter()
            .copied()
            .map(|key| (key, locked_keys.contains(&key)))
            .collect()
    }

    #[test]
    fn compute_wrapped_slots_creates_new_rows() {
        let ordered_keys = vec![0, 1, 2];
        let locked_by_key = locked_map(&ordered_keys, &[]);
        let size_by_key = size_map(&ordered_keys, 100.0, 40.0);
        let (slots, intrinsic_size) = compute_wrapped_slots(
            &ordered_keys,
            &locked_by_key,
            &size_by_key,
            220.0,
            Padding::ZERO,
            10.0,
            Alignment::Start,
        );

        assert_eq!(slots[0].bounds.x, 0.0);
        assert_eq!(slots[0].bounds.y, 0.0);
        assert_eq!(slots[1].bounds.x, 110.0);
        assert_eq!(slots[1].bounds.y, 0.0);
        assert_eq!(slots[2].bounds.x, 0.0);
        assert_eq!(slots[2].bounds.y, 50.0);
        assert_eq!(intrinsic_size.width, 210.0);
        assert_eq!(intrinsic_size.height, 90.0);
    }

    #[test]
    fn reordered_keys_for_drag_inserts_key_at_target_index() {
        let keys = [0, 1, 2, 3];
        let locked_by_key = locked_map(&keys, &[]);
        let reordered = reordered_keys_for_drag(&keys, &locked_by_key, &0, 3);
        assert_eq!(reordered, vec![1, 2, 3, 0]);
    }

    #[test]
    fn target_index_tracks_wrapped_drop_positions() {
        let ordered_keys = vec![0, 1, 2, 3];
        let locked_by_key = locked_map(&ordered_keys, &[]);
        let size_by_key = size_map(&ordered_keys, 100.0, 40.0);

        let (slots, _) = compute_wrapped_slots(
            &ordered_keys,
            &locked_by_key,
            &size_by_key,
            220.0,
            Padding::ZERO,
            10.0,
            Alignment::Start,
        );

        let target_index = target_index_for_drag(&slots, &0, Point::new(160.0, 70.0));

        assert_eq!(target_index, 3);
    }

    #[test]
    fn reordered_keys_for_drag_preserves_locked_positions() {
        let keys = [10, 11, 12, 13];
        let locked_by_key = locked_map(&keys, &[10, 13]);
        let reordered = reordered_keys_for_drag(&keys, &locked_by_key, &11, 1);

        assert_eq!(reordered, vec![10, 12, 11, 13]);
    }
}
