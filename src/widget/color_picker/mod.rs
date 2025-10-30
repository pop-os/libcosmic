// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Widgets for selecting colors with a color picker.

use std::borrow::Cow;
use std::iter;
use std::rc::Rc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use crate::Element;
use crate::theme::iced::Slider;
use crate::theme::{Button, THEME};
use crate::widget::{button::Catalog, container, segmented_button::Entity, slider};
use derive_setters::Setters;
use iced::Task;
use iced_core::event::{self, Event};
use iced_core::gradient::{ColorStop, Linear};
use iced_core::renderer::Quad;
use iced_core::widget::{Tree, tree};
use iced_core::{
    Background, Border, Clipboard, Color, Layout, Length, Radians, Rectangle, Renderer, Shadow,
    Shell, Size, Vector, Widget, layout, mouse, renderer,
};

use iced_widget::slider::HandleShape;
use iced_widget::{Row, canvas, column, horizontal_space, row, scrollable, vertical_space};
use palette::{FromColor, RgbHue};

use super::divider::horizontal;
use super::icon::{self, from_name};
use super::segmented_button::{self, SingleSelect};
use super::{Icon, button, segmented_control, text, text_input, tooltip};

#[doc(inline)]
pub use ColorPickerModel as Model;

// TODO is this going to look correct enough?
pub static HSV_RAINBOW: LazyLock<Vec<Color>> = LazyLock::new(|| {
    (0u16..8)
        .map(|h| {
            Color::from(palette::Srgba::from_color(palette::Hsv::new_srgb_const(
                RgbHue::new(f32::from(h) * 360.0 / 7.0),
                1.0,
                1.0,
            )))
        })
        .collect()
});

fn hsv_rainbow(low_hue: f32, high_hue: f32) -> Vec<ColorStop> {
    let mut colors = Vec::new();
    let steps: u8 = 7;
    let step_size = (high_hue - low_hue) / f32::from(steps);
    for i in 0..=steps {
        let hue = low_hue + step_size * f32::from(i);
        colors.push(ColorStop {
            color: Color::from(palette::Srgba::from_color(palette::Hsv::new_srgb_const(
                RgbHue::new(hue),
                1.0,
                1.0,
            ))),
            offset: f32::from(i) / f32::from(steps),
        });
    }
    colors
}

const MAX_RECENT: usize = 20;

#[derive(Debug, Clone)]
pub enum ColorPickerUpdate {
    ActiveColor(palette::Hsv),
    ActionFinished,
    Input(String),
    AppliedColor,
    Reset,
    ActivateSegmented(Entity),
    Copied(Instant),
    Cancel,
    ToggleColorPicker,
}

#[derive(Setters)]
pub struct ColorPickerModel {
    #[setters(skip)]
    segmented_model: segmented_button::Model<SingleSelect>,
    #[setters(skip)]
    active_color: palette::Hsv,
    #[setters(skip)]
    save_next: Option<Color>,
    #[setters(skip)]
    input_color: String,
    #[setters(skip)]
    applied_color: Option<Color>,
    #[setters(skip)]
    fallback_color: Option<Color>,
    #[setters(skip)]
    recent_colors: Vec<Color>,
    active: bool,
    width: Length,
    height: Length,
    #[setters(skip)]
    must_clear_cache: Rc<AtomicBool>,
    #[setters(skip)]
    copied_at: Option<Instant>,
}

impl ColorPickerModel {
    #[must_use]
    pub fn new(
        hex: impl Into<Cow<'static, str>> + Clone,
        rgb: impl Into<Cow<'static, str>> + Clone,
        fallback_color: Option<Color>,
        initial_color: Option<Color>,
    ) -> Self {
        let initial = initial_color.or(fallback_color);
        let initial_srgb = palette::Srgb::from(initial.unwrap_or(Color::BLACK));
        let hsv = palette::Hsv::from_color(initial_srgb);
        Self {
            segmented_model: segmented_button::Model::builder()
                .insert(move |b| b.text(hex.clone()).activate())
                .insert(move |b| b.text(rgb.clone()))
                .build(),
            active_color: hsv,
            save_next: None,
            input_color: color_to_string(hsv, true),
            applied_color: initial,
            fallback_color,
            recent_colors: Vec::new(), // TODO should all color pickers show the same recent colors?
            active: false,
            width: Length::Fixed(300.0),
            height: Length::Fixed(200.0),
            must_clear_cache: Rc::new(AtomicBool::new(false)),
            copied_at: None,
        }
    }

