use std::{
    fmt,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use directories::{BaseDirsExt, ProjectDirsExt};
use lazy_static::lazy_static;
use palette::Srgba;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::{util::CssColor, NAME, PALETTE_DIR};

lazy_static! {
    /// built in light palette
    pub static ref LIGHT_PALETTE: CosmicPalette<CssColor> =
        ron::from_str(include_str!("light.ron")).unwrap();
    /// built in dark palette
    pub static ref DARK_PALETTE: CosmicPalette<CssColor> =
        ron::from_str(include_str!("dark.ron")).unwrap();
}

/// Palette type
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum CosmicPalette<C> {
    /// Dark mode
    Dark(CosmicPaletteInner<C>),
    /// Light mode
    Light(CosmicPaletteInner<C>),
    /// High contrast light mode
    HighContrastLight(CosmicPaletteInner<C>),
    /// High contrast dark mode
    HighContrastDark(CosmicPaletteInner<C>),
}

impl<C> AsRef<CosmicPaletteInner<C>> for CosmicPalette<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    fn as_ref(&self) -> &CosmicPaletteInner<C> {
        match self {
            CosmicPalette::Dark(p) => p,
            CosmicPalette::Light(p) => p,
            CosmicPalette::HighContrastLight(p) => p,
            CosmicPalette::HighContrastDark(p) => p,
        }
    }
}

impl<C> CosmicPalette<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    /// check if the palette is dark
    pub fn is_dark(&self) -> bool {
        match self {
            CosmicPalette::Dark(_) | CosmicPalette::HighContrastDark(_) => true,
            CosmicPalette::Light(_) | CosmicPalette::HighContrastLight(_) => false,
        }
    }

    /// check if the palette is high_contrast
    pub fn is_high_contrast(&self) -> bool {
        match self {
            CosmicPalette::HighContrastLight(_) | CosmicPalette::HighContrastDark(_) => true,
            CosmicPalette::Light(_) | CosmicPalette::Dark(_) => false,
        }
    }
}

impl<C> Default for CosmicPalette<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    fn default() -> Self {
        CosmicPalette::Dark(Default::default())
    }
}

/// The palette for Cosmic Theme, from which all color properties are derived
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct CosmicPaletteInner<C> {
    /// name of the palette
    pub name: String,

    /// basic palette
    /// blue: colors used for various points of emphasis in the UI
    pub blue: C,
    /// red: colors used for various points of emphasis in the UI
    pub red: C,
    /// green: colors used for various points of emphasis in the UI
    pub green: C,
    /// yellow: colors used for various points of emphasis in the UI
    pub yellow: C,

    /// surface grays
    /// colors used for three levels of surfaces in the UI
    pub gray_1: C,
    /// colors used for three levels of surfaces in the UI
    pub gray_2: C,
    /// colors used for three levels of surfaces in the UI
    pub gray_3: C,

    /// System Neutrals
    /// A wider spread of dark colors for more general use.
    pub neutral_1: C,
    /// A wider spread of dark colors for more general use.
    pub neutral_2: C,
    /// A wider spread of dark colors for more general use.
    pub neutral_3: C,
    /// A wider spread of dark colors for more general use.
    pub neutral_4: C,
    /// A wider spread of dark colors for more general use.
    pub neutral_5: C,
    /// A wider spread of dark colors for more general use.
    pub neutral_6: C,
    /// A wider spread of dark colors for more general use.
    pub neutral_7: C,
    /// A wider spread of dark colors for more general use.
    pub neutral_8: C,
    /// A wider spread of dark colors for more general use.
    pub neutral_9: C,
    /// A wider spread of dark colors for more general use.
    pub neutral_10: C,

    /// Extended Color Palette
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_warm_grey: C,
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_orange: C,
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_yellow: C,
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_blue: C,
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_purple: C,
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_pink: C,
    /// Colors used for themes, app icons, illustrations, and other brand purposes.
    pub ext_indigo: C,

    /// Potential Accent Color Combos
    pub accent_warm_grey: C,
    /// Potential Accent Color Combos
    pub accent_orange: C,
    /// Potential Accent Color Combos
    pub accent_yellow: C,
    /// Potential Accent Color Combos
    pub accent_purple: C,
    /// Potential Accent Color Combos
    pub accent_pink: C,
    /// Potential Accent Color Combos
    pub accent_indigo: C,
}

