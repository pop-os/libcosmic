use crate::Theme;
use configparser::ini::Ini;
use std::{
    fs::{self, File},
    path::PathBuf,
};

use super::OutputError;

impl Theme {
    /// The "version" of this theme.
    ///
    /// To avoid repeatedly overwriting the user's config, we use a version system.
    ///
    /// Increment this value when changes to qt{5,6}ct.conf are needed.
    /// If the config's version is outdated, we update several sections.
    /// Otherwise, only the light/dark mode is updated.
    const COSMIC_QT_VERSION: u64 = 1;

    /// Edits qt{5,6}ct.conf to use COSMIC styles if needed.
    #[cold]
    pub fn apply_qt56ct(is_dark: bool) -> Result<(), OutputError> {
        let qt5ct_res = Self::apply_ct("qt5ct", is_dark);
        let qt6ct_res = Self::apply_ct("qt6ct", is_dark);
        qt5ct_res?;
        qt6ct_res?;
        Ok(())
    }
    #[must_use]
    #[cold]
    fn apply_ct(ct: &str, is_dark: bool) -> Result<(), OutputError> {
        let path = Self::get_conf_path(ct)?;
        let file_content = fs::read_to_string(&path).map_err(OutputError::Io)?;
        let mut ini = Ini::new_cs();
        ini.read(file_content).map_err(OutputError::Ini)?;

        let old_version = ini
            .getuint("Appearance", "cosmic_qt_version")
            .map_err(OutputError::Ini)?
            .unwrap_or_default();

        let color_scheme_path = Self::get_qt_colors_path(is_dark)?;
        let icon_theme = if is_dark { "breeze-dark" } else { "breeze" };

        ini.set(
            "Appearance",
            "cosmic_qt_version",
            Some(Theme::COSMIC_QT_VERSION.to_string()),
        );

        if old_version < Theme::COSMIC_QT_VERSION {
            // Config is outdated, update it unconditionally!

            ini.setstr(
                "Appearance",
                "color_scheme_path",
                color_scheme_path.to_str(),
            );
            // Enable the above color scheme, instead of using the default color scheme of e.g. Breeze
            ini.setstr("Appearance", "custom_palette", Some("true"));
            // COSMIC icons are stuck in light mode, so use breeze icons instead
            ini.setstr("Appearance", "icon_theme", Some(icon_theme));
            // Use COSMIC dialogs instead of KDE's
            ini.setstr("Appearance", "standard_dialogs", Some("xdgdesktopportal"));

            // TODO: Add fonts section to match COSMIC
        } else {
            // Config is not outdated, check before updating light/dark mode only!

            let old_color_scheme_path = ini
                .get("Appearance", "color_scheme_path")
                .unwrap_or_else(|| "CosmicPlease".to_owned());
            if old_color_scheme_path.contains("Cosmic") {
                ini.setstr(
                    "Appearance",
                    "color_scheme_path",
                    color_scheme_path.to_str(),
                );
            }

            let old_icon_theme = ini
                .get("Appearance", "icon_theme")
                .unwrap_or_else(|| "breeze".to_owned());
            if old_icon_theme.contains("breeze") {
                ini.setstr("Appearance", "icon_theme", Some(icon_theme));
            }
        }

        ini.write(path).map_err(OutputError::Io)?;
        Ok(())
    }

    /// Returns the file paths of the form `~/.config/ct/ct.conf`:
    /// e.g. `~/.config/qt6ct/qt6ct.conf`.
    ///
    /// The file and its parent directory are created if they don't exist.
    fn get_conf_path(ct: &str) -> Result<PathBuf, OutputError> {
        let Some(mut config_dir) = dirs::config_dir() else {
            return Err(OutputError::MissingConfigDir);
        };
        config_dir.push(&ct);
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(OutputError::Io)?;
        }

        let file_path = config_dir.join(ct.to_owned() + ".conf");
        if !file_path.exists() {
            File::create_new(&file_path).map_err(OutputError::Io)?;
        }

        Ok(file_path)
    }
}
