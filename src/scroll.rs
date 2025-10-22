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
}

impl DiscreteScrollState {
    pub fn reset(&mut self) {
        self.x.reset();
        self.y.reset();
    }

    // Accumulate delta with a timer
    pub fn update(&mut self, delta: ScrollDelta) -> DiscreteScrollDelta {
        let (x, y) = match delta {
            ScrollDelta::Pixels { x, y } => (x / SCROLL_PIXELS, y / SCROLL_PIXELS),
            ScrollDelta::Lines { x, y } => (x, y),
        };

        DiscreteScrollDelta {
            x: self.x.update(x),
            y: self.y.update(y),
        }
    }
}

/// Scroll over a single axis
#[derive(Debug, Default)]
struct Scroll(Option<(f32, Instant)>);

impl Scroll {
    fn reset(&mut self) {
        self.0 = None;
    }

    fn update(&mut self, delta: f32) -> isize {
        if delta == 0. {
            // If delta is 0, scroll is on other axis; clear accumulated scroll
            self.0 = None;
            0
        } else {
            let previous_scroll = if let Some((scroll, last_scroll_time)) = self.0 {
                if last_scroll_time.elapsed() > SCROLL_TIMEOUT {
                    0.
                } else {
                    scroll
                }
            } else {
                0.
            };

            let scroll = previous_scroll + delta;

            // Return integer part of scroll, and keep remainder
            self.0 = Some((scroll.fract(), Instant::now()));
            scroll.trunc() as isize
        }
    }
}
