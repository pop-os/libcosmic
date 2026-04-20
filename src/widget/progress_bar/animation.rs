use crate::anim::smootherstep;
use iced::time::Instant;
use std::time::Duration;

const LAG: f32 = 0.1;

pub struct Progress {
    pub current: f32,
    last: Instant,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            current: 0.0,
            last: Instant::now(),
        }
    }
}

impl Progress {
    /// Smoothly chases `target` using exponential decay.
    /// Returns `true` if still animating and a redraw should be requested.
    pub fn update(&mut self, target: f32, now: Instant) -> bool {
        let dt = (now - self.last).as_secs_f32();
        self.last = now;
        let next = self.current + (target - self.current) * (1.0 - (-dt / LAG).exp());
        if (next - target).abs() > 0.001 {
            self.current = next;
            true
        } else {
            self.current = target;
            false
        }
    }
}

#[derive(Clone, Copy)]
pub struct Animation {
    expanding: bool,
    start: Instant,
    last: Instant,
    offset: u32,
}

impl Default for Animation {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            expanding: true,
            start: now,
            last: now,
            offset: 0,
        }
    }
}

impl Animation {
    pub fn timed_transition(
        &self,
        cycle_duration: Duration,
        period: Duration,
        wrap: f32,
        now: Instant,
    ) -> Self {
        let additional =
            ((now - self.last).as_secs_f32() / period.as_secs_f32() * u32::MAX as f32) as u32;
        let new_offset = self.offset.wrapping_add(additional);

        if !cycle_duration.is_zero() && now.duration_since(self.start) > cycle_duration {
            let offset = if self.expanding {
                new_offset
            } else {
                new_offset.wrapping_add((wrap * u32::MAX as f32) as u32)
            };
            Self {
                expanding: !self.expanding,
                start: now,
                last: now,
                offset,
            }
        } else {
            Self {
                last: now,
                offset: new_offset,
                ..*self
            }
        }
    }

    pub fn bar_positions(&self, cycle_duration: Duration, min: f32, wrap: f32) -> (f32, f32) {
        let offset = self.offset as f32 / u32::MAX as f32;
        let progress = if !cycle_duration.is_zero() {
            smootherstep(
                self.last.duration_since(self.start).as_secs_f32() / cycle_duration.as_secs_f32(),
            )
        } else {
            1.0
        };
        if self.expanding {
            (offset, offset + min + wrap * progress)
        } else {
            (offset + wrap * progress, offset + min + wrap)
        }
    }
}
