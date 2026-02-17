use crate::Theme;
use configparser::ini::Ini;
use palette::{Mix, Srgba, blend::Compose};
use std::{
    fs::{self, File},
    io::{self, Write},
    path::{Path, PathBuf},
};

use super::OutputError;

impl Theme {
    /// Produces a color scheme ini file for Qt.
    ///
    /// Some high-level documentation for this file can be found at:
    /// https://web.archive.org/web/20250402234329/https://docs.kde.org/stable5/en/plasma-workspace/kcontrol/colors/
    #[must_use]
    #[cold]
    pub fn as_qt(&self) -> String {
        // Usually, disabled elements will have strongly reduced contrast and are often notably darker or lighter
        let disabled_color_effects = IniColorEffects {
            color: self.button.disabled,
            color_amount: 0.0,
            color_effect: ColorEffect::Desaturate,
            contrast_amount: 0.65,
            contrast_effect: ColorEffect::Fade,
            intensity_amount: 0.1,
            intensity_effect: IntensityEffect::Lighten,
        };
        // Usually, inactive elements will have reduced contrast (text fades slightly into the background) and may have slightly reduced intensity
        let inactive_color_effects = IniColorEffects {
            color: self.palette.gray_1,
            color_amount: 0.025,
            color_effect: ColorEffect::Tint,
            contrast_amount: 0.1,
            contrast_effect: ColorEffect::Tint,
            intensity_amount: 0.0,
            intensity_effect: IntensityEffect::Shade,
        };

        let bg = self.background.base;
        // the background container
        let view_colors = IniColors {
            background_alternate: bg.mix(self.accent.base, 0.05),
            background_normal: bg,
            decoration_focus: self.accent_text_color(),
            decoration_hover: self.accent_text_color(),
            foreground_active: self.accent_text_color(),
            foreground_inactive: self.background.on.mix(bg, 0.1),
            foreground_link: self.link_button.base,
            foreground_negative: self.destructive_text_color(),
            foreground_neutral: self.warning_text_color(),
            foreground_normal: self.background.on,
            foreground_positive: self.success_text_color(),
            foreground_visited: self.accent_text_color(),
        };
        // components inside the background container
        let window_colors = IniColors {
            background_alternate: self.background.component.base.mix(self.accent.base, 0.05),
            background_normal: self.background.component.base,
            ..view_colors
        };

        // selected text and items
        let selection_colors = {
            let selected = self.background.component.selected;
            let selected_text = self.background.component.selected_text;
            IniColors {
                background_alternate: selected.mix(bg, 0.5),
                background_normal: selected,
                decoration_focus: selected,
                decoration_hover: selected,
                foreground_active: selected_text,
                foreground_inactive: selected_text.mix(selected, 0.5),
                foreground_link: self.link_button.on,
                foreground_negative: self.destructive_color(),
                foreground_neutral: self.warning_color(),
                foreground_normal: selected_text,
                foreground_positive: self.success_color(),
                foreground_visited: self.accent_color(),
            }
        };

        let button_colors = IniColors {
            background_alternate: self.accent_button.base,
            background_normal: self.button.base,
            ..view_colors
        };

        // Complementary: Areas of applications with an alternative color scheme; usually with a dark background for light color schemes.
        let complementary_colors = {
            let dark = if self.is_dark {
                self.clone()
            } else {
                Self::get_active_with_brightness(false).unwrap_or_else(|_| self.clone())
            };
            IniColors {
                background_alternate: dark.accent.base,
                background_normal: dark.background.base,
                decoration_focus: dark.accent_text_color(),
                decoration_hover: dark.accent_text_color(),
                foreground_active: dark.accent_text_color(),
                foreground_inactive: dark.background.on.mix(dark.background.base, 0.1),
                foreground_link: dark.link_button.base,
                foreground_negative: dark.destructive_text_color(),
                foreground_neutral: dark.warning_text_color(),
                foreground_normal: dark.background.on,
                foreground_positive: dark.success_text_color(),
                foreground_visited: dark.accent_text_color(),
            }
        };

        // headers in cosmic don't have a background
        let header_colors = &view_colors;
        let header_colors_inactive = &view_colors;
        // tool tips, "What's This" tips, and similar elements
        let tooltip_colors = &window_colors;

        let general_color_scheme = if self.is_dark {
            "CosmicDark"
        } else {
            "CosmicLight"
        };
        let general_name = if self.is_dark {
            "COSMIC Dark"
        } else {
            "COSMIC Light"
        };
        // COSMIC icons are stuck in light mode, so use breeze icons instead
        let icons_theme = if self.is_dark {
            "breeze-dark"
        } else {
            "breeze"
        };

        format!(
            r#"# GENERATED BY COSMIC

[ColorEffects:Disabled]
{}

[ColorEffects:Inactive]
ChangeSelectionColor=false
Enable=false
{}

[Colors:Button]
{}

[Colors:Complementary]
{}

[Colors:Header]
{}

[Colors:Header][Inactive]
{}

[Colors:Selection]
{}

[Colors:Tooltip]
{}

[Colors:View]
{}

[Colors:Window]
{}

[General]
ColorScheme={general_color_scheme}
Name={general_name}
shadeSortColumn=true

[Icons]
Theme={icons_theme}

[KDE]
contrast=4
widgetStyle=qt6ct-style

[WM]
{}
"#,
            format_ini_color_effects(&disabled_color_effects, bg),
            format_ini_color_effects(&inactive_color_effects, bg),
            format_ini_colors(&button_colors, bg),
            format_ini_colors(&complementary_colors, bg),
            format_ini_colors(&header_colors, bg),
            format_ini_colors(&header_colors_inactive, bg),
            format_ini_colors(&selection_colors, bg),
            format_ini_colors(&tooltip_colors, bg),
            format_ini_colors(&view_colors, bg),
            format_ini_colors(&window_colors, bg),
            format_ini_wm_colors(&view_colors, self.is_dark),
        )
    }

