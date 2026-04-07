//! Show a circular progress indicator.
use super::style::StyleSheet;
use crate::anim::smootherstep;
use iced::advanced::layout;
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{self, Clipboard, Layout, Shell, Widget};
use iced::mouse;
use iced::time::Instant;
use iced::widget::canvas;
use iced::window;
use iced::{Element, Event, Length, Radians, Rectangle, Renderer, Size, Vector};

use std::f32::consts::PI;
use std::time::Duration;

const MIN_ANGLE: Radians = Radians(PI / 8.0);
const WRAP_ANGLE: Radians = Radians(2.0 * PI - PI / 4.0);
const BASE_ROTATION_SPEED: u32 = u32::MAX / 80;

#[must_use]
pub struct Circular<Theme>
where
    Theme: StyleSheet,
{
    size: f32,
    bar_height: f32,
    style: <Theme as StyleSheet>::Style,
    cycle_duration: Duration,
    rotation_duration: Duration,
    progress: Option<f32>,
}

impl<Theme> Circular<Theme>
where
    Theme: StyleSheet,
{
    /// Creates a new [`Circular`] with the given content.
    pub fn new() -> Self {
        Circular {
            size: 40.0,
            bar_height: 4.0,
            style: <Theme as StyleSheet>::Style::default(),
            cycle_duration: Duration::from_millis(1500),
            rotation_duration: Duration::from_secs(2),
            progress: None,
        }
    }

    /// Sets the size of the [`Circular`].
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets the bar height of the [`Circular`].
    pub fn bar_height(mut self, bar_height: f32) -> Self {
        self.bar_height = bar_height;
        self
    }

    /// Sets the style variant of this [`Circular`].
    pub fn style(mut self, style: <Theme as StyleSheet>::Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the cycle duration of this [`Circular`].
    pub fn cycle_duration(mut self, duration: Duration) -> Self {
        self.cycle_duration = duration / 2;
        self
    }

    /// Sets the base rotation duration of this [`Circular`]. This is the duration that a full
    /// rotation would take if the cycle rotation were set to 0.0 (no expanding or contracting)
    pub fn rotation_duration(mut self, duration: Duration) -> Self {
        self.rotation_duration = duration;
        self
    }

    /// Override the default behavior by providing a determinate progress value between `0.0` and `1.0`.
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = Some(progress.clamp(0.0, 1.0));
        self
    }
}

impl<Theme> Default for Circular<Theme>
where
    Theme: StyleSheet,
{
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Copy)]
enum Animation {
    Expanding {
        start: Instant,
        progress: f32,
        rotation: u32,
        last: Instant,
    },
    Contracting {
        start: Instant,
        progress: f32,
        rotation: u32,
        last: Instant,
    },
}

impl Default for Animation {
    fn default() -> Self {
        Self::Expanding {
            start: Instant::now(),
            progress: 0.0,
            rotation: 0,
            last: Instant::now(),
        }
    }
}

impl Animation {
    fn next(&self, additional_rotation: u32, now: Instant) -> Self {
        match self {
            Self::Expanding { rotation, .. } => Self::Contracting {
                start: now,
                progress: 0.0,
                rotation: rotation.wrapping_add(additional_rotation),
                last: now,
            },
            Self::Contracting { rotation, .. } => Self::Expanding {
                start: now,
                progress: 0.0,
                rotation: rotation.wrapping_add(BASE_ROTATION_SPEED.wrapping_add(
                    (f64::from(WRAP_ANGLE / (2.0 * Radians::PI)) * f64::from(u32::MAX)) as u32,
                )),
                last: now,
            },
        }
    }

    fn start(&self) -> Instant {
        match self {
            Self::Expanding { start, .. } | Self::Contracting { start, .. } => *start,
        }
    }

    fn last(&self) -> Instant {
        match self {
            Self::Expanding { last, .. } | Self::Contracting { last, .. } => *last,
        }
    }

    fn timed_transition(
        &self,
        cycle_duration: Duration,
        rotation_duration: Duration,
        now: Instant,
    ) -> Self {
        let elapsed = now.duration_since(self.start());
        let additional_rotation = ((now - self.last()).as_secs_f32()
            / rotation_duration.as_secs_f32()
            * (u32::MAX) as f32) as u32;

        match elapsed {
            elapsed if elapsed > cycle_duration => self.next(additional_rotation, now),
            _ => self.with_elapsed(cycle_duration, additional_rotation, elapsed, now),
        }
    }

    fn with_elapsed(
        &self,
        cycle_duration: Duration,
        additional_rotation: u32,
        elapsed: Duration,
        now: Instant,
    ) -> Self {
        let progress = elapsed.as_secs_f32() / cycle_duration.as_secs_f32();
        match self {
            Self::Expanding {
                start, rotation, ..
            } => Self::Expanding {
                start: *start,
                progress,
                rotation: rotation.wrapping_add(additional_rotation),
                last: now,
            },
            Self::Contracting {
                start, rotation, ..
            } => Self::Contracting {
                start: *start,
                progress,
                rotation: rotation.wrapping_add(additional_rotation),
                last: now,
            },
        }
    }

    fn rotation(&self) -> f32 {
        match self {
            Self::Expanding { rotation, .. } | Self::Contracting { rotation, .. } => {
                *rotation as f32 / u32::MAX as f32
            }
        }
    }
}

#[derive(Default)]
struct State {
    animation: Animation,
    cache: canvas::Cache,
    progress: Option<f32>,
}

