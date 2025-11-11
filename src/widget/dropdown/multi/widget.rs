// Copyright 2023 System76 <info@system76.com>
// Copyright 2019 Héctor Ramón, Iced contributors
// SPDX-License-Identifier: MPL-2.0 AND MIT

use super::menu::{self, Menu};
use crate::widget::icon;
use derive_setters::Setters;
use iced_core::event::{self, Event};
use iced_core::text::{self, Paragraph, Text};
use iced_core::widget::tree::{self, Tree};
use iced_core::{
    Clipboard, Layout, Length, Padding, Pixels, Rectangle, Shell, Size, Vector, Widget,
};
use iced_core::{Shadow, alignment, keyboard, layout, mouse, overlay, renderer, svg, touch};
use iced_widget::pick_list;
use std::ffi::OsStr;

pub use iced_widget::pick_list::{Catalog, Style};

/// A widget for selecting a single value from a list of selections.
#[derive(Setters)]
pub struct Dropdown<'a, S: AsRef<str>, Message, Item> {
    #[setters(skip)]
    on_selected: Box<dyn Fn(Item) -> Message + 'a>,
    #[setters(skip)]
    selections: &'a super::Model<S, Item>,
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

impl<'a, S: AsRef<str>, Message, Item: Clone + PartialEq + 'static> Dropdown<'a, S, Message, Item> {
    /// The default gap.
    pub const DEFAULT_GAP: f32 = 4.0;

    /// The default padding.
    pub const DEFAULT_PADDING: Padding = Padding::new(8.0);

    /// Creates a new [`Dropdown`] with the given list of selections, the current
    /// selected value, and the message to produce when an option is selected.
    pub fn new(
        selections: &'a super::Model<S, Item>,
        on_selected: impl Fn(Item) -> Message + 'a,
    ) -> Self {
        Self {
            on_selected: Box::new(on_selected),
            selections,
            width: Length::Shrink,
            gap: Self::DEFAULT_GAP,
            padding: Self::DEFAULT_PADDING,
            text_size: None,
            text_line_height: text::LineHeight::Relative(1.2),
            font: None,
        }
    }
}

impl<'a, S: AsRef<str>, Message: 'a, Item: Clone + PartialEq + 'static>
    Widget<Message, crate::Theme, crate::Renderer> for Dropdown<'a, S, Message, Item>
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Item>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Item>::new())
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout(
            renderer,
            limits,
            self.width,
            self.gap,
            self.padding,
            self.text_size.unwrap_or(14.0),
            self.text_line_height,
            self.font,
            self.selections.selected.as_ref().and_then(|id| {
                self.selections.get(id).map(AsRef::as_ref).zip({
                    let state = tree.state.downcast_mut::<State<Item>>();

                    if state.selections.is_empty() {
                        for list in &self.selections.lists {
                            for (_, item) in &list.options {
                                state
                                    .selections
                                    .push((item.clone(), crate::Plain::default()));
                            }
                        }
                    }

                    state
                        .selections
                        .iter_mut()
                        .find(|(i, _)| i == id)
                        .map(|(_, p)| p)
                })
            }),
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
            self.selections,
            || tree.state.downcast_mut::<State<Item>>(),
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
        viewport: &Rectangle,
    ) {
        let font = self.font.unwrap_or_else(crate::font::default);

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
            self.selections
                .selected
                .as_ref()
                .and_then(|id| self.selections.get(id)),
            tree.state.downcast_ref::<State<Item>>(),
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        let state = tree.state.downcast_mut::<State<Item>>();

        overlay(
            layout,
            renderer,
            state,
            self.gap,
            self.padding,
            self.text_size.unwrap_or(14.0),
            self.font,
            self.text_line_height,
            self.selections,
            &self.on_selected,
            translation,
        )
    }
}

impl<'a, S: AsRef<str>, Message: 'a, Item: Clone + PartialEq + 'static>
    From<Dropdown<'a, S, Message, Item>> for crate::Element<'a, Message>
{
    fn from(pick_list: Dropdown<'a, S, Message, Item>) -> Self {
        Self::new(pick_list)
    }
}