    /// Write the color scheme to the appropriate directory.
    /// Should be written in `~/.local/share/color-schemes/`.
    ///
    /// See the docs: https://develop.kde.org/docs/plasma/#color-scheme
    ///
    /// # Errors
    ///
    /// Returns an `OutputError` if there is an error writing the colors file.
    #[cold]
    pub fn write_qt(&self) -> Result<(), OutputError> {
        let colors = self.as_qt();
        let file_path = Self::get_qt_colors_path(self.is_dark)?;
        let tmp_file_path = file_path.with_extension("colors.new");

        // Write to tmp_file_path first, then move it to file_path
        let mut tmp_file = File::create(&tmp_file_path).map_err(OutputError::Io)?;
        let res = tmp_file
            .write_all(colors.as_bytes())
            .and_then(|_| tmp_file.flush())
            .and_then(|_| std::fs::rename(&tmp_file_path, file_path));
        if let Err(e) = res {
            _ = std::fs::remove_file(&tmp_file_path);
            return Err(OutputError::Io(e));
        }

        Ok(())
    }

    /// Apply the color scheme by copying its values to `~/.config/kdeglobals`.
    ///
    /// See the docs: https://develop.kde.org/docs/plasma/#color-scheme
    ///
    /// # Errors
    ///
    /// Returns an `OutputError` if there is an error applying the color scheme.
    #[cold]
    pub fn apply_qt(is_dark: bool) -> Result<(), OutputError> {
        let Some(config_dir) = dirs::config_dir() else {
            return Err(OutputError::MissingConfigDir);
        };
        let kdeglobals_file = config_dir.join("kdeglobals");
        let mut kdeglobals_ini = Self::read_ini(&kdeglobals_file)?;

        let src_file = Self::get_qt_colors_path(is_dark)?;
        let src_ini = Self::read_ini(&src_file)?;

        Self::backup_non_cosmic_kdeglobals(&kdeglobals_file).map_err(OutputError::Io)?;

        for (section, key_value) in src_ini.get_map_ref() {
            for (key, value) in key_value {
                kdeglobals_ini.set(section, key, value.clone());
            }
        }

        kdeglobals_ini.write(kdeglobals_file).map_err(OutputError::Io)?;
        Ok(())
    }

