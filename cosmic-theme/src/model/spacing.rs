use serde::{Deserialize, Serialize};

/// Spacing variables for the Cosmic theme
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Spacing {
    /// No spacing
    pub space_none: u16,
    /// smallest spacing that can be non-zero
    pub space_xxxs: u16,
    /// extra extra small spacing
    pub space_xxs: u16,
    /// extra small spacing
    pub space_xs: u16,
    /// small spacing
    pub space_s: u16,
    /// medium spacing
    pub space_m: u16,
    /// large spacing
    pub space_l: u16,
    /// extra large spacing
    pub space_xl: u16,
    /// extra extra large spacing
    pub space_xxl: u16,
    /// largest possible spacing
    pub space_xxxl: u16,
}

impl Default for Spacing {
    fn default() -> Self {
        Self {
            space_none: 0,
            space_xxxs: 4,
            space_xxs: 8,
            space_xs: 12,
            space_s: 16,
            space_m: 24,
            space_l: 32,
            space_xl: 48,
            space_xxl: 64,
            space_xxxl: 128,
        }
    }
}
