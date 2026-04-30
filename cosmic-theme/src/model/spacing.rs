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

/// Density options for the Cosmic theme
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Density {
    /// Lower padding/spacing of elements
    Compact,
    /// Standard padding/spacing of elements
    #[default]
    Standard,
    /// Higher padding/spacing of elements
    Spacious,
}

impl From<Density> for Spacing {
    fn from(value: Density) -> Self {
        match value {
            Density::Compact => Spacing {
                space_none: 0,
                space_xxxs: 4,
                space_xxs: 4,
                space_xs: 8,
                space_s: 8,
                space_m: 16,
                space_l: 24,
                space_xl: 32,
                space_xxl: 48,
                space_xxxl: 64,
            },
            Density::Standard => Spacing::default(),
            Density::Spacious => Spacing {
                space_none: 4,
                space_xxxs: 8,
                space_xxs: 12,
                space_xs: 16,
                space_s: 24,
                space_m: 32,
                space_l: 48,
                space_xl: 64,
                space_xxl: 128,
                space_xxxl: 160,
            },
        }
    }
}

impl From<Spacing> for Density {
    fn from(value: Spacing) -> Self {
        if value.space_m.saturating_sub(16) == 0 {
            Self::Compact
        } else if value.space_m.saturating_sub(24) == 0 {
            Self::Standard
        } else {
            Self::Spacious
        }
    }
}
