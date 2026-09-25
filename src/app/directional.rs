// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use crate::direction::{
    Direction, FocusableArea, SpatialNavigation, ViewContainer, is_directional, is_insider,
};
use iced::widget::selector;
use iced::{Rectangle, Vector, window};
use iced_widget::scrollable::AbsoluteOffset;
use std::collections::HashMap;

/// Space kept between a widget revealed by scrolling and the edge of the
/// scrollable revealing it.
const REVEAL_MARGIN: f32 = 4.0;

#[derive(Debug, Default, PartialEq)]
pub(crate) struct Plan {
    pub(crate) focus: Option<iced::widget::Id>,
    /// The scrolls revealing it, outermost scrollable first.
    pub(crate) scrolls: Vec<(iced::widget::Id, AbsoluteOffset<Option<f32>>)>,
}

pub(crate) fn plan_navigation(
    w_id: window::Id,
    direction: Direction,
    targets: &[(bool, selector::Target, window::Id)],
    surface: Option<Rectangle>,
) -> Plan {
    let scrollables = ScrollTree::new(w_id, targets);

    let Some(origin) = FocusOrigin::find(w_id, targets, &scrollables, surface) else {
        return Plan {
            focus: targets.iter().find_map(|(_, target, window)| match target {
                selector::Target::Focusable { id: Some(id), .. } if window == &w_id => {
                    Some(id.clone())
                }
                _ => None,
            }),
            scrolls: Vec::new(),
        };
    };

    let candidates = candidates(w_id, targets, &scrollables, surface);

    // Search the innermost scrollable first, then outwards, and finally
    // everything else, which includes the widgets of sibling scrollables.
    let mut scopes: Vec<Option<usize>> = scrollables
        .chained(origin.scrollable)
        .into_iter()
        .rev()
        .map(Some)
        .collect();
    scopes.push(None);

    for scope in scopes {
        let pool: Vec<&Candidate> = candidates
            .iter()
            .filter(|candidate| match scope {
                Some(scope) => scrollables.scrollables[scope].contains(candidate.index),
                None => true,
            })
            .collect();

        let visible: Vec<Candidate> = pool
            .iter()
            .copied()
            .filter(|candidate| candidate.visible)
            .cloned()
            .collect();
        if let Some(candidate) = best(direction, origin.decide, &visible) {
            return plan_for(&scrollables, &candidate);
        }

        let reachable: Vec<Candidate> = pool
            .iter()
            .copied()
            .filter(|candidate| {
                !candidate.visible
                    && scrollables
                        .chained(candidate.scrollable)
                        .into_iter()
                        .any(|node| scrollables.scrollables[node].can_scroll(direction))
            })
            .cloned()
            .collect();
        if let Some(candidate) = best(direction, origin.decide, &reachable) {
            return plan_for(&scrollables, &candidate);
        }
    }

    Plan::default()
}

fn plan_for(scroll_tree: &ScrollTree, candidate: &Candidate) -> Plan {
    Plan {
        focus: Some(candidate.id.clone()),
        scrolls: reveal(scroll_tree, candidate),
    }
}

/// Picks the best candidate in `direction`, navigating from `origin`.
fn best(direction: Direction, origin: Rectangle, candidates: &[Candidate]) -> Option<Candidate> {
    // `best_candidate` returns the only candidate without checking the
    // direction, which would move focus the wrong way when a scope holds a
    // single candidate, so the candidates that cannot be navigated to are
    // dropped first.
    let candidates: Vec<Candidate> = candidates
        .iter()
        .filter(|candidate| {
            let bbox = candidate.bbox();
            is_insider(&bbox, &origin, direction) || is_directional(&bbox, &origin, direction)
        })
        .cloned()
        .collect();

    SpatialNavigation::new(
        origin,
        None,
        ViewContainer::new(Rectangle::default(), candidates),
    )
    .navigate_all(direction)
    .cloned()
}

