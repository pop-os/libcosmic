// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! A context menu is a menu in a graphical user interface that appears upon user interaction, such as a right-click mouse operation.

use crate::widget::menu::{
    self, CloseCondition, ItemHeight, ItemWidth, MenuBarState, PathHighlight,
};
use derive_setters::Setters;
use iced::touch::Finger;
use iced::{Event, Vector};
use iced_core::widget::{Tree, Widget, tree};
use iced_core::{Length, Point, Size, event, mouse, touch};
use std::collections::HashSet;

use super::dropdown::menu::State;

/// A context menu is a menu in a graphical user interface that appears upon user interaction, such as a right-click mouse operation.
pub fn context_menu<Message: 'static + Clone>(
    content: impl Into<crate::Element<'static, Message>> + 'static,
    // on_context: Message,
    context_menu: Option<Vec<menu::Tree<Message>>>,
) -> ContextMenu<'static, Message> {
    let mut this = ContextMenu {
        content: content.into(),
        context_menu: context_menu.map(|menus| {
            vec![menu::Tree::with_children(
                crate::Element::from(crate::widget::row::<'static, Message>()),
                menus,
            )]
        }),
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
            menu_state: Default::default(),
        })
    }

    fn children(&self) -> Vec<Tree> {
        let mut children = Vec::with_capacity(if self.context_menu.is_some() { 2 } else { 1 });

        children.push(Tree::new(self.content.as_widget()));

        // Assign the context menu's elements as this widget's children.
        if let Some(ref context_menu) = self.context_menu {
            let mut tree = Tree::empty();
            tree.state = tree::State::new(MenuBarState::default());
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

        if cursor.is_over(bounds) {
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
            if self.context_menu.is_some()
                && (right_button_released(&event) || (touch_lifted(&event) && fingers_pressed == 2))
            {
                state.context_cursor = cursor.position().unwrap_or_default();
                let menu_state = tree.children[1].state.downcast_mut::<MenuBarState>();

                menu_state.inner.with_data_mut(|state| {
                    state.open = true;
                    state.view_cursor = cursor;
                });

                return event::Status::Captured;
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
        let state = tree.state.downcast_ref::<LocalState>();
        let menu_state = state.menu_state.clone();

        let Some(context_menu) = self.context_menu.as_mut() else {
            return None;
        };

        if !tree.children[1]
            .state
            .downcast_ref::<MenuBarState>()
            .inner
            .with_data(|state| state.open)
        {
            return None;
        }

        let mut bounds = layout.bounds();
        bounds.x = state.context_cursor.x;
        bounds.y = state.context_cursor.y;
        Some(
            crate::widget::menu::Menu {
                tree: menu_state,
                menu_roots: std::borrow::Cow::Borrowed(context_menu),
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

impl<'a, Message: Clone + 'static> From<ContextMenu<'static, Message>>
    for crate::Element<'static, Message>
{
    fn from(widget: ContextMenu<'static, Message>) -> Self {
        Self::new(widget)
    }
}

fn right_button_released(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Right,))
    )
}

fn touch_lifted(event: &Event) -> bool {
    matches!(event, Event::Touch(touch::Event::FingerLifted { .. }))
}

pub struct LocalState {
    context_cursor: Point,
    fingers_pressed: HashSet<Finger>,
    menu_state: MenuBarState,
}
