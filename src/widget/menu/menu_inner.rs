// From iced_aw, license MIT

//! Menu tree overlay
use std::borrow::Cow;

use super::{menu_bar::MenuBarState, menu_tree::MenuTree};
use crate::style::menu_bar::StyleSheet;

use iced::window;
use iced_core::{Border, Renderer as IcedRenderer, Shadow, Widget};
use iced_widget::core::{
    Clipboard, Layout, Length, Padding, Point, Rectangle, Shell, Size, Vector, event,
    layout::{Limits, Node},
    mouse::{self, Cursor},
    overlay, renderer, touch,
    widget::Tree,
};

/// The condition of when to close a menu
#[derive(Debug, Clone, Copy)]
pub struct CloseCondition {
    /// Close menus when the cursor moves outside the check bounds
    pub leave: bool,

    /// Close menus when the cursor clicks outside the check bounds
    pub click_outside: bool,

    /// Close menus when the cursor clicks inside the check bounds
    pub click_inside: bool,
}

/// The width of an item
#[derive(Debug, Clone, Copy)]
pub enum ItemWidth {
    /// Use uniform width
    Uniform(u16),
    /// Static tries to use the width value of each menu(menu tree with children),
    /// the widths of items(menu tree with empty children) will be the same as the menu they're in,
    /// if that value is None,
    /// the default value will be used instead,
    /// which is the value of the Static variant
    Static(u16),
}

/// The height of an item
#[derive(Debug, Clone, Copy)]
pub enum ItemHeight {
    /// Use uniform height.
    Uniform(u16),
    /// Static tries to use `MenuTree.height` as item height,
    /// when it's `None` it'll fallback to the value of the `Static` variant.
    Static(u16),
    /// Dynamic tries to automatically choose the proper item height for you,
    /// but it only works in certain cases:
    ///
    /// - Fixed height
    /// - Shrink height
    /// - Menu tree height
    ///
    /// If none of these is the case, it'll fallback to the value of the `Dynamic` variant.
    Dynamic(u16),
}

/// Methods for drawing path highlight
#[derive(Debug, Clone, Copy)]
pub enum PathHighlight {
    /// Draw the full path,
    Full,
    /// Omit the active item(the last item in the path)
    OmitActive,
    /// Omit the active item if it's not a menu
    MenuActive,
}

/// X+ goes right and Y+ goes down
#[derive(Debug, Clone, Copy)]
pub(crate) enum Direction {
    Positive,
    Negative,
}

/// Adaptive open direction
#[derive(Debug)]
#[allow(clippy::struct_excessive_bools)]
struct Aod {
    // whether or not to use aod
    horizontal: bool,
    vertical: bool,

    // whether or not to use overlap
    horizontal_overlap: bool,
    vertical_overlap: bool,

    // default direction
    horizontal_direction: Direction,
    vertical_direction: Direction,

    // Offset of the child in the default direction
    horizontal_offset: f32,
    vertical_offset: f32,
}
impl Aod {
    /// Returns child position and offset position
    #[allow(clippy::too_many_arguments)]
    fn adaptive(
        parent_pos: f32,
        parent_size: f32,
        child_size: f32,
        max_size: f32,
        offset: f32,
        on: bool,
        overlap: bool,
        direction: Direction,
    ) -> (f32, f32) {
        /*
        Imagine there're two sticks, parent and child
        parent: o-----o
        child:  o----------o

        Now we align the child to the parent in one dimension
        There are 4 possibilities:

        1. to the right
                    o-----oo----------o

        2. to the right but allow overlaping
                    o-----o
                    o----------o

        3. to the left
        o----------oo-----o

        4. to the left but allow overlaping
                    o-----o
               o----------o

        The child goes to the default direction by default,
        if the space on the default direction runs out it goes to the the other,
        whether to use overlap is the caller's decision

        This can be applied to any direction
        */

        match direction {
            Direction::Positive => {
                let space_negative = parent_pos;
                let space_positive = max_size - parent_pos - parent_size;

                if overlap {
                    let overshoot = child_size - parent_size;
                    if on && space_negative > space_positive && overshoot > space_positive {
                        (parent_pos - overshoot, parent_pos - overshoot)
                    } else {
                        (parent_pos, parent_pos)
                    }
                } else {
                    let overshoot = child_size + offset;
                    if on && space_negative > space_positive && overshoot > space_positive {
                        (parent_pos - overshoot, parent_pos - offset)
                    } else {
                        (parent_pos + parent_size + offset, parent_pos + parent_size)
                    }
                }
            }
            Direction::Negative => {
                let space_positive = parent_pos;
                let space_negative = max_size - parent_pos - parent_size;

                if overlap {
                    let overshoot = child_size - parent_size;
                    if on && space_negative > space_positive && overshoot > space_positive {
                        (parent_pos, parent_pos)
                    } else {
                        (parent_pos - overshoot, parent_pos - overshoot)
                    }
                } else {
                    let overshoot = child_size + offset;
                    if on && space_negative > space_positive && overshoot > space_positive {
                        (parent_pos + parent_size + offset, parent_pos + parent_size)
                    } else {
                        (parent_pos - overshoot, parent_pos - offset)
                    }
                }
            }
        }
    }

    /// Returns child position and offset position
    fn resolve(
        &self,
        parent_bounds: Rectangle,
        children_size: Size,
        viewport_size: Size,
    ) -> (Point, Point) {
        let (x, ox) = Self::adaptive(
            parent_bounds.x,
            parent_bounds.width,
            children_size.width,
            viewport_size.width,
            self.horizontal_offset,
            self.horizontal,
            self.horizontal_overlap,
            self.horizontal_direction,
        );
        let (y, oy) = Self::adaptive(
            parent_bounds.y,
            parent_bounds.height,
            children_size.height,
            viewport_size.height,
            self.vertical_offset,
            self.vertical,
            self.vertical_overlap,
            self.vertical_direction,
        );

        ([x, y].into(), [ox, oy].into())
    }
}

