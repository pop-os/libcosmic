// Copyright 2023 System76 <info@system76.com>
// Copyright 2019 Héctor Ramón, Iced contributors
// SPDX-License-Identifier: MPL-2.0 AND MIT

use super::menu::{self, Menu};
use crate::widget::icon;
use derive_setters::Setters;
use iced_core::event::{self, Event};
use iced_core::text::{self, Text};
use iced_core::widget::tree::{self, Tree};
use iced_core::{alignment, keyboard, layout, mouse, overlay, renderer, svg, touch};
use iced_core::{Clipboard, Layout, Length, Padding, Pixels, Rectangle, Shell, Size, Widget};
use std::ffi::OsStr;

pub use iced_widget::style::pick_list::{Appearance, StyleSheet};

/// A widget for selecting a single value from a list of selections.
#[derive(Setters)]
pub struct Dropdown<'a, S: AsRef<str>, Message> {
    #[setters(skip)]
    on_selected: Box<dyn Fn(usize) -> Message + 'a>,
    #[setters(skip)]
    selections: &'a [S],
    #[setters(skip)]
    selected: Option<usize>,
    #[setters(into)]
    width: Length,
    gap: f32,
    #[setters(into)]
    padding: Padding,
    #[setters(strip_option)]
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    #[setters(strip_option)]
    font: Option<crate::font::Font>,
}

impl<'a, S: AsRef<str>, Message> Dropdown<'a, S, Message> {
    /// The default gap.
    pub const DEFAULT_GAP: f32 = 4.0;

    /// The default padding.
    pub const DEFAULT_PADDING: Padding = Padding::new(8.0);

    /// Creates a new [`Dropdown`] with the given list of selections, the current
    /// selected value, and the message to produce when an option is selected.
    pub fn new(
        selections: &'a [S],
        selected: Option<usize>,
        on_selected: impl Fn(usize) -> Message + 'a,
    ) -> Self {
        Self {
            on_selected: Box::new(on_selected),
            selections,
            selected,
            width: Length::Shrink,
            gap: Self::DEFAULT_GAP,
            padding: Self::DEFAULT_PADDING,
            text_size: None,
            text_line_height: text::LineHeight::Relative(1.2),
            font: None,
        }
    }
}

impl<'a, S: AsRef<str>, Message: 'a> Widget<Message, crate::Renderer> for Dropdown<'a, S, Message> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn width(&self) -> Length {
        self.width
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &crate::Renderer, limits: &layout::Limits) -> layout::Node {
        layout(
            renderer,
            limits,
            self.width,
            self.gap,
            self.padding,
            self.text_size.unwrap_or(14.0),
            self.text_line_height,
            self.font,
            self.selected
                .and_then(|id| self.selections.get(id))
                .map(AsRef::as_ref),
        )
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &crate::Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        update(
            &event,
            layout,
            cursor,
            shell,
            self.on_selected.as_ref(),
            self.selected,
            self.selections,
            || tree.state.downcast_mut::<State>(),
        )
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &crate::Renderer,
    ) -> mouse::Interaction {
        mouse_interaction(layout, cursor)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        _style: &iced_core::renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let font = self
            .font
            .unwrap_or_else(|| text::Renderer::default_font(renderer));
        draw(
            renderer,
            theme,
            layout,
            cursor,
            self.gap,
            self.padding,
            self.text_size,
            self.text_line_height,
            font,
            self.selected.and_then(|id| self.selections.get(id)),
            tree.state.downcast_ref::<State>(),
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
    ) -> Option<overlay::Element<'b, Message, crate::Renderer>> {
        let state = tree.state.downcast_mut::<State>();

        overlay(
            layout,
            renderer,
            state,
            self.gap,
            self.padding,
            self.text_size.unwrap_or(14.0),
            self.font,
            self.selections,
            self.selected,
            &self.on_selected,
        )
    }
}

impl<'a, S: AsRef<str>, Message: 'a> From<Dropdown<'a, S, Message>>
    for crate::Element<'a, Message>
{
    fn from(pick_list: Dropdown<'a, S, Message>) -> Self {
        Self::new(pick_list)
    }
}

/// The local state of a [`Dropdown`].
#[derive(Debug)]
pub struct State {
    icon: Option<svg::Handle>,
    menu: menu::State,
    keyboard_modifiers: keyboard::Modifiers,
    is_open: bool,
    hovered_option: Option<usize>,
}

