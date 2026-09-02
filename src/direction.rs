//! Directional navigation with arrow keys inspired by CSS Spatial Navigation.

use float_cmp::approx_eq;
use iced_core::Rectangle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// A focusable element/area in spatial navigation.
pub trait FocusableArea {
    /// Final layout bounds.
    fn bbox(&self) -> Rectangle;

    /// Higher values are treated as on top.
    fn z(&self) -> i32;
}

/// A focusable area.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Candidate {
    pub bbox: Rectangle,
    pub z: i32,
}

impl Candidate {
    pub fn new(bbox: Rectangle) -> Self {
        Self { bbox, z: 0 }
    }
}

impl FocusableArea for Candidate {
    fn bbox(&self) -> Rectangle {
        self.bbox
    }

    fn z(&self) -> i32 {
        self.z
    }
}

#[derive(Debug)]
/// A window container.
///
/// `candidates` listed in depth first order from the tree.
pub struct ViewContainer<C: FocusableArea> {
    pub inside_area: Rectangle,
    pub candidates: Vec<C>,
}

impl<C: FocusableArea> ViewContainer<C> {
    pub fn new(inside_area: Rectangle, candidates: Vec<C>) -> Self {
        Self {
            inside_area,
            candidates,
        }
    }
}

#[derive(Debug)]
pub struct SpatialNavigation<C: FocusableArea> {
    pub search_origin: Rectangle,
    pub container: ViewContainer<C>,
}

impl<C: FocusableArea + PartialEq> SpatialNavigation<C> {
    pub fn new(
        current_focus: Rectangle,
        starting_point: Option<Rectangle>,
        container: ViewContainer<C>,
    ) -> Self {
        let search_origin = choose_search_origin(current_focus, starting_point);
        Self {
            search_origin,
            container,
        }
    }

    pub fn with_origin(search_origin: Rectangle, container: ViewContainer<C>) -> Self {
        Self {
            search_origin,
            container,
        }
    }

    pub fn focusables(&self, visible_only: bool) -> Vec<&C> {
        focusable_elements(&self.container, visible_only)
    }

    pub fn navigate_visible(&self, dir: Direction) -> Option<&C> {
        best_candidate_in_container(dir, self.search_origin, &self.container, true)
    }

    pub fn navigate_all(&self, dir: Direction) -> Option<&C> {
        best_candidate_in_container(dir, self.search_origin, &self.container, false)
    }
}

pub fn choose_search_origin(
    search_origin: Rectangle,
    starting_point: Option<Rectangle>,
) -> Rectangle {
    if let Some(starting_point) = starting_point
        && starting_point_inside(&search_origin, &starting_point)
    {
        return starting_point;
    }

    search_origin
}

/// Returns focusable elements within `container` in depth first order.
///
/// `visible_only = true` filters to candidates whose boundary box is at least partly
/// inside `container.inside_area`.
pub fn focusable_elements<C: FocusableArea>(
    container: &ViewContainer<C>,
    visible_only: bool,
) -> Vec<&C> {
    container
        .candidates
        .iter()
        .filter(|c| {
            !visible_only
                || c.bbox()
                    .intersection(&container.inside_area)
                    .unwrap_or_default()
                    .area()
                    > 0.0
        })
        .collect()
}

/// Selects the best candidate from `candidates` in `dir`, starting from `search_origin`.
///
/// `candidates` must be in DFS order.
pub fn best_candidate<'a, C: FocusableArea + PartialEq, I>(
    dir: Direction,
    search_origin: Rectangle,
    candidates: I,
) -> Option<&'a C>
where
    I: IntoIterator<Item = &'a C>,
{
    let candidate_list: Vec<&'a C> = candidates.into_iter().collect();

    if candidate_list.is_empty() {
        return None;
    }

    if candidate_list.len() == 1 {
        return Some(candidate_list[0]);
    }

    let insiders: Vec<&'a C> = candidate_list
        .iter()
        .copied()
        .filter(|c| is_insider(&c.bbox(), &search_origin, dir))
        .collect();

    if !insiders.is_empty() {
        let min_edge = min_finite(
            insiders
                .iter()
                .map(|c| edge_distance(&c.bbox(), &search_origin, dir)),
        )?;

        let closest: Vec<&'a C> = insiders
            .iter()
            .copied()
            .filter(|c| approx_eq!(f32, edge_distance(&c.bbox(), &search_origin, dir), min_edge))
            .collect();

        if !closest.is_empty() {
            return pick_tied(&closest);
        }

        return Some(insiders[0]);
    }

    let directionals: Vec<&'a C> = candidate_list
        .iter()
        .copied()
        .filter(|c| is_directional(&c.bbox(), &search_origin, dir))
        .collect();

    if directionals.is_empty() {
        return None;
    }

    let min_distance = min_finite(
        directionals
            .iter()
            .map(|c| shortest_distance(dir, &search_origin, &c.bbox())),
    )?;

    let tied: Vec<&'a C> = directionals
        .iter()
        .copied()
        .filter(|c| {
            approx_eq!(
                f32,
                shortest_distance(dir, &search_origin, &c.bbox()),
                min_distance
            )
        })
        .collect();

    if tied.is_empty() {
        return Some(directionals[0]);
    }

    pick_tied(&tied)
}

