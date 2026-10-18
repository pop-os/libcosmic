use serde::{Deserialize, Serialize};

/// Corner radii variables for the Cosmic theme
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct CornerRadii {
    /// corner radii of 0
    pub radius_0: [f32; 4],
    /// smallest size of corner radii that can be non-zero
    pub radius_xs: [f32; 4],
    /// small corner radii
    pub radius_s: [f32; 4],
    /// medium corner radii
    pub radius_m: [f32; 4],
    /// large corner radii
    pub radius_l: [f32; 4],
    /// extra large corner radii
    pub radius_xl: [f32; 4],
}

impl Default for CornerRadii {
    fn default() -> Self {
        Self {
            radius_0: [0.0; 4],
            radius_xs: [4.0; 4],
            radius_s: [8.0; 4],
            radius_m: [16.0; 4],
            radius_l: [32.0; 4],
            radius_xl: [160.0; 4],
        }
    }
}

/// Roundness options for the Cosmic theme
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Roundness {
    /// Round style
    #[default]
    Round,
    /// Slightly round style
    SlightlyRound,
    /// Square style
    Square,
}

impl From<Roundness> for CornerRadii {
    fn from(value: Roundness) -> Self {
        match value {
            Roundness::Round => CornerRadii::default(),
            Roundness::SlightlyRound => CornerRadii {
                radius_0: [0.0; 4],
                radius_xs: [2.0; 4],
                radius_s: [8.0; 4],
                radius_m: [8.0; 4],
                radius_l: [8.0; 4],
                radius_xl: [8.0; 4],
            },
            Roundness::Square => CornerRadii {
                radius_0: [0.0; 4],
                radius_xs: [2.0; 4],
                radius_s: [2.0; 4],
                radius_m: [2.0; 4],
                radius_l: [2.0; 4],
                radius_xl: [2.0; 4],
            },
        }
    }
}

impl From<CornerRadii> for Roundness {
    /// Returns the preset closest to the given radii.
    fn from(value: CornerRadii) -> Self {
        let level = value.roundness_level();
        if level < 0.25 {
            Self::Square
        } else if level < 0.75 {
            Self::SlightlyRound
        } else {
            Self::Round
        }
    }
}

impl Roundness {
    /// Position of the preset on the continuous roundness scale.
    pub fn level(self) -> f32 {
        match self {
            Self::Square => 0.0,
            Self::SlightlyRound => 0.5,
            Self::Round => 1.0,
        }
    }
}

/// Non-zero radii (`xs`, `s`, `m`, `l`, `xl`) of the presets, in roundness order.
const ANCHORS: [[f32; 5]; 3] = [
    [2.0, 2.0, 2.0, 2.0, 2.0],
    [2.0, 8.0, 8.0, 8.0, 8.0],
    [4.0, 8.0, 16.0, 32.0, 160.0],
];

impl CornerRadii {
    /// Corner radii for a continuous roundness level.
    ///
    /// `0.0` is [`Roundness::Square`], `0.5` is [`Roundness::SlightlyRound`]
    /// and `1.0` is [`Roundness::Round`]; values in between are interpolated
    /// from the neighbouring presets. `radius_xl` is interpolated
    /// geometrically, since radii beyond half of an element's height have no
    /// visible effect and a linear ramp to 160 would spend most of the range
    /// on pill shapes.
    pub fn from_roundness_level(level: f32) -> Self {
        let level = if level.is_nan() {
            0.0
        } else {
            level.clamp(0.0, 1.0)
        };
        let (lo, hi, t) = if level <= 0.5 {
            (ANCHORS[0], ANCHORS[1], level / 0.5)
        } else {
            (ANCHORS[1], ANCHORS[2], (level - 0.5) / 0.5)
        };

        let round = |v: f32| (v * 100.0).round() / 100.0;
        let linear = |i: usize| [round(lo[i] + (hi[i] - lo[i]) * t); 4];
        let geometric = |i: usize| [round(lo[i] * (hi[i] / lo[i]).powf(t)); 4];

        Self {
            radius_0: [0.0; 4],
            radius_xs: linear(0),
            radius_s: linear(1),
            radius_m: linear(2),
            radius_l: linear(3),
            radius_xl: geometric(4),
        }
    }

