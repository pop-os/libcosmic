use crate::Theme;
use configparser::ini::Ini;
use std::path::PathBuf;
use std::{fs, io};

use super::OutputError;

impl Theme {
    /// Reset the applied qt colors:
    /// - Remove relevant keys from the `~/.config/kdeglobals` file
    /// - Delete `~/.local/share/color-schemes/CosmicDark.colors` and `CosmicLight.colors`
    ///
    /// This does not restore the backed up kdeglobals file.
    ///
    /// # Errors
    ///
    /// Returns an `OutputError` if there is an error resetting the kdeglobals
    /// file or deleting the color schemes.
    #[cold]
    pub fn reset_qt() -> Result<(), OutputError> {
        let Some(config_dir) = dirs::config_dir() else {
            return Err(OutputError::MissingConfigDir);
        };
        let kdeglobals_file = config_dir.join("kdeglobals");
        let mut kdeglobals_ini = Self::read_ini(&kdeglobals_file)?;

        if !Self::is_cosmic_kdeglobals(&kdeglobals_ini)
            .map_err(OutputError::Io)?
            .unwrap_or_default()
        {
            // Not a cosmic kdeglobals file, do nothing
            return Ok(());
        }

        let light_scheme = Self::get_kcolorscheme_path(false)?;
        let dark_scheme = Self::get_kcolorscheme_path(true)?;
        if light_scheme.exists() {
            let src_ini = Self::read_ini(&light_scheme)?;

            // Remove color scheme keys from kdeglobals
            for (section, key_value) in src_ini.get_map_ref() {
                for key in key_value.keys() {
                    kdeglobals_ini.remove_key(section, key);
                }
            }

            kdeglobals_ini
                .write(kdeglobals_file)
                .map_err(OutputError::Io)?;
        }

        // Delete now-unused kcolorscheme files
        let delete_light_res = fs::remove_file(&light_scheme).map_err(OutputError::Io);
        let delete_dark_res = fs::remove_file(&dark_scheme).map_err(OutputError::Io);
        delete_light_res?;
        delete_dark_res?;

        Ok(())
    }

    /// Gets a path like `~/.local/share/color-schemes/CosmicDark.colors`
    fn get_kcolorscheme_path(is_dark: bool) -> Result<PathBuf, OutputError> {
        let Some(mut data_dir) = dirs::data_dir() else {
            return Err(OutputError::MissingDataDir);
        };
        data_dir.push("color-schemes");

        let file_name = if is_dark {
            "CosmicDark.colors"
        } else {
            "CosmicLight.colors"
        };

        Ok(data_dir.join(file_name))
    }

    #[cold]
    fn read_ini(path: &PathBuf) -> Result<Ini, OutputError> {
        let mut ini = Ini::new_cs();
        if !path.exists() {
            return Ok(ini);
        }
        let file_content = fs::read_to_string(path).map_err(OutputError::Io)?;
        ini.read(file_content).map_err(OutputError::Ini)?;
        Ok(ini)
    }

    #[cold]
    fn is_cosmic_kdeglobals(ini: &Ini) -> io::Result<Option<bool>> {
        let color_scheme = ini.get("General", "ColorScheme");
        if let Some(color_scheme) = color_scheme {
            Ok(Some(
                color_scheme == "CosmicDark" || color_scheme == "CosmicLight",
            ))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_cosmic_kdeglobals_dark() -> io::Result<()> {
        let mut ini = Ini::new_cs();
        ini.setstr("General", "ColorScheme", Some("CosmicDark"));
        let is_cosmic = Theme::is_cosmic_kdeglobals(&ini)?;
        assert_eq!(is_cosmic, Some(true));
        Ok(())
    }

    #[test]
    fn test_is_cosmic_kdeglobals_light() -> io::Result<()> {
        let mut ini = Ini::new_cs();
        ini.setstr("General", "ColorScheme", Some("CosmicLight"));
        let is_cosmic = Theme::is_cosmic_kdeglobals(&ini)?;
        assert_eq!(is_cosmic, Some(true));
        Ok(())
    }

    #[test]
    fn test_is_cosmic_kdeglobals_breeze() -> io::Result<()> {
        let mut ini = Ini::new_cs();
        ini.setstr("General", "ColorScheme", Some("BreezeDark"));
        let is_cosmic = Theme::is_cosmic_kdeglobals(&ini)?;
        assert_eq!(is_cosmic, Some(false));
        Ok(())
    }

    #[test]
    fn test_is_cosmic_kdeglobals_blank() -> io::Result<()> {
        let mut ini = Ini::new_cs();
        ini.setstr("General", "ColorScheme", None);
        let is_cosmic = Theme::is_cosmic_kdeglobals(&ini)?;
        assert_eq!(is_cosmic, Some(false));
        Ok(())
    }
}
