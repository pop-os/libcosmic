//! Show a linear progress indicator.
use iced::advanced::layout;
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{self, Clipboard, Layout, Shell, Widget};
use iced::mouse;
use iced::time::Instant;
use iced::window;
use iced::{Background, Element, Event, Length, Rectangle, Size};

use crate::anim::smootherstep;

use super::style::StyleSheet;

use std::time::Duration;

#[must_use]
pub struct Linear<Theme>
where
    Theme: StyleSheet,
{
    width: Length,
    girth: Length,
    style: Theme::Style,
    cycle_duration: Duration,
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

#[derive(Clone, Copy)]
enum State {
    Expanding { start: Instant, progress: f32 },
    Contracting { start: Instant, progress: f32 },
}

impl Default for State {
    fn default() -> Self {
        Self::Expanding {
            start: Instant::now(),
            progress: 0.0,
        }
    }
}

impl State {
    fn next(&self, now: Instant) -> Self {
        match self {
            Self::Expanding { .. } => Self::Contracting {
                start: now,
                progress: 0.0,
            },
            Self::Contracting { .. } => Self::Expanding {
                start: now,
                progress: 0.0,
            },
        }
    }

    fn start(&self) -> Instant {
        match self {
            Self::Expanding { start, .. } | Self::Contracting { start, .. } => *start,
        }
    }

    fn timed_transition(&self, cycle_duration: Duration, now: Instant) -> Self {
        let elapsed = now.duration_since(self.start());

        match elapsed {
            elapsed if elapsed > cycle_duration => self.next(now),
            _ => self.with_elapsed(cycle_duration, elapsed),
        }
    }

    fn with_elapsed(&self, cycle_duration: Duration, elapsed: Duration) -> Self {
        let progress = elapsed.as_secs_f32() / cycle_duration.as_secs_f32();
        match self {
            Self::Expanding { start, .. } => Self::Expanding {
                start: *start,
                progress,
            },
            Self::Contracting { start, .. } => Self::Contracting {
                start: *start,
                progress,
            },
        }
    }
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
        if self.progress.is_some() {
            return;
        }

        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            *state = state.timed_transition(self.cycle_duration, *now);

            shell.request_redraw();
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
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: bounds.height,
                },
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

        if let Some(progress) = self.progress {
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x,
                        y: bounds.y,
                        width: progress * bounds.width,
                        height: bounds.height,
                    },
                    border: iced::Border {
                        width: 0.,
                        color: iced::Color::TRANSPARENT,
                        radius: custom_style.border_radius.into(),
                    },
                    snap: true,
                    ..renderer::Quad::default()
                },
                Background::Color(custom_style.bar_color),
            );
        } else {
            match state {
                State::Expanding { progress, .. } => renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x,
                            y: bounds.y,
                            width: smootherstep(*progress) * bounds.width,
                            height: bounds.height,
                        },
                        border: iced::Border {
                            width: 0.,
                            color: iced::Color::TRANSPARENT,
                            radius: custom_style.border_radius.into(),
                        },
                        snap: true,
                        ..renderer::Quad::default()
                    },
                    Background::Color(custom_style.bar_color),
                ),

                State::Contracting { progress, .. } => renderer.fill_quad(
                    Quad {
                        bounds: Rectangle {
                            x: bounds.x + smootherstep(*progress) * bounds.width,
                            y: bounds.y,
                            width: (1.0 - smootherstep(*progress)) * bounds.width,
                            height: bounds.height,
                        },
                        border: iced::Border {
                            width: 0.,
                            color: iced::Color::TRANSPARENT,
                            radius: custom_style.border_radius.into(),
                        },
                        snap: true,
                        ..renderer::Quad::default()
                    },
                    Background::Color(custom_style.bar_color),
                ),
            }
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
