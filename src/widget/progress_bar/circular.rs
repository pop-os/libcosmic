//! Show a circular progress indicator.
use super::animation::Animation;
use super::style::StyleSheet;
use iced::advanced::layout;
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{self, Clipboard, Layout, Shell, Widget};
use iced::mouse;
use iced::widget::canvas;
use iced::window;
use iced::{Element, Event, Length, Radians, Rectangle, Renderer, Size, Vector};

use std::f32::consts::PI;
use std::time::Duration;

const MIN_ANGLE: Radians = Radians(PI / 8.0);

#[must_use]
pub struct Circular<Theme>
where
    Theme: StyleSheet,
{
    size: f32,
    bar_height: f32,
    style: Theme::Style,
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
            style: Theme::Style::default(),
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
    pub fn style(mut self, style: Theme::Style) -> Self {
        self.style = style;
        self
    }

    /// Sets the cycle duration of this [`Circular`].
    pub fn cycle_duration(mut self, duration: Duration) -> Self {
        self.cycle_duration = duration / 2;
        self
    }

    /// Sets the base rotation duration of this [`Circular`]. This is the duration that a full
    /// rotation would take if the cycle duration were set to 0.0 (no expanding or contracting)
    pub fn rotation_duration(mut self, duration: Duration) -> Self {
        self.rotation_duration = duration;
        self
    }

    /// Override the default behavior by providing a determinate progress value between `0.0` and `1.0`.
    pub fn progress(mut self, progress: f32) -> Self {
        self.progress = Some(progress.clamp(0.0, 1.0));
        self
    }

    fn min_wrap(&self, track_radius: f32) -> (f32, f32) {
        let cap_angle = self.bar_height / track_radius;
        let gap = MIN_ANGLE.0.max(cap_angle);
        ((gap - cap_angle) / (2.0 * PI), 1.0 - gap / PI)
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
            if state.progress != self.progress {
                state.progress = self.progress;
                state.cache.clear();
            }
            return;
        }
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            let (_, wrap) = self.min_wrap(self.size / 2.0 - self.bar_height);
            state.animation = state.animation.timed_transition(
                self.cycle_duration,
                self.rotation_duration,
                wrap,
                *now,
            );
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
        let custom_style = Theme::appearance(theme, &self.style, self.progress.is_some(), true);

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let track_radius = frame.width() / 2.0 - self.bar_height;
            let track_path = canvas::Path::circle(frame.center(), track_radius);

            frame.stroke(
                &track_path,
                canvas::Stroke::default()
                    .with_color(custom_style.track_color)
                    .with_width(self.bar_height),
            );

            // Converts a track fraction to an angle in radians, with 0 being top of circle
            let to_angle = |t: f32| t * 2.0 * PI - PI / 2.0;

            let draw_cap = |frame: &mut canvas::Frame, t: f32, flip: bool| {
                let angle = to_angle(t);
                let center = frame.center() + Vector::new(angle.cos(), angle.sin()) * track_radius;
                let (start_angle, end_angle) = if flip {
                    (angle - PI, angle)
                } else {
                    (angle, angle + PI)
                };
                let mut builder = canvas::path::Builder::new();
                builder.arc(canvas::path::Arc {
                    center,
                    radius: self.bar_height / 2.0,
                    start_angle: Radians(start_angle),
                    end_angle: Radians(end_angle),
                });
                frame.fill(&builder.build(), custom_style.bar_color);
            };

            let draw_bar = |frame: &mut canvas::Frame, start: f32, end: f32| {
                let mut builder = canvas::path::Builder::new();
                builder.arc(canvas::path::Arc {
                    center: frame.center(),
                    radius: track_radius,
                    start_angle: Radians(to_angle(start)),
                    end_angle: Radians(to_angle(end)),
                });
                frame.stroke(
                    &builder.build(),
                    canvas::Stroke::default()
                        .with_color(custom_style.bar_color)
                        .with_width(self.bar_height),
                );
                draw_cap(frame, end, false);
                draw_cap(frame, start, true);
            };

            if let Some(progress) = self.progress {
                if let Some(border_color) = custom_style.border_color {
                    for radius_offset in [self.bar_height / 2.0, -(self.bar_height / 2.0)] {
                        let border_path =
                            canvas::Path::circle(frame.center(), track_radius + radius_offset);
                        frame.stroke(
                            &border_path,
                            canvas::Stroke::default()
                                .with_color(border_color)
                                .with_width(1.0),
                        );
                    }
                }
                draw_bar(frame, 0.0, progress);
            } else {
                let (min, wrap) = self.min_wrap(track_radius);
                let (start, end) = state
                    .animation
                    .bar_positions(self.cycle_duration, min, wrap);
                draw_bar(frame, start, end);
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
