use cosmic_config::{Config, ConfigGet, ConfigSet, CosmicConfigEntry};

/// ID for the ThemeMode config
pub const THEME_MODE_ID: &str = "com.system76.CosmicTheme.Mode";

/// The config for cosmic theme dark / light settings
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, cosmic_config::cosmic_config_derive::CosmicConfigEntry,
)]
#[version = 1]
pub struct ThemeMode {
    /// The theme dark mode setting.
    pub is_dark: bool,
    /// The theme auto-switch dark and light mode setting.
    pub auto_switch: bool,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self {
            is_dark: true,
            auto_switch: false,
        }
    }
}

impl ThemeMode {
    /// Check if the theme is currently using dark mode
    pub fn is_dark(config: &Config) -> Result<bool, cosmic_config::Error> {
        config.get::<bool>("is_dark")
    }

    /// version of the theme
    pub fn version() -> u64 {
        1
    }

    /// Get the config for the theme mode
    pub fn config() -> Result<Config, cosmic_config::Error> {
        Config::new(THEME_MODE_ID, Self::version())
    }
}
