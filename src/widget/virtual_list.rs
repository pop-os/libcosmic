// Copyright 2026 System76 / local COSMIC performance work
// SPDX-License-Identifier: MPL-2.0

//! Virtualized vertical lists — browser-style list performance for iced/COSMIC.
//!
//! # Cost model (keyboard nav)
//!
//! | Frame type | Rebuild row widgets | Remeasure text/layout | Work |
//! |------------|---------------------|------------------------|------|
//! | Focus only (range stable) | No | No | paint selection + reposition if scroll changed |
//! | Scroll, same window | No | No | reposition cached layout nodes |
//! | Range / content change | Yes (visible only) | Yes (visible only) | O(visible) |
//!
//! Always runs `tree.diff_children` so iced Button state stays valid.

use crate::widget::{column, space};
use crate::{Element, Renderer};
use iced::Length;
use iced::widget;
use iced_core::layout::{Layout, Limits, Node};
use iced_core::widget::tree::{self, Tree};
use iced_core::widget::{Operation, Widget};
use iced_core::{
    Background, Border, Clipboard, Color, Event, Point, Rectangle, Shell, Size, Vector, mouse,
    overlay, renderer, Renderer as _,
};

/// Default extra rows above/below the viewport.
pub const DEFAULT_OVERSCAN: usize = 2;

/// Visible item index range `[start, end)`.
#[must_use]
pub fn visible_range(
    item_count: usize,
    item_height: f32,
    scroll_offset_y: f32,
    viewport_height: f32,
    overscan: usize,
) -> (usize, usize) {
    if item_count == 0 || item_height <= 0.0 {
        return (0, 0);
    }

    let content_height = item_count as f32 * item_height;
    if content_height <= viewport_height {
        return (0, item_count);
    }

    let scroll = scroll_offset_y.max(0.0);
    let mut start = (scroll / item_height).floor() as usize;
    start = start.saturating_sub(overscan);

    let visible = (viewport_height / item_height).ceil() as usize;
    let mut end = (start + visible + overscan * 2).min(item_count);
    if end <= start {
        end = (start + 1).min(item_count);
    }
    (start, end)
}

#[must_use]
pub fn content_height(item_count: usize, item_height: f32) -> f32 {
    item_count as f32 * item_height.max(0.0)
}

#[must_use]
pub fn max_scroll(item_count: usize, item_height: f32, viewport_height: f32) -> f32 {
    (content_height(item_count, item_height) - viewport_height).max(0.0)
}

// --- Simple stateless helper -------------------------------------------------

pub fn content<'a, Message: 'a>(
    item_count: usize,
    item_height: f32,
    scroll_offset_y: f32,
    viewport_height: f32,
    item: impl Fn(usize) -> Element<'a, Message> + 'a,
) -> Element<'a, Message> {
    content_with_overscan(
        item_count,
        item_height,
        scroll_offset_y,
        viewport_height,
        DEFAULT_OVERSCAN,
        item,
    )
}

pub fn content_with_overscan<'a, Message: 'a>(
    item_count: usize,
    item_height: f32,
    scroll_offset_y: f32,
    viewport_height: f32,
    overscan: usize,
    item: impl Fn(usize) -> Element<'a, Message> + 'a,
) -> Element<'a, Message> {
    let item_height = item_height.max(0.0);
    if item_count == 0 || item_height == 0.0 {
        return space::vertical().height(0).into();
    }

    let (start, end) = visible_range(
        item_count,
        item_height,
        scroll_offset_y,
        viewport_height,
        overscan,
    );

    let top = start as f32 * item_height;
    let mid = (end - start) as f32 * item_height;
    let total = content_height(item_count, item_height);
    let bottom = (total - top - mid).max(0.0);

    let mut col = column::with_capacity((end - start) + 2).width(Length::Fill);

    if top > 0.0 {
        col = col.push(space::vertical().height(Length::Fixed(top)));
    }

    for i in start..end {
        col = col.push(
            widget::container(item(i))
                .width(Length::Fill)
                .height(Length::Fixed(item_height)),
        );
    }

    if bottom > 0.0 {
        col = col.push(space::vertical().height(Length::Fixed(bottom)));
    }

    col.into()
}

// --- Stateful viewport virtual list ------------------------------------------

