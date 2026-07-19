// Copyright 2026 System76 / local COSMIC performance work
// SPDX-License-Identifier: MPL-2.0

//! Virtualized vertical lists — browser-style recycling for iced/COSMIC.
//!
//! # Why this exists
//!
//! In the browser, a list of ~100 rows stays as DOM nodes; changing the
//! selected row only updates classes / `scrollTop`. In iced, `view()` builds a
//! new `Element` tree every frame. If each keypress rebuilds and re-lays-out
//! every row, navigation feels laggy even for 20 items.
//!
//! This widget:
//! 1. Only **creates** row widgets for the visible window (+ overscan).
//! 2. **Reuses** those row widgets when only the selection index changes
//!    (no child rebuild, cached layout).
//! 3. Keeps full content height so proportional `snap_to` / scrollbars work.
//!
//! # Pattern
//!
//! ```ignore
//! scrollable(
//!     virtual_list(items.len())
//!         .item_height(48.0)
//!         .viewport_height(504.0)
//!         .scroll_offset_y(scroll_y)
//!         .focused(Some(focused))
//!         .item(|i| row_ui(i)) // must not depend on selection styling
//! )
//! .id(LIST_ID)
//! .on_scroll(|vp| Message::Scrolled(vp.absolute_offset().y))
//! ```
//!
//! Keep `scroll_offset_y` in app state (from `on_scroll` and when issuing
//! `snap_to` / `scroll_to`). Put **selection chrome** via `.focused(Some(i))`
//! so row bodies stay cacheable.

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

/// Visible item index range `[start, end)` for a vertical virtual list.
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

/// Total content height for `item_count` fixed-height rows.
#[must_use]
pub fn content_height(item_count: usize, item_height: f32) -> f32 {
    item_count as f32 * item_height.max(0.0)
}

// --- Stateless helper (always rebuilds; fine for tiny lists / tests) ---------

/// Build a virtualized column (spacers + visible rows). Prefer [`VirtualList`]
/// when selection changes often — it reuses row widgets across frames.
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

/// Same as [`content`] with a custom overscan row count.
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

/// Convenience scrollable wrapper around [`content`].
pub fn scrollable<'a, Message: 'a>(
    item_count: usize,
    item_height: f32,
    scroll_offset_y: f32,
    viewport_height: f32,
    item: impl Fn(usize) -> Element<'a, Message> + 'a,
) -> widget::Scrollable<'a, Message, crate::Theme, Renderer> {
    let body = content(
        item_count,
        item_height,
        scroll_offset_y,
        viewport_height,
        item,
    );
    crate::widget::scrollable(body).height(Length::Fixed(viewport_height))
}

// --- Stateful virtual list (cache rows across selection-only updates) --------

/// Create a [`VirtualList`] with `item_count` fixed-height rows.
#[must_use]
pub fn virtual_list<'a, Message: 'static>(
    item_count: usize,
) -> VirtualList<'a, Message> {
    VirtualList {
        item_count,
        item_height: 48.0,
        scroll_offset_y: 0.0,
        viewport_height: 504.0,
        overscan: DEFAULT_OVERSCAN,
        focused: None,
        selection_fill: None,
        width: Length::Fill,
        item: Box::new(|_| space::vertical().height(0).into()),
        _marker: std::marker::PhantomData,
    }
}

/// A virtualized list that **reuses row widgets** when only the focused index
/// changes (browser-like selection updates).
pub struct VirtualList<'a, Message: 'static> {
    item_count: usize,
    item_height: f32,
    scroll_offset_y: f32,
    viewport_height: f32,
    overscan: usize,
    focused: Option<usize>,
    /// Optional selection background (RGBA). Defaults to a soft accent-like fill.
    selection_fill: Option<Color>,
    width: Length,
    item: Box<dyn Fn(usize) -> Element<'static, Message> + 'a>,
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

    /// Keyboard / app selection index. Drawn as a background behind that row;
    /// keep this **out** of [`Self::item`] so rows stay cacheable.
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

    /// Row builder. Must produce `'static` elements (own your strings/handles).
    /// **Do not** bake selection into the row — use [`Self::focused`].
    #[must_use]
    pub fn item(
        mut self,
        f: impl Fn(usize) -> Element<'static, Message> + 'a,
    ) -> Self {
        self.item = Box::new(f);
        self
    }

    fn range(&self) -> (usize, usize) {
        visible_range(
            self.item_count,
            self.item_height,
            self.scroll_offset_y,
            self.viewport_height,
            self.overscan,
        )
    }
}

struct State<Message: 'static> {
    item_count: usize,
    item_height: f32,
    range: (usize, usize),
    /// Owned row elements for the current range (reused across selection-only frames).
    rows: Vec<Element<'static, Message>>,
    layout_cache: Option<Node>,
    layout_width: f32,
}