    /// Get a color picker button that displays the applied color
    ///
    pub fn picker_button<
        'a,
        Message: 'static + std::clone::Clone,
        T: Fn(ColorPickerUpdate) -> Message,
    >(
        &self,
        f: T,
        icon_portion: Option<u16>,
    ) -> crate::widget::Button<'a, Message> {
        color_button(
            Some(f(ColorPickerUpdate::ToggleColorPicker)),
            self.applied_color,
            Length::FillPortion(icon_portion.unwrap_or(12)),
        )
    }

    pub fn update<Message>(&mut self, update: ColorPickerUpdate) -> Task<Message> {
        match update {
            ColorPickerUpdate::ActiveColor(c) => {
                self.must_clear_cache.store(true, Ordering::SeqCst);
                self.input_color = color_to_string(c, self.is_hex());
                if let Some(to_save) = self.save_next.take() {
                    self.recent_colors.insert(0, to_save);
                    self.recent_colors.truncate(MAX_RECENT);
                }
                self.active_color = c;
                self.copied_at = None;
            }
            ColorPickerUpdate::AppliedColor => {
                let srgb = palette::Srgb::from_color(self.active_color);
                if let Some(applied_color) = self.applied_color.take() {
                    self.recent_colors.push(applied_color);
                }
                self.applied_color = Some(Color::from(srgb));
                self.active = false;
            }
            ColorPickerUpdate::ActivateSegmented(e) => {
                self.segmented_model.activate(e);
                self.input_color = color_to_string(self.active_color, self.is_hex());
                self.copied_at = None;
            }
            ColorPickerUpdate::Copied(t) => {
                self.copied_at = Some(t);

                return iced::clipboard::write(self.input_color.clone());
            }
            ColorPickerUpdate::Reset => {
                self.must_clear_cache.store(true, Ordering::SeqCst);

                let initial_srgb = palette::Srgb::from(self.fallback_color.unwrap_or(Color::BLACK));
                let hsv = palette::Hsv::from_color(initial_srgb);
                self.active_color = hsv;
                self.applied_color = self.fallback_color;
                self.copied_at = None;
            }
            ColorPickerUpdate::Cancel => {
                self.must_clear_cache.store(true, Ordering::SeqCst);

                self.active = false;
                self.copied_at = None;
            }
            ColorPickerUpdate::Input(c) => {
                self.must_clear_cache.store(true, Ordering::SeqCst);

                self.input_color = c;
                self.copied_at = None;
                // parse as rgba or hex and update active color
                if let Ok(c) = self.input_color.parse::<css_color::Srgb>() {
                    self.active_color =
                        palette::Hsv::from_color(palette::Srgb::new(c.red, c.green, c.blue));
                }
            }
            ColorPickerUpdate::ActionFinished => {
                let srgb = palette::Srgb::from_color(self.active_color);
                if let Some(applied_color) = self.applied_color.take() {
                    self.recent_colors.push(applied_color);
                }
                self.applied_color = Some(Color::from(srgb));
                self.active = false;
                self.save_next = Some(Color::from(srgb));
            }
            ColorPickerUpdate::ToggleColorPicker => {
                self.must_clear_cache.store(true, Ordering::SeqCst);
                self.active = !self.active;
                self.copied_at = None;
            }
        };
        Task::none()
    }

    #[must_use]
    pub fn is_hex(&self) -> bool {
        self.segmented_model.position(self.segmented_model.active()) == Some(0)
    }

    /// Get whether or not the picker should be visible
    #[must_use]
    pub fn get_is_active(&self) -> bool {
        self.active
    }

    /// Get the applied color of the picker
    #[must_use]
    pub fn get_applied_color(&self) -> Option<Color> {
        self.applied_color
    }

    #[must_use]
    pub fn builder<Message>(
        &self,
        on_update: fn(ColorPickerUpdate) -> Message,
    ) -> ColorPickerBuilder<'_, Message> {
        ColorPickerBuilder {
            model: &self.segmented_model,
            active_color: self.active_color,
            recent_colors: &self.recent_colors,
            on_update,
            width: self.width,
            height: self.height,
            must_clear_cache: self.must_clear_cache.clone(),
            input_color: &self.input_color,
            reset_label: None,
            save_label: None,
            cancel_label: None,
            copied_at: self.copied_at,
        }
    }
}

