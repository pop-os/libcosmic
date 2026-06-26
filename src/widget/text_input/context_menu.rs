// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Right-click editing context menu (cut/copy/paste/select all) for `text_input`.

use super::input::State;
use super::value::Value;
use crate::fl;
use iced_core::Renderer as _;
use iced_core::text::{self, Text};
use iced_core::{
    Border, Clipboard, Event, Layout, Pixels, Point, Rectangle, Shadow, Shell, Size, Vector,
    alignment, keyboard, layout, mouse, overlay, renderer, touch,
};

const ITEM_COUNT: usize = 4;
const ITEM_HEIGHT: f32 = 32.0;
const MENU_V_PADDING: f32 = 4.0;
const ITEM_H_PADDING: f32 = 16.0;
const ITEM_H_INSET: f32 = 4.0;
const MENU_MIN_WIDTH: f32 = 180.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContextMenuAction {
    Cut,
    Copy,
    Paste,
    SelectAll,
}

#[derive(Debug, Clone)]
pub(crate) struct ContextMenuState {
    pub position: Point,
    pub hovered: Option<usize>,
    pub paste_enabled: bool,
}

struct Item {
    action: ContextMenuAction,
    label: String,
    enabled: bool,
}

fn build_items(
    state: &State,
    value: &Value,
    is_secure: bool,
    is_read_only: bool,
    paste_enabled: bool,
) -> Vec<Item> {
    let has_selection = state.cursor_has_selection(value);
    let not_empty = !value.is_empty();

    vec![
        Item {
            action: ContextMenuAction::Cut,
            label: fl!("cut"),
            enabled: !is_secure && !is_read_only && has_selection,
        },
        Item {
            action: ContextMenuAction::Copy,
            label: fl!("copy"),
            enabled: !is_secure && has_selection,
        },
        Item {
            action: ContextMenuAction::Paste,
            label: fl!("paste"),
            enabled: !is_read_only && paste_enabled,
        },
        Item {
            action: ContextMenuAction::SelectAll,
            label: fl!("select-all"),
            enabled: not_empty,
        },
    ]
}

pub(crate) struct ContextMenu<'a, Message> {
    state: &'a mut State,
    value: &'a mut Value,
    is_secure: bool,
    is_read_only: bool,
    on_input: Option<&'a dyn Fn(String) -> Message>,
    on_paste: Option<&'a dyn Fn(String) -> Message>,
    anchor: Point,
    text_size: f32,
}

impl<'a, Message: 'a> ContextMenu<'a, Message> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        state: &'a mut State,
        value: &'a mut Value,
        is_secure: bool,
        is_read_only: bool,
        on_input: Option<&'a dyn Fn(String) -> Message>,
        on_paste: Option<&'a dyn Fn(String) -> Message>,
        anchor: Point,
        text_size: f32,
    ) -> Self {
        Self {
            state,
            value,
            is_secure,
            is_read_only,
            on_input,
            on_paste,
            anchor,
            text_size,
        }
    }

    pub(crate) fn overlay_element(
        self,
    ) -> overlay::Element<'a, Message, crate::Theme, crate::Renderer> {
        overlay::Element::new(Box::new(self))
    }

    fn items(&self) -> Vec<Item> {
        let paste_enabled = self
            .state
            .context_menu
            .as_ref()
            .is_some_and(|c| c.paste_enabled);
        build_items(
            self.state,
            self.value,
            self.is_secure,
            self.is_read_only,
            paste_enabled,
        )
    }

    fn menu_size(&self) -> Size {
        let longest = self
            .items()
            .iter()
            .map(|item| item.label.chars().count())
            .max()
            .unwrap_or(0);

        let width =
            (longest as f32 * self.text_size * 0.62 + 2.0 * ITEM_H_PADDING).max(MENU_MIN_WIDTH);
        let height = ITEM_COUNT as f32 * ITEM_HEIGHT + 2.0 * MENU_V_PADDING;

        Size::new(width, height)
    }

    fn item_at(&self, y_in_bounds: f32) -> Option<usize> {
        let y = y_in_bounds - MENU_V_PADDING;
        if y < 0.0 {
            return None;
        }
        let index = (y / ITEM_HEIGHT) as usize;
        (index < ITEM_COUNT).then_some(index)
    }

    fn close(&mut self) {
        self.state.context_menu = None;
    }
}