#[must_use]
pub fn virtual_list<'a, Message: 'static>(item_count: usize) -> VirtualList<'a, Message> {
    VirtualList {
        item_count,
        item_height: 48.0,
        scroll_offset_y: 0.0,
        viewport_height: 504.0,
        overscan: DEFAULT_OVERSCAN,
        content_revision: 0,
        focused: None,
        selection_fill: None,
        width: Length::Fill,
        on_scroll: None,
        item: Box::new(|_| space::vertical().height(0).into()),
        range: (0, 0),
        _marker: std::marker::PhantomData,
    }
}

pub struct VirtualList<'a, Message: 'static> {
    item_count: usize,
    item_height: f32,
    scroll_offset_y: f32,
    viewport_height: f32,
    overscan: usize,
    /// Bump when item *data* changes (search results), even if count is same.
    content_revision: u64,
    focused: Option<usize>,
    selection_fill: Option<Color>,
    width: Length,
    on_scroll: Option<Box<dyn Fn(f32) -> Message + 'a>>,
    item: Box<dyn Fn(usize) -> Element<'static, Message> + 'a>,
    range: (usize, usize),
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, Message: 'static> VirtualList<'a, Message> {
    #[must_use]
    pub fn item_height(mut self, height: f32) -> Self {
        self.item_height = height.max(0.0);
        self
    }

    #[must_use]
    pub fn scroll_offset_y(mut self, y: f32) -> Self {
        self.scroll_offset_y = y.max(0.0);
        self
    }

    #[must_use]
    pub fn viewport_height(mut self, height: f32) -> Self {
        self.viewport_height = height.max(0.0);
        self
    }

    #[must_use]
    pub fn overscan(mut self, rows: usize) -> Self {
        self.overscan = rows;
        self
    }

    /// Bump when item *contents* change even if `item_count` is unchanged.
    #[must_use]
    pub fn content_revision(mut self, revision: u64) -> Self {
        self.content_revision = revision;
        self
    }

    #[must_use]
    pub fn focused(mut self, focused: Option<usize>) -> Self {
        self.focused = focused;
        self
    }

    #[must_use]
    pub fn selection_fill(mut self, color: Color) -> Self {
        self.selection_fill = Some(color);
        self
    }

    #[must_use]
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    #[must_use]
    pub fn on_scroll(mut self, f: impl Fn(f32) -> Message + 'a) -> Self {
        self.on_scroll = Some(Box::new(f));
        self
    }

    /// Row body without selection styling (`focused` paints the highlight).
    #[must_use]
    pub fn item(mut self, f: impl Fn(usize) -> Element<'static, Message> + 'a) -> Self {
        self.item = Box::new(f);
        self
    }

    fn compute_range(&self) -> (usize, usize) {
        visible_range(
            self.item_count,
            self.item_height,
            self.scroll_offset_y,
            self.viewport_height,
            self.overscan,
        )
    }

    fn clamp_scroll(&self, y: f32) -> f32 {
        y.clamp(
            0.0,
            max_scroll(self.item_count, self.item_height, self.viewport_height),
        )
    }
}

struct State<Message: 'static> {
    item_count: usize,
    item_height: f32,
    content_revision: u64,
    range: (usize, usize),
    rows: Vec<Element<'static, Message>>,
    /// Nested layout for each row; repositioned on scroll without remeasure.
    child_nodes: Vec<Node>,
    layout_width: f32,
}

impl<Message: 'static> Default for State<Message> {
    fn default() -> Self {
        Self {
            item_count: 0,
            item_height: 0.0,
            content_revision: u64::MAX,
            range: (0, 0),
            rows: Vec::new(),
            child_nodes: Vec::new(),
            layout_width: 0.0,
        }
    }
}