impl<Message, Theme> Widget<Message, Theme, Renderer> for Circular<Theme>
where
    Message: Clone,
    Theme: StyleSheet,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fixed(self.size),
            height: Length::Fixed(self.size),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.size, self.size)
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
        if self.progress.is_some() {
            if !float_cmp::approx_eq!(
                f32,
                state.progress.unwrap_or_default(),
                self.progress.unwrap_or_default()
            ) {
                state.progress = self.progress;
                state.cache.clear();
            }
            return;
        }
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            state.animation =
                state
                    .animation
                    .timed_transition(self.cycle_duration, self.rotation_duration, *now);

            state.cache.clear();
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
        use advanced::Renderer as _;

        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let custom_style =
            <Theme as StyleSheet>::appearance(theme, &self.style, self.progress.is_some(), true);

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let track_radius = frame.width() / 2.0 - self.bar_height;
            let track_path = canvas::Path::circle(frame.center(), track_radius);

            frame.stroke(
                &track_path,
                canvas::Stroke::default()
                    .with_color(custom_style.track_color)
                    .with_width(self.bar_height),
            );

            if let Some(progress) = self.progress {
                // outer border
                if let Some(border_color) = custom_style.border_color {
                    let border_path =
                        canvas::Path::circle(frame.center(), track_radius + self.bar_height / 2.0);

                    frame.stroke(
                        &border_path,
                        canvas::Stroke::default()
                            .with_color(border_color)
                            .with_width(1.0),
                    );
                }

                // inner border
                if let Some(border_color) = custom_style.border_color {
                    let border_path =
                        canvas::Path::circle(frame.center(), track_radius - self.bar_height / 2.0);

                    frame.stroke(
                        &border_path,
                        canvas::Stroke::default()
                            .with_color(border_color)
                            .with_width(1.0),
                    );
                }

                // bar
                let mut builder = canvas::path::Builder::new();

                builder.arc(canvas::path::Arc {
                    center: frame.center(),
                    radius: track_radius,
                    start_angle: Radians(-PI / 2.0),
                    end_angle: Radians(-PI / 2.0 + progress * 2.0 * PI),
                });

                let bar_path = builder.build();

                frame.stroke(
                    &bar_path,
                    canvas::Stroke::default()
                        .with_color(custom_style.bar_color)
                        .with_width(self.bar_height),
                );

                let mut builder = canvas::path::Builder::new();

                // get center of end of arc for rounded cap
                let end_angle = -PI / 2.0 + progress * 2.0 * PI;
                let end_center =
                    frame.center() + Vector::new(end_angle.cos(), end_angle.sin()) * track_radius;
                builder.arc(canvas::path::Arc {
                    center: end_center,
                    radius: self.bar_height / 2.0,
                    start_angle: Radians(end_angle),
                    end_angle: Radians(end_angle + PI),
                });

                // get center of start of arc for rounded cap
                let start_angle = -PI / 2.0;
                let start_center = frame.center()
                    + Vector::new(start_angle.cos(), start_angle.sin()) * track_radius;
                builder.arc(canvas::path::Arc {
                    center: start_center,
                    radius: self.bar_height / 2.0,
                    start_angle: Radians(start_angle - PI),
                    end_angle: Radians(start_angle),
                });

                let cap_path = builder.build();
                frame.fill(&cap_path, custom_style.bar_color);
            } else {
                let mut builder = canvas::path::Builder::new();

                let start = Radians(state.animation.rotation() * 2.0 * PI);
                let (start_angle, end_angle) = match state.animation {
                    Animation::Expanding { progress, .. } => (
                        start,
                        start + MIN_ANGLE + WRAP_ANGLE * (smootherstep(progress)),
                    ),
                    Animation::Contracting { progress, .. } => (
                        start + WRAP_ANGLE * (smootherstep(progress)),
                        start + MIN_ANGLE + WRAP_ANGLE,
                    ),
                };
                builder.arc(canvas::path::Arc {
                    center: frame.center(),
                    radius: track_radius,
                    start_angle,
                    end_angle,
                });

                let bar_path = builder.build();

                frame.stroke(
                    &bar_path,
                    canvas::Stroke::default()
                        .with_color(custom_style.bar_color)
                        .with_width(self.bar_height),
                );

                let mut builder = canvas::path::Builder::new();

                // get center of end of arc for rounded cap
                let end_center = frame.center()
                    + Vector::new(end_angle.0.cos(), end_angle.0.sin()) * track_radius;
                builder.arc(canvas::path::Arc {
                    center: end_center,
                    radius: self.bar_height / 2.0,
                    start_angle: Radians(end_angle.0),
                    end_angle: Radians(end_angle.0 + PI),
                });

                // get center of start of arc for rounded cap
                let start_center = frame.center()
                    + Vector::new(start_angle.0.cos(), start_angle.0.sin()) * track_radius;
                builder.arc(canvas::path::Arc {
                    center: start_center,
                    radius: self.bar_height / 2.0,
                    start_angle: Radians(start_angle.0 - PI),
                    end_angle: Radians(start_angle.0),
                });

                let cap_path = builder.build();
                frame.fill(&cap_path, custom_style.bar_color);
            }
        });

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            use iced::advanced::graphics::geometry::Renderer as _;

            renderer.draw_geometry(geometry);
        });
    }
}

impl<'a, Message, Theme> From<Circular<Theme>> for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: StyleSheet + 'a,
{
    fn from(circular: Circular<Theme>) -> Self {
        Self::new(circular)
    }
}