impl From<CosmicPaletteInner<CssColor>> for CosmicPaletteInner<Srgba> {
    fn from(p: CosmicPaletteInner<CssColor>) -> Self {
        CosmicPaletteInner {
            name: p.name,
            blue: p.blue.into(),
            red: p.red.into(),
            green: p.green.into(),
            yellow: p.yellow.into(),
            gray_1: p.gray_1.into(),
            gray_2: p.gray_2.into(),
            gray_3: p.gray_3.into(),
            neutral_1: p.neutral_1.into(),
            neutral_2: p.neutral_2.into(),
            neutral_3: p.neutral_3.into(),
            neutral_4: p.neutral_4.into(),
            neutral_5: p.neutral_5.into(),
            neutral_6: p.neutral_6.into(),
            neutral_7: p.neutral_7.into(),
            neutral_8: p.neutral_8.into(),
            neutral_9: p.neutral_9.into(),
            neutral_10: p.neutral_10.into(),
            ext_warm_grey: p.ext_warm_grey.into(),
            ext_orange: p.ext_orange.into(),
            ext_yellow: p.ext_yellow.into(),
            ext_blue: p.ext_blue.into(),
            ext_purple: p.ext_purple.into(),
            ext_pink: p.ext_pink.into(),
            ext_indigo: p.ext_indigo.into(),
            accent_warm_grey: p.accent_warm_grey.into(),
            accent_orange: p.accent_orange.into(),
            accent_yellow: p.accent_yellow.into(),
            accent_purple: p.accent_purple.into(),
            accent_pink: p.accent_pink.into(),
            accent_indigo: p.accent_indigo.into(),
        }
    }
}

impl<C> CosmicPalette<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    /// name of the palette
    pub fn name(&self) -> &str {
        match &self {
            CosmicPalette::Dark(p) => &p.name,
            CosmicPalette::Light(p) => &p.name,
            CosmicPalette::HighContrastLight(p) => &p.name,
            CosmicPalette::HighContrastDark(p) => &p.name,
        }
    }
    /// save the theme to the theme directory
    pub fn save(&self) -> anyhow::Result<()> {
        let ron_path: PathBuf = [NAME, PALETTE_DIR].iter().collect();
        let ron_dirs = directories::ProjectDirs::from_path(ron_path)
            .context("Failed to get project directories.")?;
        let ron_name = format!("{}.ron", self.name());

        if let Ok(p) = ron_dirs.place_config_file(ron_name) {
            let mut f = File::create(p)?;
            f.write_all(ron::ser::to_string_pretty(self, Default::default())?.as_bytes())?;
        } else {
            anyhow::bail!("Failed to write RON theme.");
        }
        Ok(())
    }

    /// init the theme directory
    pub fn init() -> anyhow::Result<PathBuf> {
        let ron_path: PathBuf = [NAME, PALETTE_DIR].iter().collect();
        let base_dirs = directories::BaseDirs::new().context("Failed to get base directories.")?;
        Ok(base_dirs.create_config_directory(ron_path)?)
    }

    /// load a theme by name
    pub fn load_from_name(name: &str) -> anyhow::Result<Self> {
        let ron_path: PathBuf = [NAME, PALETTE_DIR].iter().collect();
        let ron_dirs = directories::ProjectDirs::from_path(ron_path)
            .context("Failed to get project directories.")?;

        let ron_name = format!("{}.ron", name);
        if let Some(p) = ron_dirs.find_config_file(ron_name) {
            let f = File::open(p)?;
            Ok(ron::de::from_reader(f)?)
        } else {
            anyhow::bail!("Failed to write RON theme.");
        }
    }

    /// load a theme by path
    pub fn load(p: &dyn AsRef<Path>) -> anyhow::Result<Self> {
        let f = File::open(p)?;
        Ok(ron::de::from_reader(f)?)
    }
}
