// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A context menu is a menu in a graphical user interface that appears upon user interaction, such as a right-click mouse operation.

#[cfg(all(feature = "wayland", feature = "winit", feature = "surface-message"))]
use crate::app::cosmic::{WINDOWING_SYSTEM, WindowingSystem};
use crate::widget::menu::{
    self, CloseCondition, Direction, ItemHeight, ItemWidth, MenuBarState, PathHighlight,
    init_root_menu, menu_roots_diff,
};
use derive_setters::Setters;
use iced::touch::Finger;
use iced::{Event, Vector, keyboard, window};
use iced_core::widget::{Tree, Widget, tree};
use iced_core::{Length, Point, Size, event, mouse, touch};
use std::collections::HashSet;
use std::sync::Arc;

/// A context menu is a menu in a graphical user interface that appears upon user interaction, such as a right-click mouse operation.
pub fn context_menu<'a, Message: 'static + Clone>(
    content: impl Into<crate::Element<'a, Message>>,
    // on_context: Message,
    context_menu: Option<Vec<menu::Tree<Message>>>,
) -> ContextMenu<'a, Message> {
    let mut this = ContextMenu {
        content: content.into(),
        context_menu: context_menu.map(|menus| {
            vec![menu::Tree::with_children(
                crate::Element::from(crate::widget::row::<'static, Message>()),
                menus,
            )]
        }),
        close_on_escape: true,
        window_id: window::Id::RESERVED,
        on_surface_action: None,
    };

    if let Some(ref mut context_menu) = this.context_menu {
        context_menu.iter_mut().for_each(menu::Tree::set_index);
    }

    this
}

/// A context menu is a menu in a graphical user interface that appears upon user interaction, such as a right-click mouse operation.
#[derive(Setters)]
#[must_use]
pub struct ContextMenu<'a, Message> {
    #[setters(skip)]
    content: crate::Element<'a, Message>,
    #[setters(skip)]
    context_menu: Option<Vec<menu::Tree<Message>>>,
    pub window_id: window::Id,
    pub close_on_escape: bool,
    #[setters(skip)]
    pub(crate) on_surface_action:
        Option<Arc<dyn Fn(crate::surface::Action) -> Message + Send + Sync + 'static>>,
}

impl<Message: Clone + 'static> ContextMenu<'_, Message> {
    #[cfg(all(feature = "wayland", feature = "winit", feature = "surface-message"))]
    #[allow(clippy::too_many_lines)]
    fn create_popup(
        &mut self,
        layout: iced_core::Layout<'_>,
        view_cursor: iced_core::mouse::Cursor,
        renderer: &crate::Renderer,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &iced::Rectangle,
        my_state: &mut LocalState,
    ) {
        if self.window_id != window::Id::NONE && self.on_surface_action.is_some() {
            use crate::{surface::action::destroy_popup, widget::menu::Menu};
            use iced_runtime::platform_specific::wayland::popup::{
                SctkPopupSettings, SctkPositioner,
            };

            let mut bounds = layout.bounds();
            bounds.x = my_state.context_cursor.x;
            bounds.y = my_state.context_cursor.y;

            let (id, root_list) = my_state.menu_bar_state.inner.with_data_mut(|state| {
                if let Some(id) = state.popup_id.get(&self.window_id).copied() {
                    // close existing popups
                    state.menu_states.clear();
                    state.active_root.clear();
                    shell.publish(self.on_surface_action.as_ref().unwrap()(destroy_popup(id)));
                    state.view_cursor = view_cursor;
                    (
                        id,
                        layout.children().map(|lo| lo.bounds()).collect::<Vec<_>>(),
                    )
                } else {
                    (
                        window::Id::unique(),
                        layout.children().map(|lo| lo.bounds()).collect(),
                    )
                }
            });
            let Some(context_menu) = self.context_menu.as_mut() else {
                return;
            };

            let mut popup_menu: Menu<'static, _> = Menu {
                tree: my_state.menu_bar_state.clone(),
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
                main_offset: -(bounds.height as i32),
                cross_offset: 0,
                root_bounds_list: vec![bounds],
                path_highlight: Some(PathHighlight::MenuActive),
                style: std::borrow::Cow::Owned(crate::theme::menu_bar::MenuBarStyle::Default),
                position: Point::new(0., 0.),
                is_overlay: false,
                window_id: id,
                depth: 0,
                on_surface_action: self.on_surface_action.clone(),
            };

            init_root_menu(
                &mut popup_menu,
                renderer,
                shell,
                view_cursor.position().unwrap(),
                viewport.size(),
                Vector::new(0., 0.),
                layout.bounds(),
                -bounds.height,
            );
            let (anchor_rect, gravity) = my_state.menu_bar_state.inner.with_data_mut(|state| {
                use iced::Rectangle;

                state.popup_id.insert(self.window_id, id);
                ({
                    let pos = view_cursor.position().unwrap_or_default();
                    Rectangle {
                        x: pos.x as i32,
                        y: pos.y as i32,
                        width: 1,
                        height: 1,
                    }
                },
                match (state.horizontal_direction, state.vertical_direction) {
                    (Direction::Positive, Direction::Positive) => cctk::wayland_protocols::xdg::shell::client::xdg_positioner::Gravity::BottomRight,
                    (Direction::Positive, Direction::Negative) => cctk::wayland_protocols::xdg::shell::client::xdg_positioner::Gravity::TopRight,
                    (Direction::Negative, Direction::Positive) => cctk::wayland_protocols::xdg::shell::client::xdg_positioner::Gravity::BottomLeft,
                    (Direction::Negative, Direction::Negative) => cctk::wayland_protocols::xdg::shell::client::xdg_positioner::Gravity::TopLeft,
                })
            });

            let menu_node =
                popup_menu.layout(renderer, iced::Limits::NONE.min_width(1.).min_height(1.));
            let popup_size = menu_node.size();
            let positioner = SctkPositioner {
                size: Some((
                    popup_size.width.ceil() as u32 + 2,
                    popup_size.height.ceil() as u32 + 2,
                )),
                anchor_rect,
                anchor: cctk::wayland_protocols::xdg::shell::client::xdg_positioner::Anchor::None,
                gravity: cctk::wayland_protocols::xdg::shell::client::xdg_positioner::Gravity::BottomRight,
                reactive: true,
                ..Default::default()
            };
            let parent = self.window_id;
            shell.publish((self.on_surface_action.as_ref().unwrap())(
                crate::surface::action::simple_popup(
                    move || SctkPopupSettings {
                        parent,
                        id,
                        positioner: positioner.clone(),
                        parent_size: None,
                        grab: true,
                        close_with_children: false,
                        input_zone: None,
                    },
                    Some(move || {
                        crate::Element::from(
                            crate::widget::container(popup_menu.clone()).center(Length::Fill),
                        )
                        .map(crate::action::app)
                    }),
                ),
            ));
        }
    }

    pub fn on_surface_action(
        mut self,
        handler: impl Fn(crate::surface::Action) -> Message + Send + Sync + 'static,
    ) -> Self {
        self.on_surface_action = Some(Arc::new(handler));
        self
    }
}

