// From iced_aw, license MIT

//! A widget that handles menu trees
use super::{
    menu_inner::{
        CloseCondition, Direction, ItemHeight, ItemWidth, Menu, MenuState, PathHighlight,
    },
    menu_tree::MenuTree,
};
use crate::{
    Renderer,
    style::menu_bar::StyleSheet,
    widget::{
        RcWrapper,
        dropdown::menu::{self, State},
    },
};

use iced::{Point, Vector, window};
use iced_core::Border;
use iced_widget::core::{
    Alignment, Clipboard, Element, Layout, Length, Padding, Rectangle, Shell, Widget, event,
    layout::{Limits, Node},
    mouse::{self, Cursor},
    overlay,
    renderer::{self, Renderer as IcedRenderer},
    touch,
    widget::{Tree, tree},
};

/// A `MenuBar` collects `MenuTree`s and handles all the layout, event processing, and drawing.
pub fn menu_bar<Message>(menu_roots: Vec<MenuTree<Message>>) -> MenuBar<Message>
where
    Message: Clone + 'static,
{
    MenuBar::new(menu_roots)
}

#[derive(Clone, Default)]
pub(crate) struct MenuBarState {
    pub(crate) inner: RcWrapper<MenuBarStateInner>,
}

pub(crate) struct MenuBarStateInner {
    pub(crate) tree: Tree,
    pub(crate) pressed: bool,
    pub(crate) view_cursor: Cursor,
    pub(crate) open: bool,
    pub(crate) active_root: Option<usize>,
    pub(crate) horizontal_direction: Direction,
    pub(crate) vertical_direction: Direction,
    pub(crate) menu_states: Vec<MenuState>,
}
impl MenuBarStateInner {
    pub(super) fn get_trimmed_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.menu_states
            .iter()
            .take_while(|ms| ms.index.is_some())
            .map(|ms| ms.index.expect("No indices were found in the menu state."))
    }

    pub(super) fn reset(&mut self) {
        self.open = false;
        self.active_root = None;
        self.menu_states.clear();
    }
}
impl Default for MenuBarStateInner {
    fn default() -> Self {
        Self {
            tree: Tree::empty(),
            pressed: false,
            view_cursor: Cursor::Available([-0.5, -0.5].into()),
            open: false,
            active_root: None,
            horizontal_direction: Direction::Positive,
            vertical_direction: Direction::Positive,
            menu_states: Vec::new(),
        }
    }
}

pub(crate) fn menu_roots_children<Message>(menu_roots: &Vec<MenuTree<Message>>) -> Vec<Tree>
where
    Message: Clone + 'static,
{
    /*
    menu bar
        menu root 1 (stateless)
            flat tree
        menu root 2 (stateless)
            flat tree
        ...
    */

    menu_roots
        .iter()
        .map(|root| {
            let mut tree = Tree::empty();
            let flat = root
                .flattern()
                .iter()
                .map(|mt| Tree::new(mt.item.clone()))
                .collect();
            tree.children = flat;
            tree
        })
        .collect()
}

#[allow(invalid_reference_casting)]
pub(crate) fn menu_roots_diff<Message>(menu_roots: &mut Vec<MenuTree<Message>>, tree: &mut Tree)
where
    Message: Clone + 'static,
{
    if tree.children.len() > menu_roots.len() {
        tree.children.truncate(menu_roots.len());
    }

    tree.children
        .iter_mut()
        .zip(menu_roots.iter())
        .for_each(|(t, root)| {
            let mut flat = root
                .flattern()
                .iter()
                .map(|mt| {
                    let widget = &mt.item;
                    let widget_ptr = widget as *const dyn Widget<Message, crate::Theme, Renderer>;
                    let widget_ptr_mut =
                        widget_ptr as *mut dyn Widget<Message, crate::Theme, Renderer>;
                    //TODO: find a way to diff_children without unsafe code
                    unsafe { &mut *widget_ptr_mut }
                })
                .collect::<Vec<_>>();

            t.diff_children(flat.as_mut_slice());
        });

    if tree.children.len() < menu_roots.len() {
        let extended = menu_roots[tree.children.len()..].iter().map(|root| {
            let mut tree = Tree::empty();
            let flat = root
                .flattern()
                .iter()
                .map(|mt| Tree::new(mt.item.clone()))
                .collect();
            tree.children = flat;
            tree
        });
        tree.children.extend(extended);
    }
}

