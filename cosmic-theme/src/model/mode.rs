use cosmic_config::{Config, ConfigGet, CosmicConfigEntry};
use jiff::civil::Time;

/// ID for the ThemeMode config
pub const THEME_MODE_ID: &str = "com.system76.CosmicTheme.Mode";

/// The config for cosmic theme dark / light settings
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, cosmic_config::cosmic_config_derive::CosmicConfigEntry,
)]
#[version = 2]
pub struct ThemeMode {
    /// The theme dark mode setting.
    pub is_dark: bool,
    /// The theme auto-switch dark and light mode setting.
    pub auto_switch: bool,
    /// The theme will switch based on fixed timestamps.
    pub auto_switch_fixed_source: bool,
    pub auto_switch_dark_from: Option<Time>,
    pub auto_switch_dark_to: Option<Time>,
}

impl Default for ThemeMode {
    #[inline]
    fn default() -> Self {
        Self {
            is_dark: true,
            auto_switch: false,
            auto_switch_fixed_source: false,
            auto_switch_dark_from: None,
            auto_switch_dark_to: None,
        }
    }
}

impl ThemeMode {
    #[inline]
    /// Check if the theme is currently using dark mode
    pub fn is_dark(config: &Config) -> Result<bool, cosmic_config::Error> {
        config.get::<bool>("is_dark")
    }

    #[inline]
    /// The current version of the theme mode config.
    pub const fn version() -> u64 {
        Self::VERSION
    }

    #[inline]
    /// Get the config for the theme mode
    pub fn config() -> Result<Config, cosmic_config::Error> {
        Config::new(THEME_MODE_ID, Self::VERSION)
    }
}
