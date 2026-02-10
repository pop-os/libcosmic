use palette::{Srgba, rgb::Rgba};
use thiserror::Error;

use crate::Theme;

/// Module for outputting the Cosmic gtk4 theme type as CSS
pub mod gtk4_output;

/// Module for outputting the Cosmic qt theme type as kdeglobals
pub mod qt_output;

pub mod vs_code;

#[derive(Error, Debug)]
pub enum OutputError {
    #[error("IO Error: {0}")]
    Io(std::io::Error),
    #[error("Missing config directory")]
    MissingConfigDir,
    #[error("Missing data directory")]
    MissingDataDir,
    #[error("Serde Error: {0}")]
    Serde(#[from] serde_json::Error),
}

impl Theme {
    #[inline]
    pub fn apply_exports(&self) -> Result<(), OutputError> {
        let gtk_res = Theme::apply_gtk(self.is_dark);
        let qt_res = Theme::apply_qt(self.is_dark);
        let vs_res = self.clone().apply_vs_code();
        gtk_res?;
        qt_res?;
        vs_res?;
        Ok(())
    }

    #[inline]
    /// To avoid rewriting too much code, I replaced calls to `Theme::apply_gtk` with this.
    /// Note that vscode isn't touched by this function.
    pub fn apply_exports_static(is_dark: bool) -> Result<(), OutputError> {
        let gtk_res = Theme::apply_gtk(is_dark);
        let qt_res = Theme::apply_qt(is_dark);
        gtk_res?;
        qt_res?;
        Ok(())
    }

    #[inline]
    pub fn write_exports(&self) -> Result<(), OutputError> {
        let gtk_res = self.write_gtk4();
        let qt_res = self.write_qt();
        gtk_res?;
        qt_res?;
        Ok(())
    }

    #[inline]
    pub fn reset_exports() -> Result<(), OutputError> {
        let gtk_res = Theme::reset_gtk();
        let qt_res = Theme::reset_qt();
        let vs_res = Theme::reset_vs_code();
        gtk_res?;
        qt_res?;
        vs_res?;
        Ok(())
    }
}

pub fn to_hex(c: Srgba) -> String {
    let c_u8: Rgba<palette::encoding::Srgb, u8> = c.into_format();
    format!(
        "{:02x}{:02x}{:02x}{:02x}",
        c_u8.red, c_u8.green, c_u8.blue, c_u8.alpha
    )
}

pub fn to_rgba(c: Srgba) -> String {
    let c_u8: Rgba<palette::encoding::Srgb, u8> = c.into_format();
    format!(
        "rgba({}, {}, {}, {:1.2})",
        c_u8.red, c_u8.green, c_u8.blue, c.alpha
    )
}
