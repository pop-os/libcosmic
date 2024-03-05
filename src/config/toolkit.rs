// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Configurations for the libcosmic toolkit.

use cosmic_config::cosmic_config_derive::CosmicConfigEntry;
use cosmic_config::{Config, CosmicConfigEntry};

/// ID for the `CosmicTk` config.
pub const ID: &str = "com.system76.CosmicTk";

#[derive(Clone, CosmicConfigEntry, Debug, Eq, PartialEq)]
#[version = 1]
pub struct CosmicTk {
    /// Show minimize button in window header.
    pub show_minimize: bool,

    /// Show maximize button in window header.
    pub show_maximize: bool,
}

impl Default for CosmicTk {
    fn default() -> Self {
        Self {
            show_minimize: true,
            show_maximize: true,
        }
    }
}

impl CosmicTk {
    pub fn config() -> Result<Config, cosmic_config::Error> {
        Config::new(ID, Self::VERSION)
    }
}