impl<'a, Message: 'static + Clone> VirtualList<'a, Message> {
    fn ensure_rows(&mut self, tree: &mut Tree) {
        let range = self.compute_range();
        self.range = range;

        let (need_rebuild, rows_len) = {
            let state = tree.state.downcast_mut::<State<Message>>();
            let structure = state.item_count != self.item_count
                || (state.item_height - self.item_height).abs() > f32::EPSILON
                || state.content_revision != self.content_revision
                || state.range != range;
            // If tree children were dropped (e.g. parent recreated tree), rebuild.
            let orphaned = !state.rows.is_empty() && tree.children.len() != state.rows.len();
            (structure || orphaned || (state.rows.is_empty() && self.item_count > 0 && range.0 < range.1), state.rows.len())
        };
        let _ = rows_len;

        if need_rebuild {
            let (start, end) = range;
            let mut rows: Vec<Element<'static, Message>> = (start..end)
                .map(|i| {
                    widget::container((self.item)(i))
                        .width(Length::Fill)
                        .height(Length::Fixed(self.item_height))
                        .into()
                })
                .collect();

            tree.diff_children(rows.as_mut_slice());

            let state = tree.state.downcast_mut::<State<Message>>();
            state.item_count = self.item_count;
            state.item_height = self.item_height;
            state.content_revision = self.content_revision;
            state.range = range;
            state.child_nodes.clear();
            state.rows = rows;
        } else {
            // Keep Element widgets (no item() rebuild) but always re-diff Tree.
            // Skipping diff_children caused Button "Downcast on stateless state".
            let mut rows = std::mem::take(&mut tree.state.downcast_mut::<State<Message>>().rows);
            tree.diff_children(rows.as_mut_slice());
            tree.state.downcast_mut::<State<Message>>().rows = rows;
        }
    }
}

