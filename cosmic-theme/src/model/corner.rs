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
    fn from(value: CornerRadii) -> Self {
        if (value.radius_m[0] - 16.0).abs() < 0.01 {
            Self::Round
        } else if (value.radius_m[0] - 8.0).abs() < 0.01 {
            Self::SlightlyRound
        } else {
            Self::Square
        }
    }
}
