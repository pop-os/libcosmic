// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Configurations available to libcosmic applications.

use cosmic_config::cosmic_config_derive::CosmicConfigEntry;
use cosmic_config::{Config, CosmicConfigEntry};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

/// ID for the `CosmicTk` config.
pub const ID: &str = "com.system76.CosmicTk";

thread_local! {
    pub static COSMIC_TK: RefCell<CosmicTk> = RefCell::new(CosmicTk::config()
    .map(|c| {
        CosmicTk::get_entry(&c).unwrap_or_else(|(errors, mode)| {
            for why in errors {
                tracing::error!(?why, "CosmicTk config entry error");
            }
            mode
        })
    })
    .unwrap_or_default())
}

/// Apply the theme to other toolkits.
pub fn apply_theme_global() -> bool {
    COSMIC_TK.with(|tk| tk.borrow().apply_theme_global)
}

/// Show minimize button in window header.
pub fn show_minimize() -> bool {
    COSMIC_TK.with(|tk| tk.borrow().show_minimize)
}

/// Show maximize button in window header.
pub fn show_maximize() -> bool {
    COSMIC_TK.with(|tk| tk.borrow().show_maximize)
}

/// Preferred icon theme.
pub fn icon_theme() -> String {
    COSMIC_TK.with(|tk| tk.borrow().icon_theme.clone())
}

/// Density of CSD/SSD header bars.
pub fn header_size() -> Density {
    COSMIC_TK.with(|tk| tk.borrow().header_size)
}

/// Interface density.
pub fn interface_density() -> Density {
    COSMIC_TK.with(|tk| tk.borrow().interface_density)
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
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub enum Density {
    Compact,
    #[default]
    Standard,
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
        }
    }
}

impl CosmicTk {
    pub fn config() -> Result<Config, cosmic_config::Error> {
        Config::new(ID, Self::VERSION)
    }
}
