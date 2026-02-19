use std::time::{Duration, Instant};

/// A simple linear interpolation calculation function.
/// p = `percent_complete` in decimal form
#[must_use]
pub fn lerp(start: f32, end: f32, p: f32) -> f32 {
    (1.0 - p) * start + p * end
}

/// A fast smooth interpolation calculation function.
/// p = `percent_complete` in decimal form
#[must_use]
pub fn slerp(start: f32, end: f32, p: f32) -> f32 {
    let t = smootherstep(p);
    (1.0 - t) * start + t * end
}

/// utility function which maps a value [0, 1] -> [0, 1] using the smootherstep function
pub fn smootherstep(t: f32) -> f32 {
    (6.0 * t.powi(5) - 15.0 * t.powi(4) + 10.0 * t.powi(3)).clamp(0.0, 1.0)
}

#[derive(Default, Debug)]
pub struct State {
    pub last_change: Option<Instant>,
}

impl State {
    pub fn changed(&mut self, dur: Duration) {
        let t = self.t(dur, false);
        let diff = dur.mul_f32(t.abs());
        let now = Instant::now();
        self.last_change = Some(now.checked_sub(diff).unwrap_or(now));
    }

    pub fn anim_done(&mut self, dur: Duration) {
        if self
            .last_change
            .is_some_and(|t| Instant::now().duration_since(t) > dur)
        {
            self.last_change = None;
        }
    }

    pub fn t(&self, dur: Duration, forward: bool) -> f32 {
        let res = self.last_change.map_or(1., |t| {
            Instant::now().duration_since(t).as_millis() as f32 / dur.as_millis() as f32
        });
        if forward { res } else { 1. - res }
    }
}
