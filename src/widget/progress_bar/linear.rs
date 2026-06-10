//! Show a linear progress indicator.
use super::animation::{Animation, Progress};
use super::style::StyleSheet;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{self, Clipboard, Layout, Shell, Widget, layout, renderer};
use iced::{Element, Event, Length, Pixels, Rectangle, Size, mouse, window};

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
            segment_spacing: 1.0,
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
    /// Each marker is a value between `0.0` and `1.0` that defines the position of a visual gap.
    pub fn markers(mut self, markers: impl Into<Vec<f32>>) -> Self {
        let mut markers = markers.into();
        for marker in &mut markers {
            *marker = marker.clamp(0.0, 1.0);
        }
        markers.sort_by(f32::total_cmp);
        markers.dedup();

        self.markers = markers;
        self
    }

    /// Sets the spacing between segments at each marker.
    pub fn segment_spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.segment_spacing = spacing.into().0.max(1.0);
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

        let mut draw_quad = |x: f32,
                             width: f32,
                             color: iced::Color,
                             mut border: iced::Border,
                             is_track: bool,
                             total_progress_width: f32| {
            let mut height = bounds.height;
            if !is_track {
                // For progress that is at the end of completion
                if total_progress_width > bounds.width - radius {
                    let border_radius =
                        radius.min(bounds.height / 2.0) - (bounds.width - total_progress_width);
                    border.radius.top_right = border_radius;
                    border.radius.bottom_right = border_radius;
                } else {
                    border.radius.top_right = 0.0;
                    border.radius.bottom_right = 0.0;
                }

                // For indeterminate mode or when progress has just started
                if x < radius.min(bounds.height / 2.0) {
                    let border_radius = radius.min(bounds.height / 2.0) - x;
                    border.radius.top_left = border_radius;
                    border.radius.bottom_left = border_radius;

                    if total_progress_width < radius.min(bounds.height / 2.0) {
                        height = bounds.height - 2.0 * radius.min(bounds.height / 2.0)
                            + total_progress_width * 2.0;
                    }
                } else {
                    border.radius.top_left = 0.0;
                    border.radius.bottom_left = 0.0;
                }

                if x > bounds.width - radius.min(bounds.height / 2.0) {
                    height = bounds.height - 2.0 * radius.min(bounds.height / 2.0) + width * 2.0;
                }
            }

            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x + x,
                        y: bounds.y + (bounds.height - height) / 2.0,
                        width,
                        height,
                    },
                    border,
                    snap: true,
                    ..renderer::Quad::default()
                },
                color,
            );
        };

        if self.progress.is_some() {
            let current_p = state.progress.current;
            let len = self.markers.len();
            let spacing = self.segment_spacing;
            let radius_inner = radius.min(spacing);

            let gap = if len != 0 {
                spacing / bounds.width
            } else {
                0.0
            };
            let drawable = 1.0 - gap * len as f32;

            let mut absolute_width = 0.0;
            for i in 0..=len {
                let (seg_lo, r_left) = if i == 0 {
                    (0.0, radius)
                } else {
                    (self.markers[i - 1], radius_inner)
                };
                let (seg_hi, r_right) = if i == len {
                    (1.0, radius)
                } else {
                    (self.markers[i], radius_inner)
                };
                let x_start = seg_lo * drawable + i as f32 * gap;
                let x_width = (seg_hi - seg_lo) * drawable;

                let mut segment_radius = if i == 0 && len == 0 {
                    [r_left, r_right, r_right, r_left].into()
                } else if i == 0 {
                    [r_left, 0.0, 0.0, r_left].into()
                } else if i == len {
                    [0.0, r_right, r_right, 0.0].into()
                } else {
                    [0.0, 0.0, 0.0, 0.0].into()
                };

                // draw track segment
                draw_quad(
                    x_start * bounds.width,
                    x_width * bounds.width,
                    custom_style.track_color,
                    iced::Border {
                        width: border_width,
                        color: border_color,
                        radius: segment_radius,
                    },
                    true,
                    bounds.width,
                );

                // draw bar segment
                if current_p > seg_lo {
                    let fill = ((current_p - seg_lo) / (seg_hi - seg_lo)).min(1.0);
                    absolute_width += x_width * fill + if i == 0 { 0.0 } else { gap };
                    segment_radius = [r_left, r_right, r_right, r_left].into();
                    draw_quad(
                        x_start * bounds.width,
                        x_width * fill * bounds.width,
                        custom_style.bar_color,
                        iced::Border {
                            radius: segment_radius,
                            ..iced::Border::default()
                        },
                        false,
                        absolute_width * bounds.width,
                    );
                }
            }
        } else {
            // draw track
            draw_quad(
                0.0,
                bounds.width,
                custom_style.track_color,
                iced::Border {
                    width: border_width,
                    color: border_color,
                    radius: radius.into(),
                },
                true,
                bounds.width,
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
            let border = iced::Border {
                radius: radius.into(),
                ..iced::Border::default()
            };

            draw_quad(
                start * bounds.width,
                right_width * bounds.width,
                custom_style.bar_color,
                border,
                false,
                (right_width + start) * bounds.width,
            );
            draw_quad(
                0.0,
                left_width * bounds.width,
                custom_style.bar_color,
                border,
                false,
                left_width * bounds.width,
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