impl<'a, Message: 'static + Clone> Widget<Message, crate::Theme, Renderer>
    for VirtualList<'a, Message>
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Message>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Message>::default())
    }

    fn children(&self) -> Vec<Tree> {
        Vec::new()
    }

    fn diff(&mut self, tree: &mut Tree) {
        self.ensure_rows(tree);
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Fixed(self.viewport_height),
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &Limits,
    ) -> Node {
        // Diff may have run already; ensure_rows is cheap when structure is stable
        // (reuses Elements, only re-diffs Tree children).
        self.ensure_rows(tree);

        let max_width = limits.max().width;
        let width = match self.width {
            Length::Fixed(w) => w.min(max_width),
            Length::Fill | Length::FillPortion(_) => max_width,
            Length::Shrink => max_width,
        };
        let viewport_h = self.viewport_height;
        let scroll = self.clamp_scroll(self.scroll_offset_y);
        let (start, _end) = self.range;
        let child_limits = Limits::new(Size::ZERO, Size::new(width, self.item_height));

        // 1.0 px tolerance — autosize can jitter subpixel widths.
        let need_remeasure = {
            let state = tree.state.downcast_ref::<State<Message>>();
            state.child_nodes.len() != state.rows.len()
                || (state.layout_width - width).abs() > 1.0
        };

        if need_remeasure {
            let mut rows = std::mem::take(&mut tree.state.downcast_mut::<State<Message>>().rows);
            let mut child_nodes = Vec::with_capacity(rows.len());
            for (row, child_tree) in rows.iter_mut().zip(tree.children.iter_mut()) {
                child_nodes.push(row.as_widget_mut().layout(child_tree, renderer, &child_limits));
            }
            let state = tree.state.downcast_mut::<State<Message>>();
            state.rows = rows;
            state.child_nodes = child_nodes;
            state.layout_width = width;
        }

        // Reposition only (O(visible)) when structure stable.
        let state = tree.state.downcast_ref::<State<Message>>();
        let mut nodes = Vec::with_capacity(state.child_nodes.len());
        for (j, base) in state.child_nodes.iter().enumerate() {
            let content_y = (start + j) as f32 * self.item_height;
            let y = content_y - scroll;
            nodes.push(base.clone().move_to(Point::new(0.0, y)));
        }

        Node::with_children(Size::new(width, viewport_h), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        operation.container(None, layout.bounds());
        let vo = layout.virtual_offset();
        let mut rows = std::mem::take(&mut tree.state.downcast_mut::<State<Message>>().rows);
        for ((child, child_tree), child_layout) in rows
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            child.as_widget_mut().operate(
                child_tree,
                child_layout.with_virtual_offset(vo),
                renderer,
                operation,
            );
        }
        tree.state.downcast_mut::<State<Message>>().rows = rows;
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        if let Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
            if cursor.is_over(layout.bounds()) {
                if let Some(on_scroll) = &self.on_scroll {
                    let dy = match delta {
                        mouse::ScrollDelta::Lines { y, .. } => -y * self.item_height,
                        mouse::ScrollDelta::Pixels { y, .. } => -y,
                    };
                    let new_y = self.clamp_scroll(self.scroll_offset_y + dy);
                    if (new_y - self.scroll_offset_y).abs() > 0.1 {
                        shell.publish(on_scroll(new_y));
                        shell.capture_event();
                        return;
                    }
                }
            }
        }

        let vo = layout.virtual_offset();
        let mut rows = std::mem::take(&mut tree.state.downcast_mut::<State<Message>>().rows);
        for ((child, child_tree), child_layout) in rows
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                child_tree,
                event,
                child_layout.with_virtual_offset(vo),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
        tree.state.downcast_mut::<State<Message>>().rows = rows;
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State<Message>>();
        let vo = layout.virtual_offset();
        state
            .rows
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, child_tree), child_layout)| {
                child.as_widget().mouse_interaction(
                    child_tree,
                    child_layout.with_virtual_offset(vo),
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
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let clip = viewport.intersection(&bounds).unwrap_or(bounds);

        renderer.with_layer(clip, |renderer| {
            let state = tree.state.downcast_ref::<State<Message>>();
            let (start, end) = state.range;
            let scroll = self.clamp_scroll(self.scroll_offset_y);
            let vo = layout.virtual_offset();

            if let Some(focused) = self.focused {
                if focused >= start && focused < end {
                    let y = bounds.y + focused as f32 * self.item_height - scroll;
                    let sel = Rectangle {
                        x: bounds.x,
                        y,
                        width: bounds.width,
                        height: self.item_height,
                    };
                    if let Some(clipped) = sel.intersection(&clip) {
                        let fill = self.selection_fill.unwrap_or_else(|| {
                            let a = theme.cosmic().accent_color();
                            Color {
                                r: a.red,
                                g: a.green,
                                b: a.blue,
                                a: 0.18,
                            }
                        });
                        renderer.fill_quad(
                            renderer::Quad {
                                bounds: clipped,
                                border: Border {
                                    radius: theme.cosmic().corner_radii.radius_s.into(),
                                    width: 0.0,
                                    color: Color::TRANSPARENT,
                                },
                                shadow: Default::default(),
                                snap: true,
                            },
                            Background::Color(fill),
                        );
                    }
                }
            }

            for ((child, child_tree), child_layout) in state
                .rows
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
            {
                if !child_layout.bounds().intersects(&clip) {
                    continue;
                }
                child.as_widget().draw(
                    child_tree,
                    renderer,
                    theme,
                    style,
                    child_layout.with_virtual_offset(vo),
                    cursor,
                    &clip,
                );
            }

            let content_h = content_height(self.item_count, self.item_height);
            if content_h > self.viewport_height + 1.0 {
                let track = Rectangle {
                    x: bounds.x + bounds.width - 6.0,
                    y: bounds.y,
                    width: 4.0,
                    height: bounds.height,
                };
                let ratio = self.viewport_height / content_h;
                let thumb_h = (track.height * ratio).max(20.0);
                let max_y = (track.height - thumb_h).max(0.0);
                let max_s = max_scroll(self.item_count, self.item_height, self.viewport_height);
                let t = if max_s > 0.0 { scroll / max_s } else { 0.0 };
                let thumb = Rectangle {
                    x: track.x,
                    y: track.y + max_y * t,
                    width: track.width,
                    height: thumb_h,
                };
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: thumb,
                        border: Border {
                            radius: [2.0; 4].into(),
                            width: 0.0,
                            color: Color::TRANSPARENT,
                        },
                        shadow: Default::default(),
                        snap: true,
                    },
                    Background::Color(Color {
                        r: 1.0,
                        g: 1.0,
                        b: 1.0,
                        a: 0.25,
                    }),
                );
            }
        });
    }

    fn overlay<'b>(
        &'b mut self,
        _tree: &'b mut Tree,
        _layout: Layout<'b>,
        _renderer: &Renderer,
        _viewport: &Rectangle,
        _translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, Renderer>> {
        // Row overlays unused by launcher; skip to avoid complex State borrows.
        None
    }
}

impl<'a, Message: 'static + Clone> From<VirtualList<'a, Message>> for Element<'a, Message> {
    fn from(list: VirtualList<'a, Message>) -> Self {
        Element::new(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_range_empty() {
        assert_eq!(visible_range(0, 48.0, 0.0, 500.0, 2), (0, 0));
    }

    #[test]
    fn visible_range_fits() {
        assert_eq!(visible_range(5, 48.0, 0.0, 500.0, 2), (0, 5));
    }

    #[test]
    fn visible_range_scrolled() {
        let (s, e) = visible_range(100, 50.0, 200.0, 100.0, 1);
        assert!(s <= 4);
        assert!(e > s);
        assert!(e <= 100);
    }
}