pub fn best_candidate_in_container<C: FocusableArea + PartialEq>(
    dir: Direction,
    search_origin: Rectangle,
    container: &ViewContainer<C>,
    visible_only: bool,
) -> Option<&C> {
    let focusables = focusable_elements(container, visible_only);
    best_candidate(dir, search_origin, focusables)
}

pub fn shortest_distance(dir: Direction, reference: &Rectangle, candidate: &Rectangle) -> f32 {
    let (rl, rt, rr, rb) = edges(reference);
    let (cl, ct, cr, cb) = edges(candidate);

    let overlap_width = overlap_length(rl, rr, cl, cr);
    let overlap_height = overlap_length(rt, rb, ct, cb);
    let overlap_area = overlap_width * overlap_height;

    let sqrt_overlap = if overlap_area.is_finite() && overlap_area > 0.0 {
        overlap_area.sqrt()
    } else {
        0.0
    };

    match dir {
        Direction::Left | Direction::Right => {
            let primary_gap = axis_gap(rl, rr, cl, cr);
            let orth_gap = axis_gap(rt, rb, ct, cb);

            let projected = overlap_height;
            let ref_dim = rb - rt;

            let euclidean = (primary_gap * primary_gap + orth_gap * orth_gap).sqrt();
            let orth_bias = ref_dim / 2.0;
            let displacement = (orth_gap + orth_bias) * 30.0;

            let align_bias = if ref_dim > 0. {
                projected / ref_dim
            } else {
                0.0
            };
            let alignment = align_bias * 5.0;

            euclidean + displacement - alignment - sqrt_overlap
        }
        Direction::Up | Direction::Down => {
            let primary_gap = axis_gap(rt, rb, ct, cb);
            let orth_gap = axis_gap(rl, rr, cl, cr);

            let projected = overlap_width;
            let ref_dim = rr - rl;

            let euclidean = (primary_gap * primary_gap + orth_gap * orth_gap).sqrt();
            let orth_bias = ref_dim / 2.0;
            let displacement = (orth_gap + orth_bias) * 2.0;

            let align_bias = if ref_dim > 0. {
                projected / ref_dim
            } else {
                0.0
            };
            let alignment = align_bias * 5.0;

            euclidean + displacement - alignment - sqrt_overlap
        }
    }
}

#[inline]
fn min_finite(iter: impl Iterator<Item = f32>) -> Option<f32> {
    let mut best = None;
    for v in iter {
        if v.is_finite() && best.is_none_or(|b| v < b) {
            best = Some(v);
        }
    }
    best
}

/// Returns `(left x, top y, right x, bottom y)`.
fn edges(r: &Rectangle) -> (f32, f32, f32, f32) {
    let w = r.width;
    let h = r.height;

    let (left, right) = if w >= 0.0 {
        (r.x, r.x + w)
    } else {
        (r.x + w, r.x)
    };

    let (top, bottom) = if h >= 0.0 {
        (r.y, r.y + h)
    } else {
        (r.y + h, r.y)
    };

    (left, top, right, bottom)
}

fn starting_point_inside(origin: &Rectangle, sp: &Rectangle) -> bool {
    sp.is_within(origin) || origin.contains(sp.center())
}

fn overlap_length(a1: f32, a2: f32, b1: f32, b2: f32) -> f32 {
    let lo = a1.max(b1);
    let hi = a2.min(b2);
    let v = hi - lo;
    if v.is_finite() && v > 0. { v } else { 0.0 }
}

fn axis_gap(a1: f32, a2: f32, b1: f32, b2: f32) -> f32 {
    let start = a1.max(b1);
    let end = a2.min(b2);

    if end >= start {
        0.0
    } else {
        let g = start - end;
        if g.is_finite() && g > 0.0 { g } else { 0.0 }
    }
}

/// A candidate is an "insider" if it overlaps search_origin, or if it partially overlaps
/// search_origin in a way consistent with the direction.
pub fn is_insider(c: &Rectangle, origin: &Rectangle, dir: Direction) -> bool {
    if c.intersection(origin).unwrap_or_default().area() > 0.0 {
        return true;
    }

    let (cl, ct, cr, cb) = edges(c);
    let (ol, ot, or, ob) = edges(origin);

    match dir {
        Direction::Down => ct >= ot && overlap_length(ol, or, cl, cr) > 0.0,
        Direction::Up => cb <= ob && overlap_length(ol, or, cl, cr) > 0.0,
        Direction::Left => cr <= or && overlap_length(ot, ob, ct, cb) > 0.0,
        Direction::Right => cl >= ol && overlap_length(ot, ob, ct, cb) > 0.0,
    }
}