/// The scrolls bringing `candidate` into view, outermost scrollable first.
fn reveal(
    scroll_tree: &ScrollTree,
    candidate: &Candidate,
) -> Vec<(iced::widget::Id, AbsoluteOffset<Option<f32>>)> {
    let mut scrolls = Vec::new();

    for node in scroll_tree.chained(candidate.scrollable) {
        let scrollable = &scroll_tree.scrollables[node];

        let rect = candidate.full + scroll_tree.prefix_translation(Some(node));
        let bounds = scrollable.bounds;

        let offset = AbsoluteOffset {
            x: axis_offset(
                rect.x,
                rect.width,
                bounds.x,
                bounds.width,
                scrollable.translation.x,
                scrollable.content.width,
            ),
            y: axis_offset(
                rect.y,
                rect.height,
                bounds.y,
                bounds.height,
                scrollable.translation.y,
                scrollable.content.height,
            ),
        };

        if offset.x.is_some() || offset.y.is_some() {
            scrolls.push((scrollable.id.clone(), offset));
        }
    }

    scrolls
}

/// The offset bringing `start..start + size` inside a scrollable, clamped to its
/// range.
fn axis_offset(
    start_in_scrollable: f32,
    size: f32,
    bounds_start: f32,
    scrollable_viewport_size: f32,
    translation: f32,
    content_size: f32,
) -> Option<f32> {
    let end = start_in_scrollable + size;
    let visible_start = bounds_start + translation;
    let visible_end = visible_start + scrollable_viewport_size;

    let offset = if start_in_scrollable < visible_start {
        start_in_scrollable - bounds_start - REVEAL_MARGIN
    } else if end > visible_end {
        end - bounds_start - scrollable_viewport_size + REVEAL_MARGIN
    } else {
        return None;
    };

    Some(offset.clamp(0.0, (content_size - scrollable_viewport_size).max(0.0)))
}

struct ScrollTree {
    scrollables: Vec<Scrollable>,
}

impl ScrollTree {
    fn new(w_id: window::Id, targets: &[(bool, selector::Target, window::Id)]) -> Self {
        let mut scrollables: Vec<Scrollable> = Vec::new();
        let mut parent_markers: Vec<Option<usize>> = Vec::new();
        let mut by_marker: HashMap<usize, usize> = HashMap::new();
        // markers of the scrollables whose content has not ended yet.
        let mut open: Vec<(iced::widget::Id, usize)> = Vec::new();

        for (index, (_, target, window)) in targets.iter().enumerate() {
            if window != &w_id {
                continue;
            }

            match target {
                selector::Target::PreOperation { id: Some(id) } => open.push((id.clone(), index)),
                selector::Target::Scrollable {
                    id: Some(id),
                    bounds,
                    content_bounds,
                    translation,
                    ..
                } => {
                    // A scrollable reports itself after its content, and is
                    // marked by a `PreOperation` with the same id before it
                    let Some(position) = open.iter().rposition(|(open_id, _)| open_id == id) else {
                        continue;
                    };
                    let (_, start) = open.remove(position);

                    by_marker.insert(start, scrollables.len());
                    parent_markers.push(position.checked_sub(1).map(|under| open[under].1));
                    scrollables.push(Scrollable {
                        index,
                        start,
                        id: id.clone(),
                        bounds: *bounds,
                        content: *content_bounds,
                        translation: *translation,
                        parent_scrollable_index: None,
                    });
                }
                _ => {}
            }
        }

        for (node, marker) in parent_markers.into_iter().enumerate() {
            let parent = marker.and_then(|marker| by_marker.get(&marker).copied());
            scrollables[node].parent_scrollable_index = parent;
        }

        Self { scrollables }
    }

    /// XXX When scrollables nest, their subtree ranges nest as well, so the one
    /// starting last is the innermost. This relies on implementation of Scrollable operations.
    fn innermost(&self, index: usize) -> Option<usize> {
        self.scrollables
            .iter()
            .enumerate()
            .filter(|(_, scrollable)| scrollable.contains(index))
            .max_by_key(|(_, scrollable)| scrollable.start)
            .map(|(node, _)| node)
    }

    /// The scrollables enclosing `node`, outermost first.
    fn chained(&self, node: Option<usize>) -> Vec<usize> {
        let mut chain = Vec::new();
        let mut current = node;

        while let Some(node) = current {
            chain.push(node);
            current = self.scrollables[node].parent_scrollable_index;
        }

        chain.reverse();
        chain
    }

    /// The scroll offsets applied to the content of `node`.
    fn prefix_translation(&self, node: Option<usize>) -> Vector {
        let mut translation = Vector::ZERO;
        let mut current = node;

        while let Some(node) = current {
            translation += self.scrollables[node].translation;
            current = self.scrollables[node].parent_scrollable_index;
        }

        translation
    }
}