impl<Message: 'static + Clone> Widget<Message, crate::Theme, crate::Renderer>
    for ContextMenu<'_, Message>
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<LocalState>()
    }

    fn state(&self) -> tree::State {
        #[allow(clippy::default_trait_access)]
        tree::State::new(LocalState {
            context_cursor: Point::default(),
            fingers_pressed: Default::default(),
            menu_bar_state: Default::default(),
        })
    }

    fn children(&self) -> Vec<Tree> {
        let mut children = Vec::with_capacity(if self.context_menu.is_some() { 2 } else { 1 });

        children.push(Tree::new(self.content.as_widget()));

        // Assign the context menu's elements as this widget's children.
        if let Some(ref context_menu) = self.context_menu {
            let mut tree = Tree::empty();
            tree.children = context_menu
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
                .collect();

            children.push(tree);
        }

        children
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.children[0].diff(self.content.as_widget_mut());
        let state = tree.state.downcast_mut::<LocalState>();
        state.menu_bar_state.inner.with_data_mut(|inner| {
            menu_roots_diff(self.context_menu.as_mut().unwrap(), &mut inner.tree);
        });

        // if let Some(ref mut context_menus) = self.context_menu {
        //     for (menu, tree) in context_menus
        //         .iter_mut()
        //         .zip(tree.children[1].children.iter_mut())
        //     {
        //         menu.item.as_widget_mut().diff(tree);
        //     }
        // }
    }

    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &crate::Renderer,
        limits: &iced_core::layout::Limits,
    ) -> iced_core::layout::Node {
        self.content
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &iced_core::renderer::Style,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn operate(
        &self,
        tree: &mut Tree,
        layout: iced_core::Layout<'_>,
        renderer: &crate::Renderer,
        operation: &mut dyn iced_core::widget::Operation<()>,
    ) {
        self.content
            .as_widget()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }

    #[allow(clippy::too_many_lines)]
    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: iced::Event,
        layout: iced_core::Layout<'_>,
        cursor: iced_core::mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn iced_core::Clipboard,
        shell: &mut iced_core::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) -> iced_core::event::Status {
        let state = tree.state.downcast_mut::<LocalState>();
        let bounds = layout.bounds();

        // XXX this should reset the state if there are no other copies of the state, which implies no dropdown menus open.
        let reset = self.window_id != window::Id::NONE
            && state
                .menu_bar_state
                .inner
                .with_data(|d| !d.open && !d.active_root.is_empty());

        let open = state.menu_bar_state.inner.with_data_mut(|state| {
            if reset {
                if let Some(popup_id) = state.popup_id.get(&self.window_id).copied() {
                    if let Some(handler) = self.on_surface_action.as_ref() {
                        shell.publish((handler)(crate::surface::Action::DestroyPopup(popup_id)));
                        state.reset();
                    }
                }
            }
            state.open
        });
        let mut was_open = false;
        if matches!(event,
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            })
            | Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Right | mouse::Button::Left,
            ))
            | Event::Touch(touch::Event::FingerPressed { .. })
            | Event::Window(window::Event::Focused)
                if open )
        {
            state.menu_bar_state.inner.with_data_mut(|state| {
                was_open = true;
                state.menu_states.clear();
                state.active_root.clear();
                state.open = false;

                #[cfg(all(feature = "wayland", feature = "winit", feature = "surface-message"))]
                if matches!(WINDOWING_SYSTEM.get(), Some(WindowingSystem::Wayland)) {
                    if let Some(id) = state.popup_id.remove(&self.window_id) {
                        {
                            let surface_action = self.on_surface_action.as_ref().unwrap();
                            shell
                                .publish(surface_action(crate::surface::action::destroy_popup(id)));
                        }
                        state.view_cursor = cursor;
                    }
                }
            });
        }

        if !was_open && cursor.is_over(bounds) {
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

            // Present a context menu on a right click event.
            if !was_open
                && self.context_menu.is_some()
                && (right_button_released(&event) || (touch_lifted(&event) && fingers_pressed == 2))
            {
                state.context_cursor = cursor.position().unwrap_or_default();
                let state = tree.state.downcast_mut::<LocalState>();
                state.menu_bar_state.inner.with_data_mut(|state| {
                    state.open = true;
                    state.view_cursor = cursor;
                });
                #[cfg(all(feature = "wayland", feature = "winit", feature = "surface-message"))]
                if matches!(WINDOWING_SYSTEM.get(), Some(WindowingSystem::Wayland)) {
                    self.create_popup(layout, cursor, renderer, shell, viewport, state);
                }

                return event::Status::Captured;
            } else if !was_open && right_button_released(&event)
                || (touch_lifted(&event))
                || left_button_released(&event)
            {
                state.menu_bar_state.inner.with_data_mut(|state| {
                    was_open = true;
                    state.menu_states.clear();
                    state.active_root.clear();
                    state.open = false;

                    #[cfg(all(
                        feature = "wayland",
                        feature = "winit",
                        feature = "surface-message"
                    ))]
                    if matches!(WINDOWING_SYSTEM.get(), Some(WindowingSystem::Wayland)) {
                        if let Some(id) = state.popup_id.remove(&self.window_id) {
                            {
                                let surface_action = self.on_surface_action.as_ref().unwrap();
                                shell.publish(surface_action(
                                    crate::surface::action::destroy_popup(id),
                                ));
                            }
                            state.view_cursor = cursor;
                        }
                    }
                });
            }
        }
        self.content.as_widget_mut().on_event(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: iced_core::Layout<'_>,
        _renderer: &crate::Renderer,
        translation: Vector,
    ) -> Option<iced_core::overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        #[cfg(all(feature = "wayland", feature = "winit", feature = "surface-message"))]
        if matches!(WINDOWING_SYSTEM.get(), Some(WindowingSystem::Wayland))
            && self.window_id != window::Id::NONE
            && self.on_surface_action.is_some()
        {
            return None;
        }

        let state = tree.state.downcast_ref::<LocalState>();

        let context_menu = self.context_menu.as_mut()?;

        if !state.menu_bar_state.inner.with_data(|state| state.open) {
            return None;
        }

        let mut bounds = layout.bounds();
        bounds.x = state.context_cursor.x;
        bounds.y = state.context_cursor.y;
        Some(
            crate::widget::menu::Menu {
                tree: state.menu_bar_state.clone(),
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
                main_offset: -(bounds.height as i32),
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

    #[cfg(feature = "a11y")]
    /// get the a11y nodes for the widget
    fn a11y_nodes(
        &self,
        layout: iced_core::Layout<'_>,
        state: &Tree,
        p: mouse::Cursor,
    ) -> iced_accessibility::A11yTree {
        let c_state = &state.children[0];
        self.content.as_widget().a11y_nodes(layout, c_state, p)
    }
}

impl<'a, Message: Clone + 'static> From<ContextMenu<'a, Message>> for crate::Element<'a, Message> {
    fn from(widget: ContextMenu<'a, Message>) -> Self {
        Self::new(widget)
    }
}

fn right_button_released(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right,))
    )
}

fn left_button_released(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left,))
    )
}

fn touch_lifted(event: &Event) -> bool {
    matches!(event, Event::Touch(touch::Event::FingerLifted { .. }))
}

pub struct LocalState {
    context_cursor: Point,
    fingers_pressed: HashSet<Finger>,
    menu_bar_state: MenuBarState,
}