impl<Message: 'static> Default for State<Message> {
    fn default() -> Self {
        Self {
            item_count: 0,
            item_height: 0.0,
            range: (0, 0),
            rows: Vec::new(),
            layout_cache: None,
            layout_width: 0.0,
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
        // Initial mount: empty until layout/diff fills state.rows
        Vec::new()
    }

    fn diff(&mut self, tree: &mut Tree) {
        let range = self.range();
        let state = tree.state.downcast_mut::<State<Message>>();

        let structure_changed = state.item_count != self.item_count
            || (state.item_height - self.item_height).abs() > f32::EPSILON
            || state.range != range;

        if structure_changed {
            state.item_count = self.item_count;
            state.item_height = self.item_height;
            state.range = range;
            state.layout_cache = None;

            let (start, end) = range;
            let mut rows: Vec<Element<'static, Message>> = (start..end)
                .map(|i| {
                    widget::container((self.item)(i))
                        .width(Length::Fill)
                        .height(Length::Fixed(self.item_height))
                        .into()
                })
                .collect();

            // Drop borrow of state before diff_children.
            let _ = state;
            tree.diff_children(rows.as_mut_slice());
            tree.state.downcast_mut::<State<Message>>().rows = rows;
        } else {
            // Selection-only (or pure redraw): keep row widgets + trees.
            let mut rows = std::mem::take(&mut state.rows);
            let _ = state;
            tree.diff_children(rows.as_mut_slice());
            tree.state.downcast_mut::<State<Message>>().rows = rows;
        }
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Fixed(content_height(self.item_count, self.item_height)),
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &Limits,
    ) -> Node {
        // Ensure rows exist (first frame / structure change).
        self.diff(tree);

        let state = tree.state.downcast_mut::<State<Message>>();
        let max_width = limits.max().width;
        let content_h = content_height(self.item_count, self.item_height);
        let width = match self.width {
            Length::Fixed(w) => w.min(max_width),
            Length::Fill | Length::FillPortion(_) => max_width,
            Length::Shrink => max_width,
        };

        // Layout cache: when only selection changed, reuse geometry.
        if let Some(cache) = &state.layout_cache {
            if (state.layout_width - width).abs() < 0.5 {
                return cache.clone();
            }
        }

        let (start, end) = state.range;
        let top = start as f32 * self.item_height;

        let child_limits = Limits::new(Size::ZERO, Size::new(width, self.item_height));

        let mut nodes = Vec::with_capacity(state.rows.len());
        for (row, child_tree) in state.rows.iter_mut().zip(tree.children.iter_mut()) {
            let mut node = row
                .as_widget_mut()
                .layout(child_tree, renderer, &child_limits);
            let y = top + nodes.len() as f32 * self.item_height;
            node = node.move_to(Point::new(0.0, y));
            nodes.push(node);
        }

        // Full content height (spacers are implicit empty space for hit-testing).
        let root = Node::with_children(Size::new(width, content_h), nodes);
        // Note: top/bottom gaps have no children — scrollable still uses content size.

        let _ = end; // range end used via state.rows len
        state.layout_width = width;
        state.layout_cache = Some(root.clone());
        root
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation<()>,
    ) {
        operation.container(None, layout.bounds());
        let virtual_offset = layout.virtual_offset();
        let state = tree.state.downcast_mut::<State<Message>>();
        for ((child, child_tree), child_layout) in state
            .rows
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            child.as_widget_mut().operate(
                child_tree,
                child_layout.with_virtual_offset(virtual_offset),
                renderer,
                operation,
            );
        }
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
        let state = tree.state.downcast_mut::<State<Message>>();
        for ((child, child_tree), child_layout) in state
            .rows
            .iter_mut()
            .zip(tree.children.iter_mut())
            .zip(layout.children())
        {
            child.as_widget_mut().update(
                child_tree,
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
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State<Message>>();
        state
            .rows
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, child_tree), child_layout)| {
                child.as_widget().mouse_interaction(
                    child_tree,
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
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Message>>();
        let (start, _) = state.range;
        let bounds = layout.bounds();

        // Selection background (does not require rebuilding row widgets).
        if let Some(focused) = self.focused {
            if focused >= start && focused < state.range.1 {
                let y = bounds.y + focused as f32 * self.item_height;
                let sel = Rectangle {
                    x: bounds.x,
                    y,
                    width: bounds.width,
                    height: self.item_height,
                };
                if let Some(clipped) = sel.intersection(viewport) {
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
            let child_bounds = child_layout.bounds();
            if !child_bounds.intersects(viewport) {
                continue;
            }
            child.as_widget().draw(
                child_tree,
                renderer,
                theme,
                style,
                child_layout.with_virtual_offset(layout.virtual_offset()),
                cursor,
                viewport,
            );
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
        let state = tree.state.downcast_mut::<State<Message>>();
        // No nested overlays aggregation for simplicity.
        let _ = (state, layout, renderer, viewport, translation);
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
    use super::{content_height, visible_range};

    #[test]
    fn visible_range_full_when_content_fits() {
        assert_eq!(visible_range(10, 40.0, 0.0, 500.0, 2), (0, 10));
    }

    #[test]
    fn visible_range_window_for_long_list() {
        let (s, e) = visible_range(100, 50.0, 0.0, 200.0, 2);
        assert_eq!(s, 0);
        assert!(e <= 10);
        assert!(e - s < 100);
    }

    #[test]
    fn visible_range_scrolls() {
        let (s, e) = visible_range(100, 50.0, 500.0, 200.0, 2);
        assert!(s >= 8);
        assert!(e > s);
        assert!(e <= 100);
    }

    #[test]
    fn content_height_scales() {
        assert_eq!(content_height(20, 49.0), 980.0);
    }
}