/// A candidate is in the non-overlapping directional set if it does not overlap search_origin
/// and is in the requested direction.
pub fn is_directional(c: &Rectangle, origin: &Rectangle, dir: Direction) -> bool {
    if c.intersection(origin).unwrap_or_default().area() > 0.0 {
        return false;
    }

    let (cl, ct, cr, cb) = edges(c);
    let (ol, ot, or, ob) = edges(origin);

    match dir {
        Direction::Down => ct >= ob,
        Direction::Up => cb <= ot,
        Direction::Left => cr <= ol,
        Direction::Right => cl >= or,
    }
}

pub fn is_candidate(c: &Rectangle, origin: &Rectangle, dir: Direction) -> bool {
    let (cl, ct, cr, cb) = edges(c);
    let (ol, ot, or, ob) = edges(origin);

    match dir {
        Direction::Down => ct >= ob,
        Direction::Up => cb <= ot,
        Direction::Left => cr <= ol,
        Direction::Right => cl >= or,
    }
}

fn edge_distance(c: &Rectangle, origin: &Rectangle, dir: Direction) -> f32 {
    let (cl, ct, cr, cb) = edges(c);
    let (ol, ot, or, ob) = edges(origin);

    match dir {
        Direction::Down => (ct - ot).abs(),
        Direction::Up => (cb - ob).abs(),
        Direction::Left => (cr - or).abs(),
        Direction::Right => (cl - ol).abs(),
    }
}

/// Tree depth first order + z tie-breaking.
fn pick_tied<'a, C: FocusableArea + PartialEq>(items: &[&'a C]) -> Option<&'a C> {
    if items.is_empty() {
        return None;
    }

    let max_z = items.iter().map(|c| c.z()).max();

    max_z
        .and_then(|z| items.iter().find(|c| c.z() == z))
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    fn candidate(r: Rectangle, z: i32) -> Candidate {
        Candidate { bbox: r, z }
    }

    fn large_container(candidates: Vec<Candidate>) -> ViewContainer<Candidate> {
        ViewContainer::new(rect(-1000.0, -1000.0, 3000.0, 3000.0), candidates)
    }

    #[test]
    fn down_prefers_straight_over_diagonal() {
        let origin = rect(0.0, 0.0, 100.0, 100.0);

        let container = large_container(vec![
            candidate(rect(0.0, 150.0, 100.0, 100.0), 0),
            candidate(rect(100.0, 150.0, 100.0, 100.0), 0),
        ]);

        let best = best_candidate_in_container(Direction::Down, origin, &container, true)
            .expect("expected a candidate");

        assert_eq!(best.bbox.x, 0.0);
    }

    #[test]
    fn right_prefers_straight_over_diagonal() {
        let origin = rect(0.0, 0.0, 100.0, 100.0);

        let container = large_container(vec![
            candidate(rect(150.0, 0.0, 100.0, 100.0), 0),
            candidate(rect(150.0, 100.0, 100.0, 100.0), 0),
        ]);

        let best = best_candidate_in_container(Direction::Right, origin, &container, true)
            .expect("expected a candidate");

        assert_eq!(best.bbox.y, 0.0);
    }

    #[test]
    fn z_breaks_ties_for_overlapping_candidates() {
        let origin = rect(0.0, 0.0, 100.0, 100.0);

        let container = large_container(vec![
            candidate(rect(0.0, 150.0, 100.0, 100.0), 0),
            candidate(rect(0.0, 150.0, 100.0, 100.0), 10),
        ]);

        let best = best_candidate_in_container(Direction::Down, origin, &container, true)
            .expect("expected a candidate");

        assert_eq!(best.z, 10);
    }

    #[test]
    fn visibility_filtering_works() {
        let outside = candidate(rect(200.0, 0.0, 100.0, 100.0), 0);

        let container = ViewContainer::new(rect(0.0, 0.0, 100.0, 100.0), vec![outside]);

        assert!(focusable_elements(&container, true).is_empty());
        assert_eq!(focusable_elements(&container, false).len(), 1);
    }

    #[test]
    fn insider_selection_uses_directional_edge() {
        let origin = rect(0.0, 0.0, 100.0, 100.0);

        let container = large_container(vec![
            candidate(rect(0.0, 50.0, 100.0, 50.0), 0),
            candidate(rect(0.0, 0.0, 100.0, 50.0), 0),
        ]);

        let down = best_candidate_in_container(Direction::Down, origin, &container, true)
            .expect("expected a candidate");
        assert_eq!(down.bbox.y, 0.0);

        let up = best_candidate_in_container(Direction::Up, origin, &container, true)
            .expect("expected a candidate");
        assert_eq!(up.bbox.y, 50.0);
    }

    #[test]
    fn starting_point_inside_is_used_as_search_origin() {
        let origin = rect(0.0, 0.0, 100.0, 100.0);

        let inside = rect(20.0, 20.0, 10.0, 10.0);
        assert_eq!(choose_search_origin(origin, Some(inside)), inside);

        let outside = rect(200.0, 200.0, 10.0, 10.0);
        assert_eq!(choose_search_origin(origin, Some(outside)), origin);
    }
}
