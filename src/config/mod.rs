// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Configurations available to libcosmic applications.

use crate::cosmic_theme::Density;
use cosmic_config::cosmic_config_derive::CosmicConfigEntry;
use cosmic_config::{Config, CosmicConfigEntry};
use iced::font::Family;
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, RwLock};
use ustr::Ustr;

/// ID for the `CosmicTk` config.
pub const ID: &str = "com.system76.CosmicTk";

pub static COSMIC_TK: LazyLock<RwLock<CosmicTk>> = LazyLock::new(|| {
    RwLock::new(
        CosmicTk::config()
            .map(|c| {
                CosmicTk::get_entry(&c).unwrap_or_else(|(errors, mode)| {
                    for why in errors {
                        tracing::error!(?why, "CosmicTk config entry error");
                    }
                    mode
                })
            })
            .unwrap_or_default(),
    )
});

/// Apply the theme to other toolkits.
#[allow(clippy::missing_panics_doc)]
pub fn apply_theme_global() -> bool {
    COSMIC_TK.read().unwrap().apply_theme_global
}

/// Show minimize button in window header.
#[allow(clippy::missing_panics_doc)]
pub fn show_minimize() -> bool {
    COSMIC_TK.read().unwrap().show_minimize
}

/// Show maximize button in window header.
#[allow(clippy::missing_panics_doc)]
pub fn show_maximize() -> bool {
    COSMIC_TK.read().unwrap().show_maximize
}

/// Preferred icon theme.
#[allow(clippy::missing_panics_doc)]
pub fn icon_theme() -> String {
    COSMIC_TK.read().unwrap().icon_theme.clone()
}

/// Density of CSD/SSD header bars.
#[allow(clippy::missing_panics_doc)]
pub fn header_size() -> Density {
    COSMIC_TK.read().unwrap().header_size
}

/// Interface density.
#[allow(clippy::missing_panics_doc)]
pub fn interface_density() -> Density {
    COSMIC_TK.read().unwrap().interface_density
}

#[allow(clippy::missing_panics_doc)]
pub fn interface_font() -> FontConfig {
    COSMIC_TK.read().unwrap().interface_font
}

#[allow(clippy::missing_panics_doc)]
pub fn monospace_font() -> FontConfig {
    COSMIC_TK.read().unwrap().monospace_font
}

#[derive(Clone, CosmicConfigEntry, Debug, Eq, PartialEq)]
#[version = 1]
pub struct CosmicTk {
    /// Apply the theme to other toolkits.
    pub apply_theme_global: bool,

    /// Show minimize button in window header.
    pub show_minimize: bool,

    /// Show maximize button in window header.
    pub show_maximize: bool,

    /// Preferred icon theme.
    pub icon_theme: String,

    /// Density of CSD/SSD header bars.
    pub header_size: Density,

    /// Interface density.
    pub interface_density: Density,

    /// Interface font family
    pub interface_font: FontConfig,

    /// Mono font family
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
                family: Ustr::from("Fira Sans"),
                weight: iced::font::Weight::Normal,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Normal,
            },
            monospace_font: FontConfig {
                family: Ustr::from("Fira Mono"),
                weight: iced::font::Weight::Normal,
                stretch: iced::font::Stretch::Normal,
                style: iced::font::Style::Normal,
            },
        }
    }
}

impl CosmicTk {
    pub fn config() -> Result<Config, cosmic_config::Error> {
        Config::new(ID, Self::VERSION)
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct FontConfig {
    pub family: Ustr,
    pub weight: iced::font::Weight,
    pub stretch: iced::font::Stretch,
    pub style: iced::font::Style,
}

impl From<FontConfig> for iced::Font {
    fn from(font: FontConfig) -> Self {
        Self {
            family: iced::font::Family::Name(font.family.as_str()),
            weight: font.weight,
            stretch: font.stretch,
            style: font.style,
        }
    }
}
