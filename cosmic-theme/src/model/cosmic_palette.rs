use palette::Srgba;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

/// built-in light palette
pub static LIGHT_PALETTE: LazyLock<CosmicPalette> =
    LazyLock::new(|| ron::from_str(include_str!("light.ron")).unwrap());

/// built-in dark palette
pub static DARK_PALETTE: LazyLock<CosmicPalette> =
    LazyLock::new(|| ron::from_str(include_str!("dark.ron")).unwrap());

/// Palette type
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum CosmicPalette {
    /// Dark mode
    Dark(CosmicPaletteInner),
    /// Light mode
    Light(CosmicPaletteInner),
    /// High contrast light mode
    HighContrastLight(CosmicPaletteInner),
    /// High contrast dark mode
    HighContrastDark(CosmicPaletteInner),
}

impl CosmicPalette {
    /// extract the inner palette
    #[inline]
    pub fn inner(self) -> CosmicPaletteInner {
        match self {
            CosmicPalette::Dark(p) => p,
            CosmicPalette::Light(p) => p,
            CosmicPalette::HighContrastLight(p) => p,
            CosmicPalette::HighContrastDark(p) => p,
        }
    }
}

impl AsMut<CosmicPaletteInner> for CosmicPalette {
    #[inline]
    fn as_mut(&mut self) -> &mut CosmicPaletteInner {
        match self {
            CosmicPalette::Dark(p) => p,
            CosmicPalette::Light(p) => p,
            CosmicPalette::HighContrastLight(p) => p,
            CosmicPalette::HighContrastDark(p) => p,
        }
    }
}

impl AsRef<CosmicPaletteInner> for CosmicPalette {
    #[inline]
    fn as_ref(&self) -> &CosmicPaletteInner {
        match self {
            CosmicPalette::Dark(p) => p,
            CosmicPalette::Light(p) => p,
            CosmicPalette::HighContrastLight(p) => p,
            CosmicPalette::HighContrastDark(p) => p,
        }
    }
}

impl CosmicPalette {
    /// check if the palette is dark
    #[inline]
    pub fn is_dark(&self) -> bool {
        match self {
            CosmicPalette::Dark(_) | CosmicPalette::HighContrastDark(_) => true,
            CosmicPalette::Light(_) | CosmicPalette::HighContrastLight(_) => false,
        }
    }

    /// check if the palette is high_contrast
    #[inline]
    pub fn is_high_contrast(&self) -> bool {
        match self {
            CosmicPalette::HighContrastLight(_) | CosmicPalette::HighContrastDark(_) => true,
            CosmicPalette::Light(_) | CosmicPalette::Dark(_) => false,
        }
    }
}

impl Default for CosmicPalette {
    #[inline]
    fn default() -> Self {
        CosmicPalette::Dark(Default::default())
    }
}

/// The palette for Cosmic Theme, from which all color properties are derived
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
pub struct CosmicPaletteInner {
    /// name of the palette
    pub name: String,

    /// Utility Colors
    /// Colors used for various points of emphasis in the UI.
    pub bright_red: Srgba,
    /// Colors used for various points of emphasis in the UI.
    pub bright_green: Srgba,
    /// Colors used for various points of emphasis in the UI.
    pub bright_orange: Srgba,

    /// Surface Grays
    /// Colors used for three levels of surfaces in the UI.
    pub gray_1: Srgba,
    /// Colors used for three levels of surfaces in the UI.
    pub gray_2: Srgba,

    /// System Neutrals
    /// A wider spread of dark colors for more general use.
    pub neutral_0: Srgba,
    /// A wider spread of dark colors for more general use.
    pub neutral_1: Srgba,
    /// A wider spread of dark colors for more general use.
    pub neutral_2: Srgba,
    /// A wider spread of dark colors for more general use.
    pub neutral_3: Srgba,
    /// A wider spread of dark colors for more general use.
    pub neutral_4: Srgba,
    /// A wider spread of dark colors for more general use.
    pub neutral_5: Srgba,
    /// A wider spread of dark colors for more general use.
    pub neutral_6: Srgba,
    /// A wider spread of dark colors for more general use.
    pub neutral_7: Srgba,
    /// A wider spread of dark colors for more general use.
    pub neutral_8: Srgba,
    /// A wider spread of dark colors for more general use.
    pub neutral_9: Srgba,
    /// A wider spread of dark colors for more general use.
    pub neutral_10: Srgba,

    /// Potential Accent Color Combos
    pub accent_blue: Srgba,
    /// Potential Accent Color Combos
    pub accent_indigo: Srgba,
    /// Potential Accent Color Combos
    pub accent_purple: Srgba,
    /// Potential Accent Color Combos
    pub accent_pink: Srgba,
    /// Potential Accent Color Combos
    pub accent_red: Srgba,
    /// Potential Accent Color Combos
    pub accent_orange: Srgba,
    /// Potential Accent Color Combos
    pub accent_yellow: Srgba,
    /// Potential Accent Color Combos
    pub accent_green: Srgba,
    /// Potential Accent Color Combos
    pub accent_warm_grey: Srgba,

    /// Extended Color Palette
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_warm_grey: Srgba,
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_orange: Srgba,
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_yellow: Srgba,
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_blue: Srgba,
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_purple: Srgba,
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_pink: Srgba,
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_indigo: Srgba,
}

impl CosmicPalette {
    /// name of the palette
    #[inline]
    pub fn name(&self) -> &str {
        match &self {
            CosmicPalette::Dark(p) => &p.name,
            CosmicPalette::Light(p) => &p.name,
            CosmicPalette::HighContrastLight(p) => &p.name,
            CosmicPalette::HighContrastDark(p) => &p.name,
        }
    }
}
