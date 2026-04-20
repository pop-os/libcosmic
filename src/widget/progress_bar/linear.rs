//! Show a linear progress indicator.
use super::animation::{Animation, Progress};
use super::style::StyleSheet;
use iced::advanced::layout;
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{self, Clipboard, Layout, Shell, Widget};
use iced::mouse;
use iced::window;
use iced::{Background, Element, Event, Length, Rectangle, Size};

use std::time::Duration;

const MIN_LENGTH: f32 = 0.15;
const WRAP_LENGTH: f32 = 0.618; // avoids animation repetition

#[must_use]
pub struct Linear<Theme>
where
    Theme: StyleSheet,
{
    width: Length,
    girth: Length,
    style: Theme::Style,
    cycle_duration: Duration,
    period: Duration,
    progress: Option<f32>,
}

impl<Theme> Linear<Theme>
where
    Theme: StyleSheet,
{
    /// Creates a new [`Linear`] with the given content.
    pub fn new() -> Self {
        Linear {
            width: Length::Fixed(100.0),
            girth: Length::Fixed(4.0),
            style: Theme::Style::default(),
            cycle_duration: Duration::from_millis(1500),
            period: Duration::from_secs(2),
            progress: None,
        }
    }

    /// Sets the width of the [`Linear`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the girth of the [`Linear`].
    pub fn girth(mut self, girth: impl Into<Length>) -> Self {
        self.girth = girth.into();
        self
    }

    /// Sets the style variant of this [`Linear`].
    pub fn style(mut self, style: impl Into<Theme::Style>) -> Self {
        self.style = style.into();
        self
    }

    /// Sets the cycle duration of this [`Linear`].
    pub fn cycle_duration(mut self, duration: Duration) -> Self {
        self.cycle_duration = duration / 2;
        self
    }

    /// Sets the base period of this [`Linear`]. This is the duration that a full traversal
    /// would take if the cycle duration were set to 0.0 (no expanding or contracting)
    pub fn period(mut self, duration: Duration) -> Self {
        self.period = duration;
        self
    }

    /// Override the default behavior by providing a determinate progress value between `0.0` and `1.0`.
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = Some(progress.clamp(0.0, 1.0));
        self
    }
}

impl<Theme> Default for Linear<Theme>
where
    Theme: StyleSheet,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
struct State {
    animation: Animation,
    progress: Progress,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Linear<Theme>
where
    Message: Clone,
    Theme: StyleSheet,
    Renderer: advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.girth,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.girth)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            if let Some(target) = self.progress {
                if state.progress.update(target, *now) {
                    shell.request_redraw();
                }
            } else {
                state.animation = state.animation.timed_transition(
                    self.cycle_duration,
                    self.period,
                    WRAP_LENGTH,
                    *now,
                );
                shell.request_redraw();
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let custom_style = theme.appearance(&self.style, self.progress.is_some(), false);
        let state = tree.state.downcast_ref::<State>();

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border {
                    width: if custom_style.border_color.is_some() {
                        1.0
                    } else {
                        0.0
                    },
                    color: custom_style.border_color.unwrap_or(custom_style.bar_color),
                    radius: custom_style.border_radius.into(),
                },
                snap: true,
                ..renderer::Quad::default()
            },
            Background::Color(custom_style.track_color),
        );

        let mut draw_segment = |x: f32, width: f32| {
            if width > 0.001 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + x * bounds.width,
                            y: bounds.y,
                            width: width * bounds.width,
                            height: bounds.height,
                        },
                        border: iced::Border {
                            width: 0.0,
                            color: iced::Color::TRANSPARENT,
                            radius: custom_style.border_radius.into(),
                        },
                        snap: true,
                        ..renderer::Quad::default()
                    },
                    Background::Color(custom_style.bar_color),
                );
            }
        };

        if self.progress.is_some() {
            draw_segment(0.0, state.progress.current);
        } else {
            let (bar_start, bar_end) =
                state
                    .animation
                    .bar_positions(self.cycle_duration, MIN_LENGTH, WRAP_LENGTH);
            let length = bar_end - bar_start;
            let start = bar_start % 1.0;
            let right_width = (1.0 - start).min(length);
            let left_width = length - right_width;

            draw_segment(start, right_width);
            draw_segment(0.0, left_width);
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Linear<Theme>> for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: StyleSheet + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(linear: Linear<Theme>) -> Self {
        Self::new(linear)
    }
}