#[derive(Setters, Clone)]
pub struct ColorPickerBuilder<'a, Message> {
    #[setters(skip)]
    model: &'a segmented_button::Model<SingleSelect>,
    #[setters(skip)]
    active_color: palette::Hsv,
    #[setters(skip)]
    input_color: &'a str,
    #[setters(skip)]
    on_update: fn(ColorPickerUpdate) -> Message,
    #[setters(skip)]
    recent_colors: &'a Vec<Color>,
    #[setters(skip)]
    must_clear_cache: Rc<AtomicBool>,
    #[setters(skip)]
    copied_at: Option<Instant>,
    // can be set
    width: Length,
    height: Length,
    #[setters(strip_option, into)]
    reset_label: Option<Cow<'a, str>>,
    #[setters(strip_option, into)]
    save_label: Option<Cow<'a, str>>,
    #[setters(strip_option, into)]
    cancel_label: Option<Cow<'a, str>>,
}

impl<'a, Message> ColorPickerBuilder<'a, Message>
where
    Message: Clone + 'static,
{
    #[allow(clippy::too_many_lines)]
    pub fn build<T: Into<Cow<'a, str>> + 'a>(
        mut self,
        recent_colors_label: T,
        copy_to_clipboard_label: T,
        copied_to_clipboard_label: T,
    ) -> ColorPicker<'a, Message> {
        fn rail_backgrounds(hue: f32) -> (Background, Background) {
            let low_range = hsv_rainbow(0., hue);
            let high_range = hsv_rainbow(hue, 360.);

            (
                Background::Gradient(iced::Gradient::Linear(
                    Linear::new(Radians(90.0)).add_stops(low_range),
                )),
                Background::Gradient(iced::Gradient::Linear(
                    Linear::new(Radians(90.0)).add_stops(high_range),
                )),
            )
        }

        let on_update = self.on_update;
        let spacing = THEME.lock().unwrap().cosmic().spacing;

        let mut inner = column![
            // segmented buttons
            segmented_control::horizontal(self.model)
                .on_activate(Box::new(move |e| on_update(
                    ColorPickerUpdate::ActivateSegmented(e)
                )))
                .minimum_button_width(0)
                .width(self.width),
            // canvas with gradient for the current color
            // still needs the canvas and the handle to be drawn on it
            container(vertical_space().height(self.height))
                .width(self.width)
                .height(self.height),
            slider(
                0.001..=359.99,
                self.active_color.hue.into_positive_degrees(),
                move |v| {
                    let mut new = self.active_color;
                    new.hue = v.into();
                    on_update(ColorPickerUpdate::ActiveColor(new))
                }
            )
            .on_release(on_update(ColorPickerUpdate::ActionFinished))
            .class(Slider::Custom {
                active: Rc::new(move |t| {
                    let cosmic = t.cosmic();
                    let mut a =
                        slider::Catalog::style(t, &Slider::default(), slider::Status::Active);
                    let hue = self.active_color.hue.into_positive_degrees();
                    a.rail.backgrounds = rail_backgrounds(hue);
                    a.rail.width = 8.0;
                    a.handle.background = Color::TRANSPARENT.into();
                    a.handle.shape = HandleShape::Circle { radius: 8.0 };
                    a.handle.border_color = cosmic.palette.neutral_10.into();
                    a.handle.border_width = 4.0;
                    a
                }),
                hovered: Rc::new(move |t| {
                    let cosmic = t.cosmic();
                    let mut a =
                        slider::Catalog::style(t, &Slider::default(), slider::Status::Active);
                    let hue = self.active_color.hue.into_positive_degrees();
                    a.rail.backgrounds = rail_backgrounds(hue);
                    a.rail.width = 8.0;
                    a.handle.background = Color::TRANSPARENT.into();
                    a.handle.shape = HandleShape::Circle { radius: 8.0 };
                    a.handle.border_color = cosmic.palette.neutral_10.into();
                    a.handle.border_width = 4.0;
                    a
                }),
                dragging: Rc::new(move |t| {
                    let cosmic = t.cosmic();
                    let mut a =
                        slider::Catalog::style(t, &Slider::default(), slider::Status::Active);
                    let hue = self.active_color.hue.into_positive_degrees();
                    a.rail.backgrounds = rail_backgrounds(hue);
                    a.rail.width = 8.0;
                    a.handle.background = Color::TRANSPARENT.into();
                    a.handle.shape = HandleShape::Circle { radius: 8.0 };
                    a.handle.border_color = cosmic.palette.neutral_10.into();
                    a.handle.border_width = 4.0;
                    a
                }),
            })
            .width(self.width),
            text_input("", self.input_color)
                .on_input(move |s| on_update(ColorPickerUpdate::Input(s)))
                .on_paste(move |s| on_update(ColorPickerUpdate::Input(s)))
                .on_submit(move |_| on_update(ColorPickerUpdate::AppliedColor))
                .leading_icon(
                    color_button(
                        None,
                        Some(Color::from(palette::Srgb::from_color(self.active_color))),
                        Length::FillPortion(12)
                    )
                    .into()
                )
                // TODO copy paste input contents
                .trailing_icon({
                    let button = button::custom(crate::widget::icon(
                        from_name("edit-copy-symbolic").size(spacing.space_s).into(),
                    ))
                    .on_press(on_update(ColorPickerUpdate::Copied(Instant::now())))
                    .class(Button::Text);

                    match self.copied_at.take() {
                        Some(t) if Instant::now().duration_since(t) > Duration::from_secs(2) => {
                            button.into()
                        }
                        Some(_) => tooltip(
                            button,
                            text(copied_to_clipboard_label),
                            iced_widget::tooltip::Position::Bottom,
                        )
                        .into(),
                        None => tooltip(
                            button,
                            text(copy_to_clipboard_label),
                            iced_widget::tooltip::Position::Bottom,
                        )
                        .into(),
                    }
                })
                .width(self.width),
        ]
        // Should we ensure the side padding is at least half the width of the handle?
        .padding([
            spacing.space_none,
            spacing.space_s,
            spacing.space_s,
            spacing.space_s,
        ])
        .spacing(spacing.space_s);

        if !self.recent_colors.is_empty() {
            inner = inner.push(horizontal::light().width(self.width));
            inner = inner.push(
                column![text(recent_colors_label), {
                    // TODO get global colors from some cache?
                    // TODO how to handle overflow? should this use a grid widget for the list or a horizontal scroll and a limit for the max?
                    crate::widget::scrollable(
                        Row::with_children(self.recent_colors.iter().map(|c| {
                            let initial_srgb = palette::Srgb::from(*c);
                            let hsv = palette::Hsv::from_color(initial_srgb);
                            color_button(
                                Some(on_update(ColorPickerUpdate::ActiveColor(hsv))),
                                Some(*c),
                                Length::FillPortion(12),
                            )
                            .into()
                        }))
                        .padding([0.0, 0.0, f32::from(spacing.space_m), 0.0])
                        .spacing(spacing.space_xxs),
                    )
                    .width(self.width)
                    .direction(iced_widget::scrollable::Direction::Horizontal(
                        scrollable::Scrollbar::new().anchor(scrollable::Anchor::End),
                    ))
                }]
                .spacing(spacing.space_xxs),
            );
        }

        if let Some(reset_to_default) = self.reset_label.take() {
            inner = inner.push(
                column![
                    horizontal::light().width(self.width),
                    button::custom(
                        text(reset_to_default)
                            .width(self.width)
                            .align_x(iced_core::Alignment::Center)
                    )
                    .width(self.width)
                    .on_press(on_update(ColorPickerUpdate::Reset))
                ]
                .spacing(spacing.space_xs)
                .width(self.width),
            );
        }
        if let (Some(save), Some(cancel)) = (self.save_label.take(), self.cancel_label.take()) {
            inner = inner.push(
                column![
                    horizontal::light().width(self.width),
                    button::custom(
                        text(cancel)
                            .width(self.width)
                            .align_x(iced_core::Alignment::Center)
                    )
                    .width(self.width)
                    .on_press(on_update(ColorPickerUpdate::Cancel)),
                    button::custom(
                        text(save)
                            .width(self.width)
                            .align_x(iced_core::Alignment::Center)
                    )
                    .width(self.width)
                    .on_press(on_update(ColorPickerUpdate::AppliedColor))
                    .class(Button::Suggested)
                ]
                .spacing(spacing.space_xs)
                .width(self.width),
            );
        }

        ColorPicker {
            on_update,
            inner: inner.into(),
            width: self.width,
            active_color: self.active_color,
            must_clear_cache: self.must_clear_cache,
        }
    }
}