/// A part of a menu where items are displayed.
///
/// When the bounds of a menu exceed the viewport,
/// only items inside the viewport will be displayed,
/// when scrolling happens, this should be updated
#[derive(Debug, Clone, Copy)]
pub(super) struct MenuSlice {
    pub(super) start_index: usize,
    pub(super) end_index: usize,
    pub(super) lower_bound_rel: f32,
    pub(super) upper_bound_rel: f32,
}

#[derive(Clone)]
/// Menu bounds in overlay space
struct MenuBounds {
    child_positions: Vec<f32>,
    child_sizes: Vec<Size>,
    children_bounds: Rectangle,
    parent_bounds: Rectangle,
    check_bounds: Rectangle,
    offset_bounds: Rectangle,
}
impl MenuBounds {
    #[allow(clippy::too_many_arguments)]
    fn new<Message>(
        menu_tree: &MenuTree<Message>,
        renderer: &crate::Renderer,
        item_width: ItemWidth,
        item_height: ItemHeight,
        viewport_size: Size,
        overlay_offset: Vector,
        aod: &Aod,
        bounds_expand: u16,
        parent_bounds: Rectangle,
        tree: &mut [Tree],
        is_overlay: bool,
    ) -> Self {
        let (children_size, child_positions, child_sizes) =
            get_children_layout(menu_tree, renderer, item_width, item_height, tree);

        // viewport space parent bounds
        let view_parent_bounds = parent_bounds + overlay_offset;

        // overlay space children position
        let (children_position, offset_position) = {
            let (cp, op) = aod.resolve(view_parent_bounds, children_size, viewport_size);
            if is_overlay {
                (cp - overlay_offset, op - overlay_offset)
            } else {
                (Point::ORIGIN, op - overlay_offset)
            }
        };

        // calc offset bounds
        let delta = children_position - offset_position;
        let offset_size = if delta.x.abs() > delta.y.abs() {
            Size::new(delta.x, children_size.height)
        } else {
            Size::new(children_size.width, delta.y)
        };
        let offset_bounds = Rectangle::new(offset_position, offset_size);

        let children_bounds = Rectangle::new(children_position, children_size);
        let check_bounds = pad_rectangle(children_bounds, bounds_expand.into());

        Self {
            child_positions,
            child_sizes,
            children_bounds,
            parent_bounds,
            check_bounds,
            offset_bounds,
        }
    }
}

#[derive(Clone)]
pub(crate) struct MenuState {
    pub(super) index: Option<usize>,
    scroll_offset: f32,
    menu_bounds: MenuBounds,
}
impl MenuState {
    pub(super) fn layout<Message>(
        &self,
        overlay_offset: Vector,
        slice: MenuSlice,
        renderer: &crate::Renderer,
        menu_tree: &[MenuTree<Message>],
        tree: &mut [Tree],
    ) -> Node {
        let MenuSlice {
            start_index,
            end_index,
            lower_bound_rel,
            upper_bound_rel,
        } = slice;

        // assert_eq!(
        //     menu_tree.children.len(),
        //     self.menu_bounds.child_positions.len()
        // );
        // dbg!(
        //     overlay_offset,
        //     menu_tree.index,
        //     menu_tree.width,
        //     menu_tree.height
        // );
        // for mt in &menu_tree.children[start_index..=end_index] {
        //     dbg!(mt.index);
        // }
        dbg!((start_index, end_index));

        // viewport space children bounds
        let children_bounds = self.menu_bounds.children_bounds + overlay_offset;
        dbg!(children_bounds);
        let child_nodes = self.menu_bounds.child_positions[start_index..=end_index]
            .iter()
            .zip(self.menu_bounds.child_sizes[start_index..=end_index].iter())
            .zip(menu_tree[start_index..=end_index].iter())
            .enumerate()
            .map(|(i, ((cp, size), mt))| {
                let mut position = *cp;
                let mut size = *size;
                dbg!(position, size);

                if position < lower_bound_rel && (position + size.height) > lower_bound_rel {
                    size.height = position + size.height - lower_bound_rel;
                    position = lower_bound_rel;
                } else if position <= upper_bound_rel && (position + size.height) > upper_bound_rel
                {
                    size.height = upper_bound_rel - position;
                }
                // dbg!(size);

                let limits = Limits::new(size, size);
                dbg!(i, mt.index, tree.len());
                for child in &mt.children {
                    dbg!(child.index);
                }

                mt.item
                    .layout(&mut tree[mt.index], renderer, &limits)
                    .move_to(Point::new(0.0, position + self.scroll_offset))
            })
            .collect::<Vec<_>>();

        // dbg!(children_bounds.size());
        Node::with_children(children_bounds.size(), child_nodes).move_to(children_bounds.position())
    }

    fn layout_single<Message>(
        &self,
        overlay_offset: Vector,
        index: usize,
        renderer: &crate::Renderer,
        menu_tree: &MenuTree<Message>,
        tree: &mut Tree,
    ) -> Node {
        // viewport space children bounds
        let children_bounds = self.menu_bounds.children_bounds + overlay_offset;

        let position = self.menu_bounds.child_positions[index];
        let limits = Limits::new(Size::ZERO, self.menu_bounds.child_sizes[index]);
        let parent_offset = children_bounds.position() - Point::ORIGIN;
        let node = menu_tree.item.layout(tree, renderer, &limits);
        node.clone().move_to(Point::new(
            parent_offset.x,
            parent_offset.y + position + self.scroll_offset,
        ))
    }

    pub(super) fn slice(
        &self,
        viewport_size: Size,
        overlay_offset: Vector,
        item_height: ItemHeight,
    ) -> MenuSlice {
        // viewport space children bounds
        let children_bounds = self.menu_bounds.children_bounds + overlay_offset;

        let max_index = self.menu_bounds.child_positions.len().saturating_sub(1);

        // viewport space absolute bounds
        let lower_bound = children_bounds.y.max(0.0);
        let upper_bound = (children_bounds.y + children_bounds.height).min(viewport_size.height);

        // menu space relative bounds
        let lower_bound_rel = lower_bound - (children_bounds.y + self.scroll_offset);
        let upper_bound_rel = upper_bound - (children_bounds.y + self.scroll_offset);

        // index range
        let (start_index, end_index) = match item_height {
            ItemHeight::Uniform(u) => {
                let start_index = (lower_bound_rel / f32::from(u)).floor() as usize;
                let end_index = ((upper_bound_rel / f32::from(u)).floor() as usize).min(max_index);
                (start_index, end_index)
            }
            ItemHeight::Static(_) | ItemHeight::Dynamic(_) => {
                let positions = &self.menu_bounds.child_positions;
                let sizes = &self.menu_bounds.child_sizes;

                let start_index = search_bound(0, 0, max_index, lower_bound_rel, positions, sizes);
                let end_index = search_bound(
                    max_index,
                    start_index,
                    max_index,
                    upper_bound_rel,
                    positions,
                    sizes,
                )
                .min(max_index);

                (start_index, end_index)
            }
        };

        MenuSlice {
            start_index,
            end_index,
            lower_bound_rel,
            upper_bound_rel,
        }
    }
}

