use serde::{Deserialize, Serialize};

/// Corner radii variables for the Cosmic theme
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CornerRadii {
    /// corner radii of 0
    pub radius_0: [u16; 4],
    /// smallest size of corner radii that can be non-zero
    pub radius_xs: [u16; 4],
    /// small corner radii
    pub radius_s: [u16; 4],
    /// medium corner radii
    pub radius_m: [u16; 4],
    /// large corner radii
    pub radius_l: [u16; 4],
    /// extra large corner radii
    pub radius_xl: [u16; 4],
}

impl Default for CornerRadii {
    fn default() -> Self {
        Self {
            radius_0: [0; 4],
            radius_xs: [4; 4],
            radius_s: [8; 4],
            radius_m: [16; 4],
            radius_l: [32; 4],
            radius_xl: [160; 4],
        }
    }
}