#[must_use]
pub struct ColorPicker<'a, Message> {
    pub(crate) on_update: fn(ColorPickerUpdate) -> Message,
    width: Length,
    active_color: palette::Hsv,
    inner: Element<'a, Message>,
    must_clear_cache: Rc<AtomicBool>,
}

impl<Message> Widget<Message, crate::Theme, crate::Renderer> for ColorPicker<'_, Message>
where
    Message: Clone + 'static,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn diff(&mut self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_mut(&mut self.inner));
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.inner)]
    }

    fn layout(
        &self,
        tree: &mut Tree,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.inner
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut crate::Renderer,
        theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let column_layout = layout;
        // First draw children
        self.inner.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        // Draw saturation value canvas
        let state: &State = tree.state.downcast_ref();

        let active_color = self.active_color;
        let canvas_layout = column_layout.children().nth(1).unwrap();

        if self
            .must_clear_cache
            .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
            .unwrap_or_default()
        {
            state.canvas_cache.clear();
        }
        let geo = state
            .canvas_cache
            .draw(renderer, canvas_layout.bounds().size(), move |frame| {
                let column_count = frame.width() as u16;
                let row_count = frame.height() as u16;

                for column in 0..column_count {
                    for row in 0..row_count {
                        let saturation = f32::from(column) / frame.width();
                        let value = 1.0 - f32::from(row) / frame.height();

                        let mut c = active_color;
                        c.saturation = saturation;
                        c.value = value;
                        frame.fill_rectangle(
                            iced::Point::new(f32::from(column), f32::from(row)),
                            iced::Size::new(1.0, 1.0),
                            Color::from(palette::Srgb::from_color(c)),
                        );
                    }
                }
            });

        let translation = Vector::new(canvas_layout.bounds().x, canvas_layout.bounds().y);
        iced_core::Renderer::with_translation(renderer, translation, |renderer| {
            iced_renderer::geometry::Renderer::draw_geometry(renderer, geo);
        });

        let bounds = canvas_layout.bounds();
        // Draw the handle on the saturation value canvas

        let t = THEME.lock().unwrap().clone();
        let t = t.cosmic();
        let handle_radius = f32::from(t.space_xs()) / 2.0;
        let (x, y) = (
            self.active_color
                .saturation
                .mul_add(bounds.width, bounds.position().x)
                - handle_radius,
            (1.0 - self.active_color.value).mul_add(bounds.height, bounds.position().y)
                - handle_radius,
        );
        renderer.with_layer(
            Rectangle {
                x,
                y,
                width: handle_radius.mul_add(2.0, 1.0),
                height: handle_radius.mul_add(2.0, 1.0),
            },
            |renderer| {
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle {
                            x,
                            y,
                            width: handle_radius.mul_add(2.0, 1.0),
                            height: handle_radius.mul_add(2.0, 1.0),
                        },
                        border: Border {
                            width: 1.0,
                            color: t.palette.neutral_5.into(),
                            radius: (1.0 + handle_radius).into(),
                        },
                        shadow: Shadow::default(),
                    },
                    Color::TRANSPARENT,
                );
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle {
                            x,
                            y,
                            width: handle_radius * 2.0,
                            height: handle_radius * 2.0,
                        },
                        border: Border {
                            width: 1.0,
                            color: t.palette.neutral_10.into(),
                            radius: handle_radius.into(),
                        },
                        shadow: Shadow::default(),
                    },
                    Color::TRANSPARENT,
                );
            },
        );
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &crate::Renderer,
        translation: Vector,
    ) -> Option<iced_core::overlay::Element<'b, Message, crate::Theme, crate::Renderer>> {
        self.inner
            .as_widget_mut()
            .overlay(&mut state.children[0], layout, renderer, translation)
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &crate::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        // if the pointer is performing a drag, intercept pointer motion and button events
        // else check if event is handled by child elements
        // if the event is not handled by a child element, check if it is over the canvas when pressing a button
        let state: &mut State = tree.state.downcast_mut();
        let column_layout = layout;
        if state.dragging {
            let bounds = column_layout.children().nth(1).unwrap().bounds();
            match event {
                Event::Mouse(mouse::Event::CursorMoved { .. } | mouse::Event::CursorEntered) => {
                    if let Some(mut clamped) = cursor.position() {
                        clamped.x = clamped.x.clamp(bounds.x, bounds.x + bounds.width);
                        clamped.y = clamped.y.clamp(bounds.y, bounds.y + bounds.height);
                        let relative_pos = clamped - bounds.position();
                        let (s, v) = (
                            relative_pos.x / bounds.width,
                            1.0 - relative_pos.y / bounds.height,
                        );

                        let hsv: palette::Hsv = palette::Hsv::new(self.active_color.hue, s, v);
                        shell.publish((self.on_update)(ColorPickerUpdate::ActiveColor(hsv)));
                    }
                }
                Event::Mouse(
                    mouse::Event::ButtonReleased(mouse::Button::Left) | mouse::Event::CursorLeft,
                ) => {
                    shell.publish((self.on_update)(ColorPickerUpdate::ActionFinished));
                    state.dragging = false;
                }
                _ => return event::Status::Ignored,
            };
            return event::Status::Captured;
        }

        let column_tree = &mut tree.children[0];
        if self.inner.as_widget_mut().on_event(
            column_tree,
            event.clone(),
            column_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) == event::Status::Captured
        {
            return event::Status::Captured;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let bounds = column_layout.children().nth(1).unwrap().bounds();
                if let Some(point) = cursor.position_over(bounds) {
                    let relative_pos = point - bounds.position();
                    let (s, v) = (
                        relative_pos.x / bounds.width,
                        1.0 - relative_pos.y / bounds.height,
                    );
                    state.dragging = true;
                    let hsv: palette::Hsv = palette::Hsv::new(self.active_color.hue, s, v);
                    shell.publish((self.on_update)(ColorPickerUpdate::ActiveColor(hsv)));
                    event::Status::Captured
                } else {
                    event::Status::Ignored
                }
            }
            _ => event::Status::Ignored,
        }
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }
}