pub(crate) struct Menu<'b, Message: std::clone::Clone> {
    pub(crate) tree: MenuBarState,
    pub(crate) menu_roots: Cow<'b, Vec<MenuTree<Message>>>,
    pub(crate) bounds_expand: u16,
    /// Allows menu overlay items to overlap the parent
    pub(crate) menu_overlays_parent: bool,
    pub(crate) close_condition: CloseCondition,
    pub(crate) item_width: ItemWidth,
    pub(crate) item_height: ItemHeight,
    pub(crate) bar_bounds: Rectangle,
    pub(crate) main_offset: i32,
    pub(crate) cross_offset: i32,
    pub(crate) root_bounds_list: Vec<Rectangle>,
    pub(crate) path_highlight: Option<PathHighlight>,
    pub(crate) style: Cow<'b, <crate::Theme as StyleSheet>::Style>,
    pub(crate) position: Point,
    pub(crate) is_overlay: bool,
    /// window id for this popup
    pub(crate) window_id: window::Id,
    pub(crate) depth: usize,
}
impl<'b, Message: Clone + 'static> Menu<'b, Message> {
    pub(crate) fn overlay(self) -> overlay::Element<'b, Message, crate::Theme, crate::Renderer> {
        overlay::Element::new(Box::new(self))
    }
    fn layout(&mut self, renderer: &crate::Renderer, limits: Limits) -> Node {
        // layout children
        let position = self.position;
        let mut intrinsic_size = Size::ZERO;

        self.tree.inner.with_data_mut(|data| {
            let overlay_offset = Point::ORIGIN - position;
            let tree_children = &mut data.tree.children;
            let children = data
                .active_root
                .get(&self.window_id)
                .cloned()
                .map(|active_root| {
                    let (active_tree, roots) = if self.depth > 0 {
                        active_root.iter().skip(1).take(self.depth).fold(
                            (
                                &mut tree_children[active_root[0]].children,
                                &self.menu_roots[active_root[0]].children,
                            ),
                            |(tree, mt), next_active_root| (tree, &mt[*next_active_root].children),
                        )
                    } else {
                        (tree_children, self.menu_roots.as_ref())
                    };

                    dbg!(roots.len());
                    dbg!(&active_root);
                    dbg!(active_tree.len());
                    // dbg!(
                    //     self.window_id,
                    //     limits.max(),
                    //     my_ref_mut.menu_states[&self.window_id].len()
                    // );
                    // dbg!(my_ref_mut.active_root.get(&self.window_id));
                    // for (i, active_tree) in active_tree.iter().enumerate() {
                    //     dbg!(i);
                    //     for (i2, active_tree) in active_tree.children.iter().enumerate() {
                    //         dbg!(i2);
                    //         for (i3, active_tree) in active_tree.children.iter().enumerate() {
                    //             dbg!(i3);
                    //         }
                    //     }
                    // }
                    data.menu_states[&self.window_id]
                        .iter()
                        .enumerate()
                        .filter(|ms| self.is_overlay || ms.0 < active_root.len())
                        .fold((roots, Vec::new()), |(menu_root, mut nodes), (_i, ms)| {
                            dbg!(ms.index);
                            let slice = ms.slice(limits.max(), overlay_offset, self.item_height);
                            let _start_index = slice.start_index;
                            let _end_index = slice.end_index;
                            dbg!(&self.window_id, menu_root.len());
                            let children_node =
                                ms.layout(overlay_offset, slice, renderer, menu_root, active_tree);
                            let node_size = children_node.size();
                            // dbg!(node_size.height);
                            intrinsic_size.height += node_size.height;
                            intrinsic_size.width = intrinsic_size.width.max(node_size.width);
                            nodes.push(children_node);
                            // if popup just use len 1?
                            // only the last menu can have a None active index
                            // dbg!(ms.index);
                            (
                                ms.index
                                    .map_or(menu_root, |active| &menu_root[active].children),
                                nodes,
                            )
                        })
                })
                .map(|(_, l)| l)
                .unwrap_or_default();
            // dbg!(intrinsic_size);
            // overlay space viewport rectangle
            Node::with_children(
                limits.resolve(Length::Shrink, Length::Shrink, intrinsic_size),
                children,
            )
            .translate(Point::ORIGIN - position)
        })
    }

    fn on_event(
        &mut self,
        event: event::Event,
        layout: Layout<'_>,
        view_cursor: Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> (Option<(usize, MenuState)>, event::Status) {
        use event::{
            Event::{Mouse, Touch},
            Status::{Captured, Ignored},
        };
        use mouse::{
            Button::Left,
            Event::{ButtonPressed, ButtonReleased, CursorMoved, WheelScrolled},
        };
        use touch::Event::{FingerLifted, FingerMoved, FingerPressed};

        if !self.tree.inner.with_data(|data| data.open) {
            return (None, Ignored);
        };

        let viewport = layout.bounds();
        let viewport_size = viewport.size();
        let overlay_offset = Point::ORIGIN - viewport.position();
        let overlay_cursor = view_cursor.position().unwrap_or_default() - overlay_offset;
        let menu_roots = match &mut self.menu_roots {
            Cow::Borrowed(_) => panic!(),
            Cow::Owned(o) => o.as_mut_slice(),
        };
        let menu_status = process_menu_events(
            self,
            event.clone(),
            view_cursor,
            renderer,
            clipboard,
            shell,
            overlay_offset,
        );

        init_root_menu(
            self,
            renderer,
            shell,
            overlay_cursor,
            viewport_size,
            overlay_offset,
            self.bar_bounds,
            self.main_offset as f32,
        );

        let ret = match event {
            Mouse(WheelScrolled { delta }) => {
                process_scroll_events(self, delta, overlay_cursor, viewport_size, overlay_offset)
                    .merge(menu_status)
            }

            Mouse(ButtonPressed(Left)) | Touch(FingerPressed { .. }) => {
                self.tree.inner.with_data_mut(|data| {
                    data.pressed = true;
                    data.view_cursor = view_cursor;
                });
                Captured
            }

            Mouse(CursorMoved { position }) | Touch(FingerMoved { position, .. }) => {
                let view_cursor = Cursor::Available(position);
                let overlay_cursor = view_cursor.position().unwrap_or_default() - overlay_offset;
                if !(self.is_overlay || view_cursor.is_over(viewport)) {
                    return (None, menu_status);
                }
                // dbg!(view_cursor, viewport, self.window_id);

                let (new_root, status) = process_overlay_events(
                    self,
                    renderer,
                    viewport_size,
                    overlay_offset,
                    view_cursor,
                    overlay_cursor,
                    self.cross_offset as f32,
                    self.is_overlay,
                );
                return (new_root, status.merge(menu_status));
            }

            Mouse(ButtonReleased(_)) | Touch(FingerLifted { .. }) => {
                self.tree.inner.with_data_mut(|state| {
                    state.pressed = false;

                    // process close condition
                    if state
                        .view_cursor
                        .position()
                        .unwrap_or_default()
                        .distance(view_cursor.position().unwrap_or_default())
                        < 2.0
                    {
                        let is_inside = state.menu_states[&self.window_id]
                            .iter()
                            .any(|ms| ms.menu_bounds.check_bounds.contains(overlay_cursor));

                        if self.close_condition.click_inside
                            && is_inside
                            && matches!(
                                event,
                                Mouse(ButtonReleased(Left)) | Touch(FingerLifted { .. })
                            )
                        {
                            state.reset();
                            return Captured;
                        }

                        if self.close_condition.click_outside && !is_inside {
                            state.reset();
                            return Captured;
                        }
                    }

                    // close all menus when clicking inside the menu bar
                    if self.bar_bounds.contains(overlay_cursor) {
                        state.reset();
                        Captured
                    } else {
                        menu_status
                    }
                })
            }

            _ => menu_status,
        };
        (None, ret)
    }

    #[allow(unused_results)]
    fn draw(
        &self,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        view_cursor: Cursor,
    ) {
        self.tree.inner.with_data(|state| {
            let Some(active_root) = state.active_root.get(&self.window_id) else {
                // dbg!(self.window_id);
                return;
            };
            let viewport = layout.bounds();
            let viewport_size = viewport.size();
            let overlay_offset = Point::ORIGIN - viewport.position();

            let render_bounds = if self.is_overlay {
                Rectangle::new(Point::ORIGIN, viewport.size())
            } else {
                Rectangle::new(Point::ORIGIN, Size::INFINITY)
            };
            // dbg!(self.window_id, &active_root);

            let styling = theme.appearance(&self.style);
            let (active_tree, roots) = if self.depth > 0 {
                active_root.iter().skip(1).take(self.depth).fold(
                    (
                        &state.tree.children[active_root[0]].children,
                        &self.menu_roots[active_root[0]].children,
                    ),
                    |(tree, mt), next_active_root| (tree, &mt[*next_active_root].children),
                )
            } else {
                (&state.tree.children, self.menu_roots.as_ref())
            };
            // let tree = &state.tree.children[active_root].children;
            // let root = &self.menu_roots[active_root];

            let indices = state
                .get_trimmed_indices(&self.window_id)
                .collect::<Vec<_>>();
            // dbg!(
            //     self.window_id,
            //     state.menu_states[&self.window_id].len(),
            //     layout.children().count(),
            //     root.children.len(),
            //     layout.bounds()
            // );
            state.menu_states[&self.window_id]
                .iter()
                .zip(layout.children())
                .enumerate()
                .filter(|ms: &(usize, (&MenuState, Layout<'_>))| {
                    self.is_overlay || ms.0 < active_root.len()
                })
                .fold(roots, |menu_roots, (i, (ms, children_layout))| {
                    let draw_path = self.path_highlight.as_ref().map_or(false, |ph| match ph {
                        PathHighlight::Full => true,
                        PathHighlight::OmitActive => !indices.is_empty() && i < indices.len() - 1,
                        PathHighlight::MenuActive => i < state.menu_states.len() - 1,
                    });

                    // react only to the last menu
                    let view_cursor = if i == state.menu_states.len() - 1 {
                        view_cursor
                    } else {
                        Cursor::Available([-1.0; 2].into())
                    };

                    let draw_menu = |r: &mut crate::Renderer| {
                        // calc slice
                        let slice = ms.slice(viewport_size, overlay_offset, self.item_height);
                        let start_index = slice.start_index;
                        let end_index = slice.end_index;

                        let children_bounds = children_layout.bounds();

                        // draw menu background
                        // let bounds = pad_rectangle(children_bounds, styling.background_expand.into());
                        // println!("cursor: {:?}", view_cursor);
                        // println!("bg_bounds: {:?}", bounds);
                        // println!("color: {:?}\n", styling.background);
                        let menu_quad = renderer::Quad {
                            bounds: pad_rectangle(
                                children_bounds,
                                styling.background_expand.into(),
                            ),
                            border: Border {
                                radius: styling.menu_border_radius.into(),
                                width: styling.border_width,
                                color: styling.border_color,
                            },
                            shadow: Shadow::default(),
                        };
                        let menu_color = styling.background;
                        r.fill_quad(menu_quad, menu_color);

                        // draw path hightlight
                        if let (true, Some(active)) = (draw_path, ms.index) {
                            if let Some(active_layout) = children_layout
                                .children()
                                .nth(active.saturating_sub(start_index))
                            {
                                let path_quad = renderer::Quad {
                                    bounds: active_layout.bounds(),
                                    border: Border {
                                        radius: styling.menu_border_radius.into(),
                                        ..Default::default()
                                    },
                                    shadow: Shadow::default(),
                                };

                                r.fill_quad(path_quad, styling.path);
                            }
                        }
                        if start_index < menu_roots.len() {
                            // draw item
                            menu_roots[start_index..=end_index]
                                .iter()
                                .zip(children_layout.children())
                                .for_each(|(mt, clo)| {
                                    mt.item.draw(
                                        &active_tree[mt.index],
                                        r,
                                        theme,
                                        style,
                                        clo,
                                        view_cursor,
                                        &children_layout.bounds(),
                                    );
                                });
                        }
                    };

                    renderer.with_layer(render_bounds, draw_menu);

                    // only the last menu can have a None active index
                    ms.index
                        .map_or(menu_roots, |active| &menu_roots[active].children)
                });
        })
    }
}
impl<Message: Clone + 'static> overlay::Overlay<Message, crate::Theme, crate::Renderer>
    for Menu<'_, Message>
{
    fn layout(&mut self, renderer: &crate::Renderer, bounds: Size) -> iced_core::layout::Node {
        Menu::layout(
            self,
            renderer,
            Limits::NONE
                .min_width(bounds.width)
                .max_width(bounds.width)
                .min_height(bounds.height)
                .max_height(bounds.height),
        )
    }

    fn on_event(
        &mut self,
        event: iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) -> event::Status {
        self.on_event(event, layout, cursor, renderer, clipboard, shell)
            .1
    }

    fn draw(
        &self,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.draw(renderer, theme, style, layout, cursor);
    }
}

fn pad_rectangle(rect: Rectangle, padding: Padding) -> Rectangle {
    Rectangle {
        x: rect.x - padding.left,
        y: rect.y - padding.top,
        width: rect.width + padding.horizontal(),
        height: rect.height + padding.vertical(),
    }
}

pub(super) fn init_root_menu<Message: Clone>(
    menu: &mut Menu<'_, Message>,
    renderer: &crate::Renderer,
    shell: &mut Shell<'_, Message>,
    overlay_cursor: Point,
    viewport_size: Size,
    overlay_offset: Vector,
    bar_bounds: Rectangle,
    main_offset: f32,
) {
    menu.tree.inner.with_data_mut(|state| {
        if !(state
            .menu_states
            .get(&menu.window_id)
            .is_none_or(|s| s.is_empty())
            && (!menu.is_overlay || bar_bounds.contains(overlay_cursor)))
        {
            return;
        }

        println!(
            "init root menu {} {:?}",
            menu.window_id, menu.root_bounds_list
        );
        dbg!(menu.menu_roots.len());
        // dbg!(state
        //     .menu_states
        //     .get(&menu.window_id)
        //     .is_none_or(|s| s.is_empty()));
        // dbg!(menu
        //     .menu_roots
        //     .as_slice()
        //     .iter()
        //     .map(|r| r.index)
        //     .collect::<Vec<_>>());

        // dbg!(state
        //     .active_root
        //     .get(&menu.window_id)
        //     .into_iter()
        //     .flatten()
        //     .count()
        //     .saturating_sub(1));
        // let root = state
        //     .active_root
        //     .get(&menu.window_id)
        //     .iter()
        //     .map(|l| l.into_iter())
        //     .flatten()
        //     .take(
        //         state
        //             .active_root
        //             .get(&menu.window_id)
        //             .into_iter()
        //             .flatten()
        //             .count()
        //             .saturating_sub(1),
        //     )
        //     .fold(menu.menu_roots.as_slice(), |m, i| {
        //         dbg!(m.len());
        //         m[*i].children.as_slice()
        //     });
        // dbg!(root.len());

        let mut set = false;
        for (i, (&root_bounds, mt)) in menu
            .root_bounds_list
            .iter()
            .zip(menu.menu_roots.iter())
            .enumerate()
        {
            if mt.children.is_empty() {
                // dbg!("skipping menu with no children");
                continue;
            }
            // dbg!(i, root_bounds.contains(overlay_cursor));
            if root_bounds.contains(overlay_cursor) {
                dbg!(root_bounds);
                // dbg!(i, root_bounds, mt.width, mt.height);
                let view_center = viewport_size.width * 0.5;
                let rb_center = root_bounds.center_x();

                state.horizontal_direction = if rb_center > view_center {
                    Direction::Negative
                } else {
                    Direction::Positive
                };

                let aod = Aod {
                    horizontal: true,
                    vertical: true,
                    horizontal_overlap: true,
                    vertical_overlap: false,
                    horizontal_direction: state.horizontal_direction,
                    vertical_direction: state.vertical_direction,
                    horizontal_offset: 0.0,
                    vertical_offset: main_offset,
                };
                let menu_bounds = MenuBounds::new(
                    mt,
                    renderer,
                    menu.item_width,
                    menu.item_height,
                    viewport_size,
                    overlay_offset,
                    &aod,
                    menu.bounds_expand,
                    root_bounds,
                    &mut state.tree.children[0].children,
                    menu.is_overlay,
                );
                set = true;
                // dbg!(i);
                // dbg!("inserting root menu state");
                // dbg!(&menu_bounds.children_bounds);
                // dbg!((&menu_bounds.child_positions));
                state.active_root.insert(menu.window_id, vec![i]);
                // do we need to insert the rest now too?
                let ms = MenuState {
                    index: None,
                    scroll_offset: 0.0,
                    menu_bounds,
                };
                let v = state.menu_states.entry(menu.window_id).or_default();
                v.push(ms);

                // Hack to ensure menu opens properly
                shell.invalidate_layout();

                break;
            }
        }
        if !set {
            dbg!(overlay_cursor);
        }
    });
}

pub(super) fn init_root_popup_menu<Message>(
    menu: &mut Menu<'_, Message>,
    renderer: &crate::Renderer,
    shell: &mut Shell<'_, Message>,
    overlay_cursor: Point,
    viewport_size: Size,
    overlay_offset: Vector,
    bar_bounds: Rectangle,
    main_offset: f32,
) where
    Message: std::clone::Clone,
{
    menu.tree.inner.with_data_mut(|state| {
        if !(state
            .menu_states
            .get(&menu.window_id)
            .is_none_or(|s| s.is_empty())
            && (!menu.is_overlay || bar_bounds.contains(overlay_cursor)))
        {
            return;
        }

        println!(
            "init root menu {} {:?}",
            menu.window_id, menu.root_bounds_list
        );
        // dbg!(state
        //     .menu_states
        //     .get(&menu.window_id)
        //     .is_none_or(|s| s.is_empty()));
        // dbg!(menu
        //     .menu_roots
        //     .as_slice()
        //     .iter()
        //     .map(|r| r.index)
        //     .collect::<Vec<_>>());

        // dbg!(state
        //     .active_root
        //     .get(&menu.window_id)
        //     .into_iter()
        //     .flatten()
        //     .count());
        let active_roots = state
            .active_root
            .get(&menu.window_id)
            .cloned()
            .unwrap_or_default();
        dbg!(&active_roots);
        dbg!(menu.menu_roots.len());
        let root = active_roots
            .iter()
            .take(active_roots.iter().count().saturating_sub(1))
            .fold(menu.menu_roots.as_slice(), |m, i| {
                // dbg!(m.len());
                m[*i].children.as_slice()
            });
        // dbg!(root.len());

        let mut set = false;
        for (i, (&root_bounds, mt)) in menu.root_bounds_list.iter().zip(root.iter()).enumerate() {
            if mt.children.is_empty() {
                // dbg!("skipping menu with no children");
                continue;
            }
            // dbg!(i, root_bounds.contains(overlay_cursor));
            if root_bounds.contains(overlay_cursor) {
                // dbg!(i, root_bounds, mt.width, mt.height);
                let view_center = viewport_size.width * 0.5;
                let rb_center = root_bounds.center_x();

                state.horizontal_direction = if rb_center > view_center {
                    Direction::Negative
                } else {
                    Direction::Positive
                };

                let aod = Aod {
                    horizontal: true,
                    vertical: true,
                    horizontal_overlap: true,
                    vertical_overlap: false,
                    horizontal_direction: state.horizontal_direction,
                    vertical_direction: state.vertical_direction,
                    horizontal_offset: 0.0,
                    vertical_offset: main_offset,
                };

                // dbg!(
                //     state.tree.children.len(),
                //     state.tree.children[0].children.len(),
                // );

                let menu_bounds = MenuBounds::new(
                    mt,
                    renderer,
                    menu.item_width,
                    menu.item_height,
                    viewport_size,
                    overlay_offset,
                    &aod,
                    menu.bounds_expand,
                    root_bounds,
                    // TODO how to select the tree for the popup
                    &mut state.tree.children[0].children,
                    menu.is_overlay,
                );
                set = true;

                let ms = MenuState {
                    index: None,
                    scroll_offset: 0.0,
                    menu_bounds,
                };
                let v = state.menu_states.entry(menu.window_id).or_default();
                v.push(ms);

                // Hack to ensure menu opens properly
                shell.invalidate_layout();

                break;
            }
        }
        if !set {
            dbg!(overlay_cursor);
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn process_menu_events<'b, Message: std::clone::Clone>(
    menu: &'b mut Menu<Message>,
    event: event::Event,
    view_cursor: Cursor,
    renderer: &crate::Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    overlay_offset: Vector,
) -> event::Status {
    use event::Status;

    let window_id = menu.window_id;
    let is_overlay = menu.is_overlay;
    let my_state = &mut menu.tree;
    let menu_roots = match &mut menu.menu_roots {
        Cow::Borrowed(_) => panic!(),
        Cow::Owned(o) => o.as_mut_slice(),
    };
    my_state.inner.with_data_mut(|state| {
        let Some(active_root) = state.active_root.get(&window_id).cloned() else {
            return Status::Ignored;
        };

        let indices = state.get_trimmed_indices(&window_id);

        let indices = if is_overlay {
            indices.collect::<Vec<_>>()
        } else {
            indices.take(1).collect::<Vec<_>>()
        };

        if indices.is_empty() {
            return Status::Ignored;
        }

        // get active item
        // let mt = indices.iter().fold(root | mt, &i | &mut mt.children[i]);
        let (tree, mt) = active_root.iter().take(menu.depth).skip(1).fold(
            (
                &mut state.tree.children[active_root[0]].children,
                &mut menu_roots[active_root[0]],
            ),
            |(tree, mt), next_active_root| (tree, &mut mt.children[*next_active_root]),
        );

        // widget tree
        let tree = &mut tree[mt.index];

        // get layout
        let last_ms = &state.menu_states[&window_id][indices.len() - 1];
        let child_node = last_ms.layout_single(
            overlay_offset,
            last_ms.index.expect("missing index within menu state."),
            renderer,
            mt,
            tree,
        );
        let child_layout = Layout::new(&child_node);

        // process only the last widget
        mt.item.on_event(
            tree,
            event,
            child_layout,
            view_cursor,
            renderer,
            clipboard,
            shell,
            &Rectangle::default(),
        )
    })
}

#[allow(unused_results)]
fn process_overlay_events<Message>(
    menu: &mut Menu<Message>,
    renderer: &crate::Renderer,
    viewport_size: Size,
    overlay_offset: Vector,
    view_cursor: Cursor,
    overlay_cursor: Point,
    cross_offset: f32,
    is_overlay: bool,
) -> (Option<(usize, MenuState)>, event::Status)
where
    Message: std::clone::Clone,
{
    use event::Status::{Captured, Ignored};
    /*
    if no active root || pressed:
        return
    else:
        remove invalid menus // overlay space
        update active item
        if active item is a menu:
            add menu // viewport space
    */
    let mut new_menu_root = None;

    menu.tree.inner.with_data_mut(|state| {
        let Some(active_root) = state.active_root.get(&menu.window_id).clone() else {
            if is_overlay && !menu.bar_bounds.contains(overlay_cursor) {
                state.reset();
            }
            return (new_menu_root, Ignored);
        };

        if state.pressed {
            return (new_menu_root, Ignored);
        }

        /* When overlay is running, cursor_position in any widget method will go negative
        but I still want Widget::draw() to react to cursor movement */
        state.view_cursor = view_cursor;

        // * remove invalid menus
        let mut prev_bounds = std::iter::once(menu.bar_bounds)
            .chain(
                state.menu_states[&menu.window_id]
                    [..state.menu_states.len().saturating_sub(1).min(1)]
                    .iter()
                    .map(|ms| ms.menu_bounds.children_bounds),
            )
            .collect::<Vec<_>>();
        let menu_states = state.menu_states.get_mut(&menu.window_id).unwrap();

        if menu.close_condition.leave {
            for i in (0..menu_states.len()).rev() {
                let mb = &menu_states[i].menu_bounds;

                if mb.parent_bounds.contains(overlay_cursor)
                    || is_overlay && mb.children_bounds.contains(overlay_cursor)
                    || mb.offset_bounds.contains(overlay_cursor)
                    || (mb.check_bounds.contains(overlay_cursor)
                        && prev_bounds.iter().all(|pvb| !pvb.contains(overlay_cursor)))
                {
                    break;
                }
                prev_bounds.pop();
                menu_states.pop();
            }
        } else {
            for i in (0..menu_states.len()).rev() {
                let mb = &menu_states[i].menu_bounds;

                if mb.parent_bounds.contains(overlay_cursor)
                    || mb.children_bounds.contains(overlay_cursor)
                    || prev_bounds.iter().all(|pvb| !pvb.contains(overlay_cursor))
                {
                    break;
                }
                // dbg!(menu_states.len());
                prev_bounds.pop();
                menu_states.pop();
            }
        }

        // get indices
        let indices = menu_states.iter().map(|ms| ms.index).collect::<Vec<_>>();
        let should_add = is_overlay || menu_states.len() < 2;

        // * update active item
        let Some(last_menu_state) = menu_states.last_mut() else {
            // no menus left
            // TODO do we want to avoid this for popups?
            state.active_root.remove(&menu.window_id);

            // keep state.open when the cursor is still inside the menu bar
            // this allows the overlay to keep drawing when the cursor is
            // moving aroung the menu bar
            if !menu.bar_bounds.contains(overlay_cursor) {
                state.open = false;
            }
            return (new_menu_root, Captured);
        };

        let last_menu_bounds = &last_menu_state.menu_bounds;
        let last_parent_bounds = last_menu_bounds.parent_bounds;
        let last_children_bounds = last_menu_bounds.children_bounds;

        if (is_overlay && !menu.menu_overlays_parent && last_parent_bounds.contains(overlay_cursor))
        // cursor is in the parent part
        || is_overlay && !last_children_bounds.contains(overlay_cursor)
        // cursor is outside
        {
            last_menu_state.index = None;
            return (new_menu_root, Captured);
        }
        // cursor is in the children part

        // calc new index
        let height_diff = (overlay_cursor.y
            - (last_children_bounds.y + last_menu_state.scroll_offset))
            .clamp(0.0, last_children_bounds.height - 0.001);
        // dbg!(height_diff);
        // let (tree, active_menu_root) = active_root.iter().skip(1).fold(
        //     (
        //         &mut state.tree.children[active_root[0]].children,
        //         &menu.menu_roots[active_root[0]],
        //     ),
        //     |(tree, mr), next_active_root| (tree, &mr.children[*next_active_root]),
        // );
        let (active_tree, roots) = if menu.depth > 0 {
            active_root.iter().skip(1).take(menu.depth).fold(
                (
                    &mut state.tree.children[active_root[0]].children,
                    &menu.menu_roots[active_root[0]].children,
                ),
                |(tree, mt), next_active_root| (tree, &mt[*next_active_root].children),
            )
        } else {
            (&mut state.tree.children, menu.menu_roots.as_ref())
        };
        let active_menu = if is_overlay {
            indices[0..indices.len().saturating_sub(1)]
                .iter()
                .fold(roots, |mt, i| {
                    &mt[i.expect("missing active child index in menu")].children
                })
        } else {
            // popup does one layer deep
            roots
        };
        // dbg!(active_menu.children.len());
        // dbg!(menu.window_id);
        let new_index = match menu.item_height {
            ItemHeight::Uniform(u) => (height_diff / f32::from(u)).floor() as usize,
            ItemHeight::Static(_) | ItemHeight::Dynamic(_) => {
                let max_index = active_menu.len() - 1;
                search_bound(
                    0,
                    0,
                    max_index,
                    height_diff,
                    &last_menu_bounds.child_positions,
                    &last_menu_bounds.child_sizes,
                )
            }
        };

        let item = &active_menu[new_index];
        // dbg!(new_index);
        // dbg!(item.index);
        // if the index changes, get a new root
        // if !last_menu_state.index.is_some_and(|old| old == new_index) && !item.children.is_empty() {
        //     dbg!(&indices);
        //     dbg!(new_index);
        //     dbg!(item.children.len());
        //     dbg!(menu.window_id);
        //     new_menu_root = Some(new_index);
        // }

        // set new index
        let old_index = last_menu_state.index.replace(new_index);
        // dbg!(should_add);

        // get new active item
        // * add new menu if the new item is a menu
        if !item.children.is_empty() {
            // dbg!("its a menu");
            let item_position = Point::new(
                0.0,
                last_menu_bounds.child_positions[new_index] + last_menu_state.scroll_offset,
            );
            let item_size = last_menu_bounds.child_sizes[new_index];

            // overlay space item bounds
            let item_bounds = Rectangle::new(item_position, item_size)
                + (last_menu_bounds.children_bounds.position() - Point::ORIGIN);

            let aod = Aod {
                horizontal: true,
                vertical: true,
                horizontal_overlap: false,
                vertical_overlap: true,
                horizontal_direction: state.horizontal_direction,
                vertical_direction: state.vertical_direction,
                horizontal_offset: cross_offset,
                vertical_offset: 0.0,
            };
            let ms = MenuState {
                index: None,
                scroll_offset: 0.0,
                menu_bounds: MenuBounds::new(
                    item,
                    renderer,
                    menu.item_width,
                    menu.item_height,
                    viewport_size,
                    overlay_offset,
                    &aod,
                    menu.bounds_expand,
                    item_bounds,
                    active_tree,
                    menu.is_overlay,
                ),
            };
            if !old_index.is_some_and(|old| old == new_index) {
                // dbg!("adding new menu");
                new_menu_root = Some((new_index, ms.clone()));
            }
            if should_add {
                let v = state.menu_states.entry(menu.window_id).or_default();
                v.push(ms);
            }
        }

        (new_menu_root, Captured)
    })
}

fn process_scroll_events<Message>(
    menu: &mut Menu<'_, Message>,
    delta: mouse::ScrollDelta,
    overlay_cursor: Point,
    viewport_size: Size,
    overlay_offset: Vector,
) -> event::Status
where
    Message: Clone,
{
    use event::Status::{Captured, Ignored};
    use mouse::ScrollDelta;

    menu.tree.inner.with_data_mut(|state| {
        let delta_y = match delta {
            ScrollDelta::Lines { y, .. } => y * 60.0,
            ScrollDelta::Pixels { y, .. } => y,
        };

        let calc_offset_bounds = |menu_state: &MenuState, viewport_size: Size| -> (f32, f32) {
            // viewport space children bounds
            let children_bounds = menu_state.menu_bounds.children_bounds + overlay_offset;

            let max_offset = (0.0 - children_bounds.y).max(0.0);
            let min_offset =
                (viewport_size.height - (children_bounds.y + children_bounds.height)).min(0.0);
            (max_offset, min_offset)
        };
        let menu_states = state.menu_states.get_mut(&menu.window_id).unwrap();

        // update
        if menu_states.is_empty() {
            return Ignored;
        } else if menu_states.len() == 1 {
            let last_ms = &mut menu_states[0];

            if last_ms.index.is_none() {
                return Captured;
            }

            let (max_offset, min_offset) = calc_offset_bounds(last_ms, viewport_size);
            last_ms.scroll_offset = (last_ms.scroll_offset + delta_y).clamp(min_offset, max_offset);
        } else {
            // >= 2
            let max_index = menu_states.len() - 1;
            let last_two = &mut menu_states[max_index - 1..=max_index];

            if last_two[1].index.is_some() {
                // scroll the last one
                let (max_offset, min_offset) = calc_offset_bounds(&last_two[1], viewport_size);
                last_two[1].scroll_offset =
                    (last_two[1].scroll_offset + delta_y).clamp(min_offset, max_offset);
            } else {
                if !last_two[0]
                    .menu_bounds
                    .children_bounds
                    .contains(overlay_cursor)
                {
                    return Captured;
                }

                // scroll the second last one
                let (max_offset, min_offset) = calc_offset_bounds(&last_two[0], viewport_size);
                let scroll_offset =
                    (last_two[0].scroll_offset + delta_y).clamp(min_offset, max_offset);
                let clamped_delta_y = scroll_offset - last_two[0].scroll_offset;
                last_two[0].scroll_offset = scroll_offset;

                // update the bounds of the last one
                last_two[1].menu_bounds.parent_bounds.y += clamped_delta_y;
                last_two[1].menu_bounds.children_bounds.y += clamped_delta_y;
                last_two[1].menu_bounds.check_bounds.y += clamped_delta_y;
            }
        }
        Captured
    })
}

#[allow(clippy::pedantic)]
/// Returns (children_size, child_positions, child_sizes)
fn get_children_layout<Message>(
    menu_tree: &MenuTree<Message>,
    renderer: &crate::Renderer,
    item_width: ItemWidth,
    item_height: ItemHeight,
    tree: &mut [Tree],
) -> (Size, Vec<f32>, Vec<Size>) {
    let width = match item_width {
        ItemWidth::Uniform(u) => f32::from(u),
        ItemWidth::Static(s) => f32::from(menu_tree.width.unwrap_or(s)),
    };

    let child_sizes: Vec<Size> = match item_height {
        ItemHeight::Uniform(u) => {
            let count = menu_tree.children.len();
            (0..count).map(|_| Size::new(width, f32::from(u))).collect()
        }
        ItemHeight::Static(s) => menu_tree
            .children
            .iter()
            .map(|mt| Size::new(width, f32::from(mt.height.unwrap_or(s))))
            .collect(),
        ItemHeight::Dynamic(d) => menu_tree
            .children
            .iter()
            .map(|mt| {
                mt.item
                    .element
                    .with_data(|w| match w.as_widget().size().height {
                        Length::Fixed(f) => Size::new(width, f),
                        Length::Shrink => {
                            let l_height = w
                                .as_widget()
                                .layout(
                                    &mut tree[mt.index],
                                    renderer,
                                    &Limits::new(Size::ZERO, Size::new(width, f32::MAX)),
                                )
                                .size()
                                .height;

                            let height = if (f32::MAX - l_height) < 0.001 {
                                f32::from(d)
                            } else {
                                l_height
                            };

                            Size::new(width, height)
                        }
                        _ => mt.height.map_or_else(
                            || Size::new(width, f32::from(d)),
                            |h| Size::new(width, f32::from(h)),
                        ),
                    })
            })
            .collect(),
    };

    let max_index = menu_tree.children.len() - 1;
    let child_positions: Vec<f32> = std::iter::once(0.0)
        .chain(child_sizes[0..max_index].iter().scan(0.0, |acc, x| {
            *acc += x.height;
            Some(*acc)
        }))
        .collect();

    let height = child_sizes.iter().fold(0.0, |acc, x| acc + x.height);

    (Size::new(width, height), child_positions, child_sizes)
}

fn search_bound(
    default: usize,
    default_left: usize,
    default_right: usize,
    bound: f32,
    positions: &[f32],
    sizes: &[Size],
) -> usize {
    // binary search
    let mut left = default_left;
    let mut right = default_right;

    let mut index = default;
    while left != right {
        let m = ((left + right) / 2) + 1;
        if positions[m] > bound {
            right = m - 1;
        } else {
            left = m;
        }
    }
    // let height = f32::from(menu_tree.children[left].height.unwrap_or(default_height));
    let height = sizes[left].height;
    if positions[left] + height > bound {
        index = left;
    }
    index
}
