use iced::Task;
use iced::mouse::ScrollDelta;
use std::time::{Duration, Instant};

// Number of scroll pixels before changing workspace
const SCROLL_PIXELS: f32 = 24.0;

// Timeout for scroll accumulation; older partial scroll is dropped
const SCROLL_TIMEOUT: Duration = Duration::from_millis(100);

/// A scroll delta with discrete integer deltas
#[derive(Debug, Default, Clone, Copy)]
pub struct DiscreteScrollDelta {
    pub x: isize,
    pub y: isize,
}

/// Helper for accumulating and converting pixel/line scrolls into and integer
/// delta between discrete options.
#[derive(Debug, Default)]
pub struct DiscreteScrollState {
    x: Scroll,
    y: Scroll,
    rate_limit: Option<Duration>,
}

impl DiscreteScrollState {
    /// Set a rate limit. If set, a call to `update()` will only not produce
    /// values other than 1, -1, or 0 and a non-zero return value will not
    /// occur more frequently than this duration.
    pub fn rate_limit(mut self, rate_limit: Option<Duration>) -> Self {
        self.rate_limit = rate_limit;
        self
    }

    /// Reset, clearing any acculuated scroll events that haven't been
    /// converted to discrete events yet.
    pub fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
    }

    /// Accumulate delta with a timer
    pub fn update(&mut self, delta: ScrollDelta) -> DiscreteScrollDelta {
        let (x, y) = match delta {
            ScrollDelta::Pixels { x, y } => (x / SCROLL_PIXELS, y / SCROLL_PIXELS),
            ScrollDelta::Lines { x, y } => (x, y),
        };

        DiscreteScrollDelta {
            x: self.x.update(x, self.rate_limit),
            y: self.y.update(y, self.rate_limit),
        }
    }
}

/// Scroll over a single axis
#[derive(Debug, Default)]
struct Scroll {
    scroll: Option<(f32, Instant)>,
    last_discrete: Option<Instant>,
}

impl Scroll {
    fn reset(&mut self) {
        *self = Default::default();
    }

    fn update(&mut self, delta: f32, rate_limit: Option<Duration>) -> isize {
        if delta == 0. {
            // If delta is 0, scroll is on other axis; clear accumulated scroll
            self.reset();
            0
        } else {
            let previous_scroll = if let Some((scroll, last_scroll_time)) = self.scroll {
                if last_scroll_time.elapsed() > SCROLL_TIMEOUT {
                    0.
                } else {
                    scroll
                }
            } else {
                0.
            };

            let scroll = previous_scroll + delta;

            if self
                .last_discrete
                .is_some_and(|time| time.elapsed() < rate_limit.unwrap_or(Duration::ZERO))
            {
                // If rate limit is hit, continute accumulating, but don't return
                // a discrete event yet.
                self.scroll = Some((scroll, Instant::now()));
                0
            } else {
                // Return integer part of scroll, and keep remainder
                self.scroll = Some((scroll.fract(), Instant::now()));
                let mut discrete = scroll.trunc() as isize;
                if discrete != 0 {
                    self.last_discrete = Some(Instant::now());
                }
                if rate_limit.is_some() {
                    // If we are rate limiting, don't return multiple discrete events
                    // at once; drop extras.
                    discrete.signum()
                } else {
                    discrete
                }
            }
        }
    }
}