    /// Reset the applied qt colors by deleting the `~/.config/kdeglobals` file.
    ///
    /// This does not restore the backed up kdeglobals file.
    ///
    /// # Errors
    ///
    /// Returns an `OutputError` if there is an error resetting the CSS file.
    #[cold]
    pub fn reset_qt() -> Result<(), OutputError> {
        let Some(config_dir) = dirs::config_dir() else {
            return Err(OutputError::MissingConfigDir);
        };
        let dest_file = config_dir.join("kdeglobals");

        if Self::is_cosmic_kdeglobals(&dest_file)
            .map_err(OutputError::Io)?
            .unwrap_or_default()
        {
            fs::remove_file(dest_file).map_err(OutputError::Io)?
        }
        Ok(())
    }

    /// Gets a path like `~/.config/color-schemes/CosmicDark.colors`
    pub fn get_qt_colors_path(is_dark: bool) -> Result<PathBuf, OutputError> {
        let Some(mut data_dir) = dirs::data_dir() else {
            return Err(OutputError::MissingDataDir);
        };

        let file_name = if is_dark {
            "CosmicDark.colors"
        } else {
            "CosmicLight.colors"
        };

        data_dir.push("color-schemes");
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir).map_err(OutputError::Io)?;
        }

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
    fn backup_non_cosmic_kdeglobals(path: &Path) -> io::Result<()> {
        if !Self::is_cosmic_kdeglobals(path)?.unwrap_or(true) {
            let backup_path = path.with_extension("bak");
            fs::rename(path, &backup_path)?;
        }
        Ok(())
    }

    #[cold]
    fn is_cosmic_kdeglobals(path: &Path) -> io::Result<Option<bool>> {
        if !path.exists() {
            return Ok(None);
        }

        if let Ok(metadata) = fs::symlink_metadata(path) {
            if !metadata.file_type().is_symlink() {
                return Ok(Some(false));
            }
            let Ok(target) = fs::read_link(path) else {
                return Ok(Some(false));
            };
            let Some(target_filename) = target.file_name() else {
                return Ok(Some(false));
            };
            return Ok(Some(
                target_filename == "CosmicDark.colors" || target_filename == "CosmicLight.colors",
            ));
        }
        Ok(Some(false))
    }
}

/// Formats a color in the form `r,g,b` e.g. `255,255,255`.
/// If the color has transparency, it is mixed with bg first.
fn to_rgb(c: Srgba, bg: Srgba) -> String {
    let c_u8: Srgba<u8> = c.over(bg).into_format();
    format!("{},{},{}", c_u8.red, c_u8.green, c_u8.blue)
}

fn format_ini_color_effects(color_effects: &IniColorEffects, bg: Srgba) -> String {
    format!(
        r#"Color={}
ColorAmount={}
ColorEffect={}
ContrastAmount={}
ContrastEffect={}
IntensityAmount={}
IntensityEffect={}"#,
        to_rgb(color_effects.color, bg),
        color_effects.color_amount,
        color_effects.color_effect.as_u8(),
        color_effects.contrast_amount,
        color_effects.contrast_effect.as_u8(),
        color_effects.intensity_amount,
        color_effects.intensity_effect.as_u8(),
    )
}

fn format_ini_colors(colors: &IniColors, bg: Srgba) -> String {
    format!(
        r#"BackgroundAlternate={}
BackgroundNormal={}
DecorationFocus={}
DecorationHover={}
ForegroundActive={}
ForegroundInactive={}
ForegroundLink={}
ForegroundNegative={}
ForegroundNeutral={}
ForegroundNormal={}
ForegroundPositive={}
ForegroundVisited={}"#,
        to_rgb(colors.background_alternate, bg),
        to_rgb(colors.background_normal, bg),
        to_rgb(colors.decoration_focus, bg),
        to_rgb(colors.decoration_hover, bg),
        to_rgb(colors.foreground_active, bg),
        to_rgb(colors.foreground_inactive, bg),
        to_rgb(colors.foreground_link, bg),
        to_rgb(colors.foreground_negative, bg),
        to_rgb(colors.foreground_neutral, bg),
        to_rgb(colors.foreground_normal, bg),
        to_rgb(colors.foreground_positive, bg),
        to_rgb(colors.foreground_visited, bg),
    )
}