#[derive(Debug, Default)]
pub struct State {
    canvas_cache: canvas::Cache,
    dragging: bool,
}

impl State {
    fn new() -> Self {
        Self::default()
    }
}

impl<Message> ColorPicker<'_, Message> where Message: Clone + 'static {}
// TODO convert active color to hex or rgba
fn color_to_string(c: palette::Hsv, is_hex: bool) -> String {
    let srgb = palette::Srgb::from_color(c);
    let hex = srgb.into_format::<u8>();
    if is_hex {
        format!("#{:02X}{:02X}{:02X}", hex.red, hex.green, hex.blue)
    } else {
        format!("rgb({}, {}, {})", hex.red, hex.green, hex.blue)
    }
}

#[allow(clippy::too_many_lines)]
/// A button for selecting a color from a color picker.
pub fn color_button<'a, Message: Clone + 'static>(
    on_press: Option<Message>,
    color: Option<Color>,
    icon_portion: Length,
) -> crate::widget::Button<'a, Message> {
    let spacing = THEME.lock().unwrap().cosmic().spacing;

    button::custom(if color.is_some() {
        Element::from(vertical_space().height(Length::Fixed(f32::from(spacing.space_s))))
    } else {
        Element::from(column![
            vertical_space().height(Length::FillPortion(6)),
            row![
                horizontal_space().width(Length::FillPortion(6)),
                Icon::from(
                    icon::from_name("list-add-symbolic")
                        .prefer_svg(true)
                        .symbolic(true)
                        .size(64)
                )
                .width(icon_portion)
                .height(Length::Fill)
                .content_fit(iced_core::ContentFit::Contain),
                horizontal_space().width(Length::FillPortion(6)),
            ]
            .height(icon_portion)
            .width(Length::Fill),
            vertical_space().height(Length::FillPortion(6)),
        ])
    })
    .width(Length::Fixed(f32::from(spacing.space_s)))
    .height(Length::Fixed(f32::from(spacing.space_s)))
    .on_press_maybe(on_press)
    .class(crate::theme::Button::Custom {
        active: Box::new(move |focused, theme| {
            let cosmic = theme.cosmic();

            let (outline_width, outline_color) = if focused {
                (1.0, cosmic.accent_color().into())
            } else {
                (0.0, Color::TRANSPARENT)
            };
            let standard = theme.active(focused, false, &Button::Standard);
            button::Style {
                shadow_offset: Vector::default(),
                background: color.map(Background::from).or(standard.background),
                border_radius: cosmic.radius_xs().into(),
                border_width: 1.0,
                border_color: cosmic.palette.neutral_8.into(),
                outline_width,
                outline_color,
                icon_color: None,
                text_color: None,
                overlay: None,
            }
        }),
        disabled: Box::new(move |theme| {
            let cosmic = theme.cosmic();

            let standard = theme.disabled(&Button::Standard);
            button::Style {
                shadow_offset: Vector::default(),
                background: color.map(Background::from).or(standard.background),
                border_radius: cosmic.radius_xs().into(),
                border_width: 1.0,
                border_color: cosmic.palette.neutral_8.into(),
                outline_width: 0.0,
                outline_color: Color::TRANSPARENT,
                icon_color: None,
                text_color: None,
                overlay: None,
            }
        }),
        hovered: Box::new(move |focused, theme| {
            let cosmic = theme.cosmic();

            let (outline_width, outline_color) = if focused {
                (1.0, cosmic.accent_color().into())
            } else {
                (0.0, Color::TRANSPARENT)
            };

            let standard = theme.hovered(focused, false, &Button::Standard);
            button::Style {
                shadow_offset: Vector::default(),
                background: color.map(Background::from).or(standard.background),
                border_radius: cosmic.radius_xs().into(),
                border_width: 1.0,
                border_color: cosmic.palette.neutral_8.into(),
                outline_width,
                outline_color,
                icon_color: None,
                text_color: None,
                overlay: None,
            }
        }),
        pressed: Box::new(move |focused, theme| {
            let cosmic = theme.cosmic();

            let (outline_width, outline_color) = if focused {
                (1.0, cosmic.accent_color().into())
            } else {
                (0.0, Color::TRANSPARENT)
            };

            let standard = theme.pressed(focused, false, &Button::Standard);
            button::Style {
                shadow_offset: Vector::default(),
                background: color.map(Background::from).or(standard.background),
                border_radius: cosmic.radius_xs().into(),
                border_width: 1.0,
                border_color: cosmic.palette.neutral_8.into(),
                outline_width,
                outline_color,
                icon_color: None,
                text_color: None,
                overlay: None,
            }
        }),
    })
}

impl<'a, Message> From<ColorPicker<'a, Message>>
    for iced::Element<'a, Message, crate::Theme, crate::Renderer>
where
    Message: 'static + Clone,
{
    fn from(
        picker: ColorPicker<'a, Message>,
    ) -> iced::Element<'a, Message, crate::Theme, crate::Renderer> {
        Element::new(picker)
    }
}
