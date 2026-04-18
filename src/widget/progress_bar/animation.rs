use crate::anim::smootherstep;
use iced::time::Instant;
use std::time::Duration;

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
        rotation_duration: Duration,
        wrap: f32,
        now: Instant,
    ) -> Self {
        let additional = ((now - self.last).as_secs_f32() / rotation_duration.as_secs_f32()
            * u32::MAX as f32) as u32;
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