impl<Message> overlay::Overlay<Message, crate::Theme, crate::Renderer> for ContextMenu<'_, Message> {
    fn layout(&mut self, _renderer: &crate::Renderer, bounds: Size) -> layout::Node {
        let size = self.menu_size();

        let mut position = self.anchor;
        if position.x + size.width > bounds.width {
            position.x = (bounds.width - size.width).max(0.0);
        }
        if position.y + size.height > bounds.height {
            position.y = (bounds.height - size.height).max(0.0);
        }

        layout::Node::new(size).move_to(position)
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
    ) {
        let bounds = layout.bounds();

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                // The open-time clipboard snapshot can be stale right after an
                // external copy, so re-check while Paste is still greyed.
                let needs_clipboard_check = self
                    .state
                    .context_menu
                    .as_ref()
                    .is_some_and(|menu| !menu.paste_enabled);
                let paste_now = needs_clipboard_check
                    && clipboard
                        .read(iced_core::clipboard::Kind::Standard)
                        .unwrap_or_default()
                        .chars()
                        .any(|c| !c.is_control());

                let mut redraw = false;
                if paste_now {
                    if let Some(menu) = self.state.context_menu.as_mut() {
                        menu.paste_enabled = true;
                        redraw = true;
                    }
                }

                let new_hovered = cursor
                    .position_in(bounds)
                    .and_then(|p| self.item_at(p.y))
                    .filter(|&i| self.items().get(i).is_some_and(|item| item.enabled));

                if let Some(menu) = self.state.context_menu.as_mut() {
                    if menu.hovered != new_hovered {
                        menu.hovered = new_hovered;
                        redraw = true;
                    }
                }

                if redraw {
                    shell.request_redraw();
                }
            }

            Event::Mouse(mouse::Event::ButtonPressed(
                mouse::Button::Left | mouse::Button::Right,
            ))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let activated = cursor
                    .position_in(bounds)
                    .and_then(|p| self.item_at(p.y))
                    .and_then(|i| {
                        let items = self.items();
                        items.get(i).filter(|item| item.enabled).map(|item| item.action)
                    });

                if let Some(action) = activated {
                    self.state.apply_context_menu_action(
                        action,
                        self.value,
                        self.is_secure,
                        self.is_read_only,
                        self.on_input,
                        self.on_paste,
                        clipboard,
                        shell,
                    );
                    self.close();
                    shell.capture_event();
                } else if cursor.is_over(bounds) {
                    shell.capture_event();
                } else {
                    self.close();
                }
                shell.request_redraw();
            }

            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Escape),
                ..
            }) => {
                self.close();
                shell.capture_event();
                shell.request_redraw();
            }

            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        let over_enabled = cursor
            .position_in(layout.bounds())
            .and_then(|p| self.item_at(p.y))
            .is_some_and(|i| self.items().get(i).is_some_and(|item| item.enabled));

        if over_enabled {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
    ) {
        use crate::widget::dropdown::menu::StyleSheet;

        let appearance = StyleSheet::appearance(theme, &());
        let bounds = layout.bounds();

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    width: appearance.border_width,
                    color: appearance.border_color,
                    radius: appearance.border_radius,
                },
                shadow: Shadow::default(),
                snap: true,
            },
            appearance.background,
        );

        let hovered = self.state.context_menu.as_ref().and_then(|c| c.hovered);

        for (i, item) in self.items().iter().enumerate() {
            let item_bounds = Rectangle {
                x: bounds.x,
                y: ITEM_HEIGHT.mul_add(i as f32, bounds.y + MENU_V_PADDING),
                width: bounds.width,
                height: ITEM_HEIGHT,
            };

            let color = if !item.enabled {
                appearance.description_color
            } else if hovered == Some(i) {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: item_bounds.x + ITEM_H_INSET,
                            width: item_bounds.width - 2.0 * ITEM_H_INSET,
                            ..item_bounds
                        },
                        border: Border {
                            radius: appearance.border_radius,
                            ..Default::default()
                        },
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    appearance.hovered_background,
                );
                appearance.hovered_text_color
            } else {
                appearance.text_color
            };

            let text_bounds = Rectangle {
                x: item_bounds.x + ITEM_H_PADDING,
                y: item_bounds.center_y(),
                width: f32::INFINITY,
                height: item_bounds.height,
            };

            text::Renderer::fill_text(
                renderer,
                Text {
                    content: item.label.clone(),
                    bounds: text_bounds.size(),
                    size: Pixels(self.text_size),
                    line_height: text::LineHeight::default(),
                    font: crate::font::default(),
                    align_x: text::Alignment::Left,
                    align_y: alignment::Vertical::Center,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::default(),
                    ellipsize: text::Ellipsize::default(),
                },
                text_bounds.position(),
                color,
                bounds,
            );
        }
    }
}

pub(crate) fn anchor_position(field_position: Point, offset: Point, translation: Vector) -> Point {
    Point::new(
        field_position.x + offset.x + translation.x,
        field_position.y + offset.y + translation.y,
    )
}
