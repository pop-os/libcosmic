use iced_core::font::Family;

pub use iced_core::font::{Font, Stretch as FontStretch, Style as FontStyle, Weight as FontWeight};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::{LazyLock, RwLock};

/// The default interface font family used by the theme.
pub const DEFAULT_FONT_FAMILY: &str = "Open Sans";

/// The default monospace font family used by the theme.
pub const DEFAULT_MONOSPACE_FONT_FAMILY: &str = "Noto Sans Mono";

/// Describes a font family and its style used by the theme.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(default)]
pub struct FontConfig {
    /// The name of the font family.
    pub family: String,
    /// The weight of the font.
    pub weight: FontWeight,
    /// The width of the font.
    pub stretch: FontStretch,
    /// The style of the font.
    pub style: FontStyle,
}

impl FontConfig {
    /// Create a font config from a family name with the default style.
    #[must_use]
    pub fn new(family: impl Into<String>) -> Self {
        Self {
            family: family.into(),
            weight: FontWeight::default(),
            stretch: FontStretch::default(),
            style: FontStyle::default(),
        }
    }

    /// Create a font config for the default monospace font family.
    #[must_use]
    pub fn monospace() -> Self {
        Self::new(DEFAULT_MONOSPACE_FONT_FAMILY)
    }
}

impl Default for FontConfig {
    fn default() -> Self {
        Self::new(DEFAULT_FONT_FAMILY)
    }
}

impl From<&str> for FontConfig {
    fn from(family: &str) -> Self {
        Self::new(family)
    }
}

impl From<String> for FontConfig {
    fn from(family: String) -> Self {
        Self::new(family)
    }
}

impl From<&FontConfig> for Font {
    fn from(font: &FontConfig) -> Self {
        /// Stores static strings of the family names for [`Font`] compatibility.
        static FAMILY_MAP: LazyLock<RwLock<BTreeSet<&'static str>>> =
            LazyLock::new(RwLock::default);

        let read_guard = FAMILY_MAP.read().unwrap();
        let name: Option<&'static str> = read_guard.get(font.family.as_str()).copied();
        drop(read_guard);

        let name = name.unwrap_or_else(|| {
            let value: &'static str = font.family.clone().leak();
            FAMILY_MAP.write().unwrap().insert(value);
            value
        });

        Self {
            family: Family::Name(name),
            weight: font.weight,
            stretch: font.stretch,
            style: font.style,
        }
    }
}

impl From<FontConfig> for Font {
    fn from(font: FontConfig) -> Self {
        Self::from(&font)
    }
}