impl State {
    /// Creates a new [`State`] for a [`Dropdown`].
    pub fn new() -> Self {
        Self {
            icon: match icon::from_name("pan-down-symbolic").size(16).handle().data {
                icon::Data::Name(named) => named
                    .path()
                    .filter(|path| path.extension().is_some_and(|ext| ext == OsStr::new("svg")))
                    .map(iced_core::svg::Handle::from_path),
                icon::Data::Svg(handle) => Some(handle),
                icon::Data::Image(_) => None,
            },
            menu: menu::State::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            is_open: false,
            hovered_option: None,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes the layout of a [`Dropdown`].
#[allow(clippy::too_many_arguments)]
pub fn layout(
    renderer: &crate::Renderer,
    limits: &layout::Limits,
    width: Length,
    gap: f32,
    padding: Padding,
    text_size: f32,
    text_line_height: text::LineHeight,
    font: Option<crate::font::Font>,
    selection: Option<&str>,
) -> layout::Node {
    use std::f32;

    let limits = limits.width(width).height(Length::Shrink).pad(padding);

    let max_width = match width {
        Length::Shrink => {
            let measure = |label: &str| -> f32 {
                let width = text::Renderer::measure_width(
                    renderer,
                    label,
                    text_size,
                    font.unwrap_or_else(|| text::Renderer::default_font(renderer)),
                    text::Shaping::Advanced,
                );

                width.round()
            };

            selection.map(measure).unwrap_or_default()
        }
        _ => 0.0,
    };

    let size = {
        let intrinsic = Size::new(
            max_width + gap + 16.0,
            f32::from(text_line_height.to_absolute(Pixels(text_size))),
        );

        limits.resolve(intrinsic).pad(padding)
    };

    layout::Node::new(size)
}

/// Processes an [`Event`] and updates the [`State`] of a [`Dropdown`]
/// accordingly.
#[allow(clippy::too_many_arguments)]
pub fn update<'a, S: AsRef<str>, Message>(
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    on_selected: &dyn Fn(usize) -> Message,
    selected: Option<usize>,
    selections: &[S],
    state: impl FnOnce() -> &'a mut State,
) -> event::Status {
    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
        | Event::Touch(touch::Event::FingerPressed { .. }) => {
            let state = state();

            if state.is_open {
                // Event wasn't processed by overlay, so cursor was clicked either outside it's
                // bounds or on the drop-down, either way we close the overlay.
                state.is_open = false;

                event::Status::Captured
            } else if cursor.is_over(layout.bounds()) {
                state.is_open = true;
                state.hovered_option = selected;

                event::Status::Captured
            } else {
                event::Status::Ignored
            }
        }
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Lines { .. },
        }) => {
            let state = state();

            if state.keyboard_modifiers.command()
                && cursor.is_over(layout.bounds())
                && !state.is_open
            {
                let next_index = selected.map(|index| index + 1).unwrap_or_default();

                if selections.len() < next_index {
                    shell.publish((on_selected)(next_index));
                }

                event::Status::Captured
            } else {
                event::Status::Ignored
            }
        }
        Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
            let state = state();

            state.keyboard_modifiers = *modifiers;

            event::Status::Ignored
        }
        _ => event::Status::Ignored,
    }
}

/// Returns the current [`mouse::Interaction`] of a [`Dropdown`].
#[must_use]
pub fn mouse_interaction(layout: Layout<'_>, cursor: mouse::Cursor) -> mouse::Interaction {
    let bounds = layout.bounds();
    let is_mouse_over = cursor.is_over(bounds);

    if is_mouse_over {
        mouse::Interaction::Pointer
    } else {
        mouse::Interaction::default()
    }
}

/// Returns the current overlay of a [`Dropdown`].
#[allow(clippy::too_many_arguments)]
pub fn overlay<'a, S: AsRef<str>, Message: 'a>(
    layout: Layout<'_>,
    renderer: &crate::Renderer,
    state: &'a mut State,
    gap: f32,
    padding: Padding,
    text_size: f32,
    font: Option<crate::font::Font>,
    selections: &'a [S],
    selected_option: Option<usize>,
    on_selected: &'a dyn Fn(usize) -> Message,
) -> Option<overlay::Element<'a, Message, crate::Renderer>> {
    if state.is_open {
        let bounds = layout.bounds();

        let menu = Menu::new(
            &mut state.menu,
            selections,
            &mut state.hovered_option,
            selected_option,
            |option| {
                state.is_open = false;

                (on_selected)(option)
            },
            None,
        )
        .width({
            let measure = |label: &str| -> f32 {
                let width = text::Renderer::measure_width(
                    renderer,
                    label,
                    text_size,
                    crate::font::FONT,
                    text::Shaping::Advanced,
                );

                width.round()
            };

            selections
                .iter()
                .map(|label| measure(label.as_ref()))
                .fold(0.0, |next, current| current.max(next))
                + gap
                + 16.0
                + padding.horizontal()
                + padding.horizontal()
        })
        .padding(padding)
        .text_size(text_size);

        let mut position = layout.position();
        position.x -= padding.left;

        Some(menu.overlay(position, bounds.height))
    } else {
        None
    }
}

/// Draws a [`Dropdown`].
#[allow(clippy::too_many_arguments)]
pub fn draw<'a, S>(
    renderer: &mut crate::Renderer,
    theme: &crate::Theme,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    gap: f32,
    padding: Padding,
    text_size: Option<f32>,
    text_line_height: text::LineHeight,
    font: crate::font::Font,
    selected: Option<&'a S>,
    state: &'a State,
) where
    S: AsRef<str> + 'a,
{
    let bounds = layout.bounds();
    let is_mouse_over = cursor.is_over(bounds);

    let style = if is_mouse_over {
        theme.hovered(&())
    } else {
        theme.active(&())
    };

    iced_core::Renderer::fill_quad(
        renderer,
        renderer::Quad {
            bounds,
            border_color: style.border_color,
            border_width: style.border_width,
            border_radius: style.border_radius,
        },
        style.background,
    );

    if let Some(handle) = state.icon.clone() {
        svg::Renderer::draw(
            renderer,
            handle,
            Some(style.text_color),
            Rectangle {
                x: bounds.x + bounds.width - gap - 16.0,
                y: bounds.center_y() - 8.0,
                width: 16.0,
                height: 16.0,
            },
        );
    }

    if let Some(content) = selected.map(AsRef::as_ref) {
        let text_size = text_size.unwrap_or_else(|| text::Renderer::default_size(renderer));

        text::Renderer::fill_text(
            renderer,
            Text {
                content,
                size: text_size,
                line_height: text_line_height,
                font,
                color: style.text_color,
                bounds: Rectangle {
                    x: bounds.x + padding.left,
                    y: bounds.center_y(),
                    width: bounds.width - padding.horizontal(),
                    height: f32::from(text_line_height.to_absolute(Pixels(text_size))),
                },
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
                shaping: text::Shaping::Advanced,
            },
        );
    }
}