struct Scrollable {
    index: usize,
    start: usize,
    id: iced::widget::Id,
    /// Viewport, in the coordinate space of the parent scrollable.
    bounds: Rectangle,
    content: Rectangle,
    translation: Vector,
    parent_scrollable_index: Option<usize>,
}

impl Scrollable {
    /// Whether the subtree of this scrollable contains the target at `index`.
    fn contains(&self, index: usize) -> bool {
        self.start < index && index < self.index
    }

    /// Whether this scrollable can still scroll towards `direction`.
    fn can_scroll(&self, direction: Direction) -> bool {
        let max = Vector::new(
            (self.content.width - self.bounds.width).max(0.0),
            (self.content.height - self.bounds.height).max(0.0),
        );

        match direction {
            Direction::Left => self.translation.x > 0.0,
            Direction::Right => self.translation.x < max.x,
            Direction::Up => self.translation.y > 0.0,
            Direction::Down => self.translation.y < max.y,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Candidate {
    index: usize,
    id: iced::widget::Id,
    full: Rectangle,
    decide: Rectangle,
    visible: bool,
    scrollable: Option<usize>,
}

impl FocusableArea for Candidate {
    fn bbox(&self) -> Rectangle {
        self.decide
    }

    fn z(&self) -> i32 {
        0
    }
}

struct SurfaceRect {
    /// The full rect, even while it is clipped or off the surface.
    full: Rectangle,
    /// The part of it that is on the surface.
    decide: Rectangle,
    /// Whether any part of it is currently visible.
    visible: bool,
    /// Whether it overlaps the surface at all.
    on_surface: bool,
}

fn surface_rects(
    scroll_tree: &ScrollTree,
    bounds: Rectangle,
    scrollable: Option<usize>,
    surface: Option<Rectangle>,
) -> SurfaceRect {
    let full = bounds - scroll_tree.prefix_translation(scrollable);
    let mut visible = full;
    let mut is_visible = true;

    for node in scroll_tree.chained(scrollable) {
        let scrollable = &scroll_tree.scrollables[node];

        // A scrollable clips its content to its own viewport, which the
        // scrollables enclosing it move around in turn.
        let viewport =
            scrollable.bounds - scroll_tree.prefix_translation(scrollable.parent_scrollable_index);

        match visible.intersection(&viewport) {
            Some(intersection) => visible = intersection,
            None => {
                is_visible = false;
                break;
            }
        }
    }

    let mut on_surface = true;

    if let Some(surface) = surface {
        on_surface = full.intersection(&surface).is_some();

        match visible.intersection(&surface) {
            Some(intersection) => visible = intersection,
            None => is_visible = false,
        }
    }

    SurfaceRect {
        full,
        decide: if is_visible { visible } else { full },
        visible: is_visible,
        on_surface,
    }
}

struct FocusOrigin {
    decide: Rectangle,
    scrollable: Option<usize>,
}

impl FocusOrigin {
    fn find(
        w_id: window::Id,
        targets: &[(bool, selector::Target, window::Id)],
        scroll_tree: &ScrollTree,
        surface: Option<Rectangle>,
    ) -> Option<Self> {
        targets
            .iter()
            .enumerate()
            .find_map(|(index, (is_focused, target, window))| {
                if !is_focused || window != &w_id {
                    return None;
                }

                let selector::Target::Focusable { bounds, .. } = target else {
                    return None;
                };

                let scrollable = scroll_tree.innermost(index);
                let rect = surface_rects(scroll_tree, *bounds, scrollable, surface);

                Some(Self {
                    decide: rect.decide,
                    scrollable,
                })
            })
    }
}

fn candidates(
    w_id: window::Id,
    targets: &[(bool, selector::Target, window::Id)],
    scroll_tree: &ScrollTree,
    surface: Option<Rectangle>,
) -> Vec<Candidate> {
    let mut candidates = Vec::new();

    for (index, (is_focused, target, window)) in targets.iter().enumerate() {
        if *is_focused || window != &w_id {
            continue;
        }

        let selector::Target::Focusable {
            id: Some(id),
            bounds,
            ..
        } = target
        else {
            continue;
        };

        let scrollable = scroll_tree.innermost(index);
        let rect = surface_rects(scroll_tree, *bounds, scrollable, surface);

        if !rect.on_surface {
            continue;
        }

        candidates.push(Candidate {
            index,
            id: id.clone(),
            full: rect.full,
            decide: rect.decide,
            visible: rect.visible,
            scrollable,
        });
    }

    candidates
}