/// A `MenuBar` collects `MenuTree`s and handles all the layout, event processing, and drawing.
#[allow(missing_debug_implementations)]
pub struct MenuBar<Message> {
    width: Length,
    height: Length,
    spacing: f32,
    padding: Padding,
    bounds_expand: u16,
    main_offset: i32,
    cross_offset: i32,
    close_condition: CloseCondition,
    item_width: ItemWidth,
    item_height: ItemHeight,
    path_highlight: Option<PathHighlight>,
    menu_roots: Vec<MenuTree<Message>>,
    style: <crate::Theme as StyleSheet>::Style,
    window_id: window::Id,
    #[cfg(feature = "wayland")]
    positioner: iced_runtime::platform_specific::wayland::popup::SctkPositioner,
}

impl<Message> MenuBar<Message>
where
    Message: Clone + 'static,
{
    /// Creates a new [`MenuBar`] with the given menu roots
    #[must_use]
    pub fn new(menu_roots: Vec<MenuTree<Message>>) -> Self {
        let mut menu_roots = menu_roots;
        menu_roots.iter_mut().for_each(MenuTree::set_index);

        Self {
            width: Length::Shrink,
            height: Length::Shrink,
            spacing: 0.0,
            padding: Padding::ZERO,
            bounds_expand: 16,
            main_offset: 0,
            cross_offset: 0,
            close_condition: CloseCondition {
                leave: false,
                click_outside: true,
                click_inside: true,
            },
            item_width: ItemWidth::Uniform(150),
            item_height: ItemHeight::Uniform(30),
            path_highlight: Some(PathHighlight::MenuActive),
            menu_roots,
            style: <crate::Theme as StyleSheet>::Style::default(),
            window_id: window::Id::NONE,
            #[cfg(feature = "wayland")]
            positioner: iced_runtime::platform_specific::wayland::popup::SctkPositioner::default(),
        }
    }

    /// Sets the expand value for each menu's check bounds
    ///
    /// When the cursor goes outside of a menu's check bounds,
    /// the menu will be closed automatically, this value expands
    /// the check bounds
    #[must_use]
    pub fn bounds_expand(mut self, value: u16) -> Self {
        self.bounds_expand = value;
        self
    }

    /// [`CloseCondition`]
    #[must_use]
    pub fn close_condition(mut self, close_condition: CloseCondition) -> Self {
        self.close_condition = close_condition;
        self
    }

    /// Moves each menu in the horizontal open direction
    #[must_use]
    pub fn cross_offset(mut self, value: i32) -> Self {
        self.cross_offset = value;
        self
    }

    /// Sets the height of the [`MenuBar`]
    #[must_use]
    pub fn height(mut self, height: Length) -> Self {
        self.height = height;
        self
    }

    /// [`ItemHeight`]
    #[must_use]
    pub fn item_height(mut self, item_height: ItemHeight) -> Self {
        self.item_height = item_height;
        self
    }

    /// [`ItemWidth`]
    #[must_use]
    pub fn item_width(mut self, item_width: ItemWidth) -> Self {
        self.item_width = item_width;
        self
    }

    /// Moves all the menus in the vertical open direction
    #[must_use]
    pub fn main_offset(mut self, value: i32) -> Self {
        self.main_offset = value;
        self
    }

    /// Sets the [`Padding`] of the [`MenuBar`]
    #[must_use]
    pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the method for drawing path highlight
    #[must_use]
    pub fn path_highlight(mut self, path_highlight: Option<PathHighlight>) -> Self {
        self.path_highlight = path_highlight;
        self
    }

    /// Sets the spacing between menu roots
    #[must_use]
    pub fn spacing(mut self, units: f32) -> Self {
        self.spacing = units;
        self
    }

    /// Sets the style of the menu bar and its menus
    #[must_use]
    pub fn style(mut self, style: impl Into<<crate::Theme as StyleSheet>::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the width of the [`MenuBar`]
    #[must_use]
    pub fn width(mut self, width: Length) -> Self {
        self.width = width;
        self
    }

    #[cfg(feature = "wayland")]
    pub fn with_positioner(
        mut self,
        positioner: iced_runtime::platform_specific::wayland::popup::SctkPositioner,
    ) -> Self {
        self.positioner = positioner;
        self
    }

    pub fn window_id(mut self, id: window::Id) -> Self {
        self.window_id = id;
        self
    }

    pub fn window_id_maybe(mut self, id: Option<window::Id>) -> Self {
        if let Some(id) = id {
            self.window_id = id;
        }
        self
    }
}
impl<Message> Widget<Message, crate::Theme, Renderer> for MenuBar<Message>
where
    Message: Clone + 'static,
{
    fn size(&self) -> iced_core::Size<Length> {
        iced_core::Size::new(self.width, self.height)
    }

    fn diff(&mut self, tree: &mut Tree) {
        menu_roots_diff(&mut self.menu_roots, tree);
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<MenuBarState>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(MenuBarState::default())
    }

    fn children(&self) -> Vec<Tree> {
        menu_roots_children(&self.menu_roots)
    }

    fn layout(&self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        use super::flex;

        let limits = limits.width(self.width).height(self.height);
        let children = self
            .menu_roots
            .iter()
            .map(|root| &root.item)
            .collect::<Vec<_>>();
        // the first children of the tree are the menu roots items
        let mut tree_children = tree
            .children
            .iter_mut()
            .map(|t| &mut t.children[0])
            .collect::<Vec<_>>();
        flex::resolve_wrapper(
            &flex::Axis::Horizontal,
            renderer,
            &limits,
            self.padding,
            self.spacing,
            Alignment::Center,
            &children,
            &mut tree_children,
        )
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: event::Event,
        layout: Layout<'_>,
        view_cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        use event::Event::{Mouse, Touch};
        use mouse::{Button::Left, Event::ButtonReleased};
        use touch::Event::{FingerLifted, FingerLost};

        let root_status = process_root_events(
            &mut self.menu_roots,
            view_cursor,
            tree,
            &event,
            layout,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        let state = tree.state.downcast_mut::<MenuBarState>();

        match event {
            Mouse(ButtonReleased(Left)) | Touch(FingerLifted { .. } | FingerLost { .. }) => {
                state.inner.with_data_mut(|state| {
                    if state.menu_states.is_empty() && view_cursor.is_over(layout.bounds()) {
                        state.view_cursor = view_cursor;
                        state.open = true;
                        // #[cfg(feature = "wayland")]
                        // TODO emit Message to open menu
                    }
                });
            }
            _ => (),
        }
        root_status
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        view_cursor: Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<MenuBarState>();
        let cursor_pos = view_cursor.position().unwrap_or_default();
        state.inner.with_data_mut(|state| {
            let position = if state.open && (cursor_pos.x < 0.0 || cursor_pos.y < 0.0) {
                state.view_cursor
            } else {
                view_cursor
            };

            // draw path highlight
            if self.path_highlight.is_some() {
                let styling = theme.appearance(&self.style);
                if let Some(active) = state.active_root {
                    let active_bounds = layout
                        .children()
                        .nth(active)
                        .expect("Active child not found in menu?")
                        .bounds();
                    let path_quad = renderer::Quad {
                        bounds: active_bounds,
                        border: Border {
                            radius: styling.bar_border_radius.into(),
                            ..Default::default()
                        },
                        shadow: Default::default(),
                    };

                    renderer.fill_quad(path_quad, styling.path);
                }
            }

            self.menu_roots
                .iter()
                .zip(&tree.children)
                .zip(layout.children())
                .for_each(|((root, t), lo)| {
                    root.item.draw(
                        &t.children[root.index],
                        renderer,
                        theme,
                        style,
                        lo,
                        position,
                        viewport,
                    );
                });
        });
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, Renderer>> {
        // #[cfg(feature = "wayland")]
        // return None;

        let state = tree.state.downcast_ref::<MenuBarState>();
        if state.inner.with_data_mut(|state| !state.open) {
            return None;
        };

        Some(
            Menu {
                tree: state.clone(),
                menu_roots: std::borrow::Cow::Borrowed(&mut self.menu_roots),
                bounds_expand: self.bounds_expand,
                menu_overlays_parent: false,
                close_condition: self.close_condition,
                item_width: self.item_width,
                item_height: self.item_height,
                bar_bounds: layout.bounds(),
                main_offset: self.main_offset,
                cross_offset: self.cross_offset,
                root_bounds_list: layout.children().map(|lo| lo.bounds()).collect(),
                path_highlight: self.path_highlight,
                style: std::borrow::Cow::Borrowed(&self.style),
                position: Point::new(translation.x, translation.y),
            }
            .overlay(),
        )
    }
}

impl<'a, Message> From<MenuBar<Message>> for Element<'a, Message, crate::Theme, Renderer>
where
    Message: Clone + 'static,
{
    fn from(value: MenuBar<Message>) -> Self {
        Self::new(value)
    }
}

#[allow(unused_results, clippy::too_many_arguments)]
fn process_root_events<Message>(
    menu_roots: &mut [MenuTree<Message>],
    view_cursor: Cursor,
    tree: &mut Tree,
    event: &event::Event,
    layout: Layout<'_>,
    renderer: &Renderer,
    clipboard: &mut dyn Clipboard,
    shell: &mut Shell<'_, Message>,
    viewport: &Rectangle,
) -> event::Status
where
{
    menu_roots
        .iter_mut()
        .zip(&mut tree.children)
        .zip(layout.children())
        .map(|((root, t), lo)| {
            // assert!(t.tag == tree::Tag::stateless());
            root.item.on_event(
                &mut t.children[root.index],
                event.clone(),
                lo,
                view_cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            )
        })
        .fold(event::Status::Ignored, event::Status::merge)
}
