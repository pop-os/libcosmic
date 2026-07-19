// Copyright 2026 System76 / local COSMIC performance work
// SPDX-License-Identifier: MPL-2.0

//! Virtualized vertical lists for large item counts.
//!
//! Immediate-mode UIs (iced) rebuild widget trees every frame. Materializing
//! hundreds of complex rows (buttons, icons, text) inside a [`scrollable`] makes
//! keyboard navigation and scrolling lag. Browsers avoid this with virtual
//! lists: only rows near the viewport exist as real widgets; spacers preserve
//! total content height so scrollbar math stays correct.
//!
//! # Pattern
//!
//! 1. Keep scroll offset in app state (`on_scroll` and/or before `snap_to`).
//! 2. Build content with [`content`] / [`content_with_overscan`].
//! 3. Wrap in [`crate::widget::scrollable`].
//!
//! For short lists that fit the viewport, all items are built (no spacers).

use crate::widget::{column, space};
use crate::{Element, Renderer};
use iced::Length;
use iced::widget;

/// Default extra rows above/below the viewport to reduce pop-in while scrolling.
pub const DEFAULT_OVERSCAN: usize = 2;

/// Visible item index range `[start, end)` for a vertical virtual list.
///
/// `end` is exclusive. When the list fits in the viewport, returns `(0, item_count)`.
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

/// Build a virtualized column: top spacer + visible rows + bottom spacer.
///
/// Put the result inside a [`scrollable`](crate::widget::scrollable). Keep
/// `scroll_offset_y` in sync with the scrollable (via `on_scroll` and when
/// issuing `snap_to` / `scroll_to` from the app).
///
/// `item_height` must match the laid-out height of each row (including any
/// spacing you bake into the row). Variable-height rows are not supported in v1.
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

    // Capacity: optional spacers + visible items
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

/// Convenience: virtual content inside a themed vertical scrollable.
///
/// The returned scrollable does **not** auto-track offset; use
/// [`.on_scroll(...)`](iced::widget::Scrollable::on_scroll) on the result (or
/// set scroll state when snapping) so the next `view` passes the right
/// `scroll_offset_y` into this builder.
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
        // 200/50 = 4 visible + 2*2 overscan = 8
        assert!(e <= 10);
        assert!(e - s < 100);
    }

    #[test]
    fn visible_range_scrolls() {
        let (s, e) = visible_range(100, 50.0, 500.0, 200.0, 2);
        assert!(s >= 8); // 500/50 = 10, minus overscan 2
        assert!(e > s);
        assert!(e <= 100);
    }

    #[test]
    fn content_height_scales() {
        assert_eq!(content_height(20, 49.0), 980.0);
    }
}
