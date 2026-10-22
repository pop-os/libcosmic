// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Configurations available to libcosmic applications.

use crate::cosmic_theme::Density;
use cosmic_config::cosmic_config_derive::CosmicConfigEntry;
use cosmic_config::{Config, CosmicConfigEntry};
use std::sync::{LazyLock, RwLock};

/// ID for the `CosmicTk` config.
pub const ID: &str = "com.system76.CosmicTk";

pub static COSMIC_TK: LazyLock<RwLock<CosmicTk>> = LazyLock::new(|| {
    RwLock::new(
        CosmicTk::config()
            .map(|c| {
                CosmicTk::get_entry(&c).unwrap_or_else(|(errors, mode)| {
                    for why in errors.into_iter().filter(cosmic_config::Error::is_err) {
                        if let cosmic_config::Error::GetKey(_, err) = &why {
                            if err.kind() == std::io::ErrorKind::NotFound {
                                // No system default config installed; don't error
                                continue;
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

#[derive(Clone, CosmicConfigEntry, Debug, Eq, PartialEq)]
#[version = 1]
pub struct CosmicTk {
    /// Apply the theme to other toolkits.
    pub apply_theme_global: bool,

    /// Show minimize button in window header.
    pub show_minimize: bool,

    /// Show maximize button in window header.
    pub show_maximize: bool,

    /// Density of CSD/SSD header bars.
    pub header_size: Density,

    /// Interface density.
    pub interface_density: Density,
}

impl Default for CosmicTk {
    fn default() -> Self {
        Self {
            apply_theme_global: false,
            show_minimize: true,
            show_maximize: true,
            header_size: Density::Standard,
            interface_density: Density::Standard,
        }
    }
}

impl CosmicTk {
    #[inline]
    pub fn config() -> Result<Config, cosmic_config::Error> {
        Config::new(ID, Self::VERSION)
    }
}
