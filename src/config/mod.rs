// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Configurations available to libcosmic applications.

use crate::cosmic_theme::Density;
use cosmic_config::cosmic_config_derive::CosmicConfigEntry;
use cosmic_config::{Config, CosmicConfigEntry};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::{Arc, LazyLock, Mutex, PoisonError, RwLock, RwLockReadGuard};

/// ID for the `CosmicTk` config.
pub const ID: &str = "com.system76.CosmicTk";

const MONO_FAMILY_DEFAULT: &str = "Noto Sans Mono";
const SANS_FAMILY_DEFAULT: &str = "Open Sans";

// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Configurations available to libcosmic applications.

use crate::cosmic_theme::Density;
use cosmic_config::cosmic_config_derive::CosmicConfigEntry;
use cosmic_config::{Config, CosmicConfigEntry};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::{LazyLock, Mutex, RwLock, RwLockReadGuard, Arc, PoisonError};

/// ID for the `CosmicTk` config.
pub const ID: &str = "com.system76.CosmicTk";

const MONO_FAMILY_DEFAULT: &str = "Noto Sans Mono";
const SANS_FAMILY_DEFAULT: &str = "Open Sans";

/// Stores static strings of the family names for `iced::Font` compatibility.
pub static FAMILY_MAP: LazyLock<Mutex<BTreeSet<Arc<str>>>> =
    LazyLock::new(|| Mutex::new(BTreeSet::new()));

pub static COSMIC_TK: LazyLock<RwLock<CosmicTk>> = LazyLock::new(|| {
    RwLock::new(
        CosmicTk::config().map(|c| {
            CosmicTk::get_entry(&c).unwrap_or_else(|(errors, mode)| {
                for why in errors.into_iter().filter(cosmic_config::Error::is_err) {
                    if let cosmic_config::Error::GetKey(_, err) = &why {
                                         if err.kind() == std::io::ErrorKind::NotFound {
                                             continue; // No system default config installed
                                         }
                                     }
                                     tracing::error!(?why, "CosmicTk config entry error");
                                 }
                                 mode
                             })
                         })
                         .unwrap_or_default(),
                 )
             });

/// Helper to handle poisoned locks and reduce repetitive `.read().unwrap()`.
fn cosmic() -> RwLockReadGuard<'static, CosmicTk> {
    COSMIC_TK.read().unwrap_or_else(|e| e.into_inner())
}

/// Apply the theme to other toolkits.
pub fn apply_theme_global() -> bool {
    cosmic().apply_theme_global
}

/// Show minimize button in window header.
pub fn show_minimize() -> bool {
    cosmic().show_minimize
}

/// Show maximize button in window header.
pub fn show_maximize() -> bool {
    cosmic().show_maximize
}

/// Preferred icon theme.
pub fn icon_theme() -> &str {
    &cosmic().icon_theme
}

/// Density of CSD/SSD header bars.
pub fn header_size() -> Density {
    cosmic().header_size
}

/// Interface density.
pub fn interface_density() -> Density {
    cosmic().interface_density
}

/// Interface font.
pub fn interface_font() -> &FontConfig {
    &cosmic().interface_font
}

/// Monospace font.
pub fn monospace_font() -> &FontConfig {
    &cosmic().monospace_font
}

#[derive(Clone, CosmicConfigEntry, Debug, Eq, PartialEq)]
#[version = 1]
pub struct CosmicTk {
    pub apply_theme_global: bool,
    pub show_minimize: bool,
    pub show_maximize: bool,
    pub icon_theme: String,
    pub header_size: Density,
    pub interface_density: Density,
    pub interface_font: FontConfig,
    pub monospace_font: FontConfig,
}

impl Default for CosmicTk {
    fn default() -> Self {
        Self {
            apply_theme_global: false,
            show_minimize: true,
            show_maximize: true,
            icon_theme: String::from("Cosmic"),
            header_size: Density::Standard,
            interface_density: Density::Standard,
            interface_font: FontConfig {
                family: SANS_FAMILY_DEFAULT.to_owned(),
                weight: iced::font::Weight::Normal,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Normal,
            },
            monospace_font: FontConfig {
                family: MONO_FAMILY_DEFAULT.to_owned(),
                weight: iced::font::Weight::Normal,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Normal,
            },
        }
    }
}

impl CosmicTk {
    #[inline]
    pub fn config() -> Result<Config, cosmic_config::Error> {
        Config::new(ID, Self::VERSION)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct FontConfig {
    pub family: String,
    pub weight: iced::font::Weight,
    pub stretch: iced::font::Stretch,
    pub style: iced::font::Style,
}

impl From<FontConfig> for iced::Font {
    fn from(font: FontConfig) -> Self {
        let mut family_map = FAMILY_MAP.lock().unwrap_or_else(|e| e.into_inner());

        let name: Arc<str> = family_map
            .get(&font.family.as_str().into())
            .cloned()
            .unwrap_or_else(|| {
                let value: Arc<str> = Arc::from(font.family.clone());
                family_map.insert(Arc::clone(&value));
                value
            });

        Self {
            family: iced::font::Family::Name(Box::leak(name.clone().into_boxed_str())),
            weight: font.weight,
            stretch: font.stretch,
            style: font.style,
        }
    }
}