    /// Best guess of the roundness level these radii were created from.
    ///
    /// Inverse of [`CornerRadii::from_roundness_level`]. It is derived from
    /// `radius_l`, which grows strictly across the whole scale.
    pub fn roundness_level(&self) -> f32 {
        let l = self.radius_l[0];
        let level = if l <= ANCHORS[1][3] {
            (l - ANCHORS[0][3]) / (ANCHORS[1][3] - ANCHORS[0][3]) * 0.5
        } else {
            0.5 + (l - ANCHORS[1][3]) / (ANCHORS[2][3] - ANCHORS[1][3]) * 0.5
        };

        if level.is_nan() {
            0.0
        } else {
            level.clamp(0.0, 1.0)
        }
    }

    /// The preset these radii match exactly, if any.
    pub fn preset(&self) -> Option<Roundness> {
        [
            Roundness::Square,
            Roundness::SlightlyRound,
            Roundness::Round,
        ]
        .into_iter()
        .find(|&preset| {
            let other = CornerRadii::from(preset);
            [
                (self.radius_0, other.radius_0),
                (self.radius_xs, other.radius_xs),
                (self.radius_s, other.radius_s),
                (self.radius_m, other.radius_m),
                (self.radius_l, other.radius_l),
                (self.radius_xl, other.radius_xl),
            ]
            .iter()
            .all(|(a, b)| a.iter().zip(b).all(|(a, b)| (a - b).abs() < 0.01))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{CornerRadii, Roundness};

    const PRESETS: [Roundness; 3] = [
        Roundness::Square,
        Roundness::SlightlyRound,
        Roundness::Round,
    ];

    fn levels() -> impl Iterator<Item = f32> {
        (0..=100).map(|i| i as f32 / 100.0)
    }

    fn radii(r: &CornerRadii) -> [f32; 5] {
        [
            r.radius_xs[0],
            r.radius_s[0],
            r.radius_m[0],
            r.radius_l[0],
            r.radius_xl[0],
        ]
    }

    #[test]
    fn presets_are_anchors() {
        for preset in PRESETS {
            let radii = CornerRadii::from_roundness_level(preset.level());
            assert_eq!(radii, CornerRadii::from(preset), "{preset:?}");
            assert_eq!(radii.preset(), Some(preset));
            assert_eq!(radii.roundness_level(), preset.level());
            assert_eq!(Roundness::from(radii), preset);
        }
    }

    #[test]
    fn radii_grow_monotonically() {
        let mut prev = radii(&CornerRadii::from_roundness_level(0.0));
        for level in levels().skip(1) {
            let next = radii(&CornerRadii::from_roundness_level(level));
            for (p, n) in prev.iter().zip(&next) {
                assert!(
                    n >= p,
                    "radius shrank at level {level}: {prev:?} -> {next:?}"
                );
            }
            prev = next;
        }
    }

    #[test]
    fn level_roundtrips() {
        for level in levels() {
            let guessed = CornerRadii::from_roundness_level(level).roundness_level();
            assert!((guessed - level).abs() < 0.005, "{level} -> {guessed}");
        }
    }

    #[test]
    fn intermediate_levels_are_custom() {
        let radii = CornerRadii::from_roundness_level(0.75);
        assert_eq!(radii.preset(), None);
        assert_eq!(radii.radius_m, [12.0; 4]);
        assert_eq!(radii.radius_l, [20.0; 4]);
        assert_eq!(radii.radius_xl, [35.78; 4]);
    }

    #[test]
    fn out_of_range_levels_are_clamped() {
        assert_eq!(
            CornerRadii::from_roundness_level(-1.0),
            Roundness::Square.into()
        );
        assert_eq!(
            CornerRadii::from_roundness_level(2.0),
            Roundness::Round.into()
        );
        assert_eq!(
            CornerRadii::from_roundness_level(f32::NAN),
            Roundness::Square.into()
        );
    }
}