/// The local state of a [`Dropdown`].
#[derive(Debug)]
pub struct State<Item: Clone + PartialEq + 'static> {
    icon: Option<svg::Handle>,
    menu: menu::State,
    keyboard_modifiers: keyboard::Modifiers,
    is_open: bool,
    hovered_option: Option<Item>,
    selections: Vec<(Item, crate::Plain)>,
    descriptions: Vec<crate::Plain>,
}

impl<Item: Clone + PartialEq + 'static> State<Item> {
    /// Creates a new [`State`] for a [`Dropdown`].
    pub fn new() -> Self {
        Self {
            icon: match icon::from_name("pan-down-symbolic").size(16).handle().data {
                icon::Data::Svg(handle) => Some(handle),
                icon::Data::Image(_) => None,
            },
            menu: menu::State::default(),
            keyboard_modifiers: keyboard::Modifiers::default(),
            is_open: false,
            hovered_option: None,
            selections: Vec::new(),
            descriptions: Vec::new(),
        }
    }
}

impl<Item: Clone + PartialEq + 'static> Default for State<Item> {
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
    selection: Option<(&str, &mut crate::Plain)>,
) -> layout::Node {
    use std::f32;

    let limits = limits.width(width).height(Length::Shrink).shrink(padding);

    let max_width = match width {
        Length::Shrink => {
            let measure = move |(label, paragraph): (_, &mut crate::Plain)| -> f32 {
                paragraph.update(Text {
                    content: label,
                    bounds: Size::new(f32::MAX, f32::MAX),
                    size: iced::Pixels(text_size),
                    line_height: text_line_height,
                    font: font.unwrap_or_else(crate::font::default),
                    horizontal_alignment: alignment::Horizontal::Left,
                    vertical_alignment: alignment::Vertical::Top,
                    shaping: text::Shaping::Advanced,
                    wrapping: text::Wrapping::default(),
                });
                paragraph.min_width().round()
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

        limits
            .resolve(width, Length::Shrink, intrinsic)
            .expand(padding)
    };

    layout::Node::new(size)
}

/// Processes an [`Event`] and updates the [`State`] of a [`Dropdown`]
/// accordingly.
#[allow(clippy::too_many_arguments)]
pub fn update<'a, S: AsRef<str>, Message, Item: Clone + PartialEq + 'static + 'a>(
    event: &Event,
    layout: Layout<'_>,
    cursor: mouse::Cursor,
    shell: &mut Shell<'_, Message>,
    on_selected: &dyn Fn(Item) -> Message,
    selections: &super::Model<S, Item>,
    state: impl FnOnce() -> &'a mut State<Item>,
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
                state.hovered_option = selections.selected.clone();

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
                if let Some(option) = selections.next() {
                    shell.publish((on_selected)(option.1.clone()));
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
pub fn overlay<'a, S: AsRef<str>, Message: 'a, Item: Clone + PartialEq + 'static>(
    layout: Layout<'_>,
    renderer: &crate::Renderer,
    state: &'a mut State<Item>,
    gap: f32,
    padding: Padding,
    text_size: f32,
    font: Option<crate::font::Font>,
    text_line_height: text::LineHeight,
    selections: &'a super::Model<S, Item>,
    on_selected: &'a dyn Fn(Item) -> Message,
    translation: Vector,
) -> Option<overlay::Element<'a, Message, crate::Theme, crate::Renderer>> {
    if state.is_open {
        let description_line_height = text::LineHeight::Absolute(Pixels(
            text_line_height.to_absolute(Pixels(text_size)).0 + 4.0,
        ));

        let bounds = layout.bounds();

        let menu = Menu::new(
            &mut state.menu,
            selections,
            &mut state.hovered_option,
            selections.selected.as_ref(),
            |option| {
                state.is_open = false;

                (on_selected)(option)
            },
            None,
        )
        .width({
            let measure =
                |label: &str, paragraph: &mut crate::Plain, line_height: text::LineHeight| {
                    paragraph.update(Text {
                        content: label,
                        bounds: Size::new(f32::MAX, f32::MAX),
                        size: iced::Pixels(text_size),
                        line_height,
                        font: font.unwrap_or_else(crate::font::default),
                        horizontal_alignment: alignment::Horizontal::Left,
                        vertical_alignment: alignment::Vertical::Top,
                        shaping: text::Shaping::Advanced,
                        wrapping: text::Wrapping::default(),
                    });
                    paragraph.min_width().round()
                };

            let mut desc_count = 0;
            padding.horizontal().mul_add(
                2.0,
                selections
                    .elements()
                    .map(|element| match element {
                        super::menu::OptionElement::Description(desc) => {
                            let paragraph = if state.descriptions.len() > desc_count {
                                &mut state.descriptions[desc_count]
                            } else {
                                state.descriptions.push(crate::Plain::default());
                                state.descriptions.last_mut().unwrap()
                            };
                            desc_count += 1;
                            measure(desc.as_ref(), paragraph, description_line_height)
                        }

                        super::menu::OptionElement::Option((option, item)) => {
                            let selection_index =
                                state.selections.iter().position(|(i, _)| i == item);

                            let selection_index = match selection_index {
                                Some(index) => index,
                                None => {
                                    state
                                        .selections
                                        .push((item.clone(), crate::Plain::default()));
                                    state.selections.len() - 1
                                }
                            };

                            let paragraph = &mut state.selections[selection_index].1;

                            measure(option.as_ref(), paragraph, text_line_height)
                        }

                        super::menu::OptionElement::Separator => 1.0,
                    })
                    .fold(0.0, |next, current| current.max(next)),
            ) + gap
                + 16.0
        })
        .padding(padding)
        .text_size(text_size);

        let mut position = layout.position();
        position.x -= padding.left;
        position.x += translation.x;
        position.y += translation.y;
        Some(menu.overlay(position, bounds.height))
    } else {
        None
    }
}

/// Draws a [`Dropdown`].
#[allow(clippy::too_many_arguments)]
pub fn draw<'a, S, Item: Clone + PartialEq + 'static>(
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
    state: &'a State<Item>,
    viewport: &Rectangle,
) where
    S: AsRef<str> + 'a,
{
    let bounds = layout.bounds();
    let is_mouse_over = cursor.is_over(bounds);

    let style = if is_mouse_over {
        theme.style(&(), pick_list::Status::Hovered)
    } else {
        theme.style(&(), pick_list::Status::Active)
    };

    iced_core::Renderer::fill_quad(
        renderer,
        renderer::Quad {
            bounds,
            border: style.border,
            shadow: Shadow::default(),
        },
        style.background,
    );

    if let Some(handle) = state.icon.as_ref() {
        let svg_handle = iced_core::Svg::new(handle.clone()).color(style.text_color);
        svg::Renderer::draw_svg(
            renderer,
            svg_handle,
            Rectangle {
                x: bounds.x + bounds.width - gap - 16.0,
                y: bounds.center_y() - 8.0,
                width: 16.0,
                height: 16.0,
            },
        );
    }

    if let Some(content) = selected.map(AsRef::as_ref) {
        let text_size = text_size.unwrap_or_else(|| text::Renderer::default_size(renderer).0);

        let bounds = Rectangle {
            x: bounds.x + padding.left,
            y: bounds.center_y(),
            width: bounds.width - padding.horizontal(),
            height: f32::from(text_line_height.to_absolute(Pixels(text_size))),
        };

        text::Renderer::fill_text(
            renderer,
            Text {
                content: content.to_string(),
                size: iced::Pixels(text_size),
                line_height: text_line_height,
                font,
                bounds: bounds.size(),
                horizontal_alignment: alignment::Horizontal::Left,
                vertical_alignment: alignment::Vertical::Center,
                shaping: text::Shaping::Advanced,
                wrapping: text::Wrapping::default(),
            },
            bounds.position(),
            style.text_color,
            *viewport,
        );
    }
}
