//! Show a linear progress indicator.
use super::animation::{Animation, Progress};
use super::style::StyleSheet;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{self, Clipboard, Layout, Shell, Widget, layout, renderer};
use iced::{Background, Element, Event, Length, Pixels, Rectangle, Size, mouse, window};

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
    markers: Vec<f32>,
    segment_spacing: f32,
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
            markers: Vec::new(),
            segment_spacing: 0.0,
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

    /// Sets the markers of a determinate progress bar, which divide the bar into segments.
    /// Each value is a progress fraction between `0.0` and `1.0 at which a visual gap is inserted.
    pub fn markers(mut self, markers: impl Into<Vec<f32>>) -> Self {
        let mut markers = markers.into();
        for bp in &mut markers {
            *bp = bp.clamp(0.0, 1.0);
        }
        markers.sort_by(f32::total_cmp);
        markers.dedup();

        self.markers = markers;
        self
    }

    /// Sets the spacing between segments at each marker.
    pub fn segment_spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.segment_spacing = spacing.into().0;
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

        let border_width = if custom_style.border_color.is_some() {
            1.0
        } else {
            0.0
        };
        let border_color = custom_style.border_color.unwrap_or(custom_style.bar_color);
        let radius = custom_style.border_radius;

        let mut draw_quad = |x: f32, width: f32, color: iced::Color, border: iced::Border| {
            // don't draw if width is less than 0.1 pixels
            if width * bounds.width > 0.1 {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds: Rectangle {
                            x: bounds.x + x * bounds.width,
                            y: bounds.y,
                            width: width * bounds.width,
                            height: bounds.height,
                        },
                        border,
                        snap: true,
                        ..renderer::Quad::default()
                    },
                    Background::Color(color),
                );
            }
        };

        if self.progress.is_some() {
            let spacing = self.segment_spacing.max(1.0);
            let radius_inner = radius.min(spacing);

            let gap = if self.markers.is_empty() {
                0.0
            } else {
                spacing / bounds.width
            };
            let drawable = 1.0 - gap * self.markers.len() as f32;
            let num_segments = self.markers.len() + 1;

            let segment_bounds = |i: usize| {
                let seg_lo = if i == 0 { 0.0 } else { self.markers[i - 1] };
                let seg_hi = if i == num_segments - 1 {
                    1.0
                } else {
                    self.markers[i]
                };
                (seg_lo, seg_hi)
            };
            let get_radius = |i: usize| {
                let r_left = if i == 0 { radius } else { radius_inner };
                let r_right = if i == num_segments - 1 {
                    radius
                } else {
                    radius_inner
                };
                [r_left, r_right, r_right, r_left].into()
            };

            // draw track segments
            for i in 0..num_segments {
                let (seg_lo, seg_hi) = segment_bounds(i);
                let x_start = seg_lo * drawable + i as f32 * gap;
                let x_width = (seg_hi - seg_lo) * drawable;

                draw_quad(
                    x_start,
                    x_width,
                    custom_style.track_color,
                    iced::Border {
                        width: border_width,
                        color: border_color,
                        radius: get_radius(i),
                    },
                );
            }

            // draw bar segments
            let current_p = state.progress.current;
            for i in 0..num_segments {
                let (seg_lo, seg_hi) = segment_bounds(i);

                // don't iterate over non-filled segments
                if current_p < seg_lo {
                    break;
                }

                let x_start = seg_lo * drawable + i as f32 * gap;
                let x_width = (seg_hi - seg_lo) * drawable;
                let fill = ((current_p - seg_lo) / (seg_hi - seg_lo)).clamp(0.0, 1.0);

                draw_quad(
                    x_start,
                    x_width * fill,
                    custom_style.bar_color,
                    iced::Border {
                        radius: get_radius(i),
                        ..iced::Border::default()
                    },
                );
            }
        } else {
            // draw track
            draw_quad(
                0.0,
                1.0,
                custom_style.track_color,
                iced::Border {
                    width: border_width,
                    color: border_color,
                    radius: radius.into(),
                },
            );

            // draw bar
            let (bar_start, bar_end) =
                state
                    .animation
                    .bar_positions(self.cycle_duration, MIN_LENGTH, WRAP_LENGTH);
            let length = bar_end - bar_start;
            let start = bar_start % 1.0;
            let right_width = (1.0 - start).min(length);
            let left_width = length - right_width;

            draw_quad(
                start,
                right_width,
                custom_style.bar_color,
                iced::Border {
                    radius: radius.into(),
                    ..iced::Border::default()
                },
            );
            draw_quad(
                0.0,
                left_width,
                custom_style.bar_color,
                iced::Border {
                    radius: radius.into(),
                    ..iced::Border::default()
                },
            );
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