/// Sets the colors for the titlebars of active and inactive windows.
fn format_ini_wm_colors(view_colors: &IniColors, is_dark: bool) -> String {
    let bg = view_colors.background_normal;
    let fg = view_colors.foreground_active;
    let blend = if is_dark { fg } else { bg };

    format!(
        r#"activeBackground={}
activeBlend={}
activeForeground={}
inactiveBackground={}
inactiveBlend={}
inactiveForeground={}"#,
        to_rgb(bg, bg),
        to_rgb(blend, bg),
        to_rgb(fg, bg),
        to_rgb(bg, bg),
        to_rgb(blend, bg),
        to_rgb(fg, bg),
    )
}

struct IniColorEffects {
    color: Srgba,
    color_amount: f32,
    color_effect: ColorEffect,
    contrast_amount: f32,
    /// Applied to the text, using the background as the reference color.
    contrast_effect: ColorEffect,
    intensity_amount: f32,
    intensity_effect: IntensityEffect,
}
/// Each color set is made up of a number of roles which are available in all other sets.
/// In addition, except for Inactive Text, there is a corresponding background role for each of the text roles. Currently (except for Normal and Alternate Background), these colors are not chosen here but are automatically determined based on Normal Background and the corresponding Text color.
struct IniColors {
    /// used when there is a need to subtly change the background to aid in item association. This might be used e.g. as the background of a heading, but is mostly used for alternating rows in lists, especially multi-column lists, to aid in visually tracking rows.
    background_alternate: Srgba,
    /// Normal background
    background_normal: Srgba,
    /// Used for drawing lines or shading UI elements to indicate the item which has active input focus.
    /// Typically the same as foreground_active.
    decoration_focus: Srgba,
    /// Used for drawing lines or shading UI elements for mouse-over effects, e.g. the "illumination" effects for buttons.
    /// Typically the same as foreground_active.
    decoration_hover: Srgba,
    /// used to indicate an active element or attract attention, e.g. alerts, notifications; also for hovered hyperlinks
    foreground_active: Srgba,
    /// used for text which should be unobtrusive, e.g. comments, "subtitles", unimportant information, etc.
    foreground_inactive: Srgba,
    /// used for hyperlinks or to otherwise indicate "something which may be visited", or to show relationships
    foreground_link: Srgba,
    /// used for errors, failure notices, notifications that an action may be dangerous (e.g. unsafe web page or security context), etc.
    foreground_negative: Srgba,
    /// used to draw attention when another role is not appropriate; e.g. warnings, to indicate secure/encrypted content, etc.
    foreground_neutral: Srgba,
    /// Normal foreground
    foreground_normal: Srgba,
    /// used for success notices, to indicate trusted content, etc.
    foreground_positive: Srgba,
    /// used for "something (e.g. a hyperlink) that has been visited", or to indicate something that is "old".
    foreground_visited: Srgba,
}

/// Intensity allows the overall color to be lightened or darkened.
#[allow(dead_code)]
enum IntensityEffect {
    /// Makes everything lighter or darker in a controlled manner.
    ///
    /// intensity_amount increases or decreases the overall intensity (i.e. perceived brightness) by an absolute amount.
    Shade,
    /// Changes the intensity to a percentage of the initial value.
    Darken,
    /// Conceptually the opposite of darken; lighten can be thought of as working with "distance from white", where darken works with "distance from black".
    Lighten,
}

impl IntensityEffect {
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Shade => 0,
            Self::Darken => 1,
            Self::Lighten => 2,
        }
    }
}

/// This also changes the overall color like [IntensityEffect],
/// but is not limited to intensity.
#[allow(dead_code)]
enum ColorEffect {
    /// changes the relative chroma
    ///
    /// This is available for "ColorEffect" but not "ContrastEffect".
    Desaturate,
    /// smoothly blends the original color into a reference color
    Fade,
    /// similar to Fade, except that the color (hue and chroma) changes more quickly while the intensity changes more slowly as the amount is increased
    Tint,
}

impl ColorEffect {
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::Desaturate => 0,
            Self::Fade => 1,
            Self::Tint => 2,
        }
    }
}
