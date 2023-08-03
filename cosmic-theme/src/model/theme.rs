use crate::{
    util::CssColor, Component, ComponentType, Container, ContainerType, CornerRadii, CosmicPalette,
    CosmicPaletteInner, Spacing, DARK_PALETTE, LIGHT_PALETTE, NAME, THEME_DIR,
};
use anyhow::Context;
use cosmic_config::{Config, ConfigGet, ConfigSet, CosmicConfigEntry};
use directories::{BaseDirsExt, ProjectDirsExt};
use palette::{Srgb, Srgba};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    fmt,
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
/// Theme layer type
pub enum Layer {
    /// Background layer
    #[default]
    Background,
    /// Primary Layer
    Primary,
    /// Secondary Layer
    Secondary,
}

/// Cosmic Theme data structure with all colors and its name
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Theme<C> {
    /// name of the theme
    pub name: String,
    /// background element colors
    pub background: Container<C>,
    /// primary element colors
    pub primary: Container<C>,
    /// secondary element colors
    pub secondary: Container<C>,
    /// accent element colors
    pub accent: Component<C>,
    /// suggested element colors
    pub success: Component<C>,
    /// destructive element colors
    pub destructive: Component<C>,
    /// warning element colors
    pub warning: Component<C>,
    /// palette
    pub palette: CosmicPaletteInner<C>,
    /// spacing
    pub spacing: Spacing,
    /// corner radii
    pub corner_radii: CornerRadii,
    /// is dark
    pub is_dark: bool,
    /// is high contrast
    pub is_high_contrast: bool,
}

impl CosmicConfigEntry for Theme<CssColor> {
    fn write_entry(&self, config: &Config) -> Result<(), cosmic_config::Error> {
        let self_ = self.clone();
        // TODO do as transaction
        let tx = config.transaction();

        tx.set("name", self_.name)?;
        tx.set("background", self_.background)?;
        tx.set("primary", self_.primary)?;
        tx.set("secondary", self_.secondary)?;
        tx.set("accent", self_.accent)?;
        tx.set("success", self_.success)?;
        tx.set("destructive", self_.destructive)?;
        tx.set("warning", self_.warning)?;
        tx.set("palette", self_.palette)?;
        tx.set("is_dark", self_.is_dark)?;
        tx.set("is_high_contrast", self_.is_high_contrast)?;

        tx.commit()
    }

    fn get_entry(config: &Config) -> Result<Self, (Vec<cosmic_config::Error>, Self)> {
        let mut default = Self::default();
        let mut errors = Vec::new();

        match config.get::<String>("name") {
            Ok(name) => default.name = name,
            Err(e) => errors.push(e),
        }
        match config.get::<Container<CssColor>>("background") {
            Ok(background) => default.background = background,
            Err(e) => errors.push(e),
        }
        match config.get::<Container<CssColor>>("primary") {
            Ok(primary) => default.primary = primary,
            Err(e) => errors.push(e),
        }
        match config.get::<Container<CssColor>>("secondary") {
            Ok(secondary) => default.secondary = secondary,
            Err(e) => errors.push(e),
        }
        match config.get::<Component<CssColor>>("accent") {
            Ok(accent) => default.accent = accent,
            Err(e) => errors.push(e),
        }
        match config.get::<Component<CssColor>>("success") {
            Ok(success) => default.success = success,
            Err(e) => errors.push(e),
        }
        match config.get::<Component<CssColor>>("destructive") {
            Ok(destructive) => default.destructive = destructive,
            Err(e) => errors.push(e),
        }
        match config.get::<Component<CssColor>>("warning") {
            Ok(warning) => default.warning = warning,
            Err(e) => errors.push(e),
        }
        match config.get::<CosmicPaletteInner<CssColor>>("palette") {
            Ok(palette) => default.palette = palette,
            Err(e) => errors.push(e),
        }
        match config.get::<bool>("is_dark") {
            Ok(is_dark) => default.is_dark = is_dark,
            Err(e) => errors.push(e),
        }
        match config.get::<bool>("is_high_contrast") {
            Ok(is_high_contrast) => default.is_high_contrast = is_high_contrast,
            Err(e) => errors.push(e),
        }
        match config.get::<Spacing>("spacing") {
            Ok(spacing) => default.spacing = spacing,
            Err(e) => errors.push(e),
        }
        match config.get::<CornerRadii>("corner_radii") {
            Ok(corner_radii) => default.corner_radii = corner_radii,
            Err(e) => errors.push(e),
        }

        if errors.is_empty() {
            Ok(default)
        } else {
            Err((errors, default))
        }
    }
}

impl Default for Theme<Srgba> {
    fn default() -> Self {
        Theme::<CssColor>::dark_default().into_srgba()
    }
}

impl Default for Theme<CssColor> {
    fn default() -> Self {
        Self::dark_default()
    }
}

/// Trait for layered themes
pub trait LayeredTheme {
    /// Set the layer of the theme
    fn set_layer(&mut self, layer: Layer);
}

impl<C> Theme<C> {
    /// version of the theme
    pub fn version() -> u64 {
        1
    }

    /// id of the theme
    pub fn id() -> &'static str {
        NAME
    }
}

impl<C> Theme<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    /// Convert the theme to a high-contrast variant
    pub fn to_high_contrast(&self) -> Self {
        todo!();
    }

    /// save the theme to the theme directory
    pub fn save(&self) -> anyhow::Result<()> {
        let ron_path: PathBuf = [NAME, THEME_DIR].iter().collect();
        let ron_dirs = directories::ProjectDirs::from_path(ron_path)
            .context("Failed to get project directories.")?;
        let ron_name = format!("{}.ron", &self.name);

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
        let ron_path: PathBuf = [NAME, THEME_DIR].iter().collect();
        let base_dirs = directories::BaseDirs::new().context("Failed to get base directories.")?;
        Ok(base_dirs.create_config_directory(ron_path)?)
    }

    /// load a theme by name
    pub fn load_from_name(name: &str) -> anyhow::Result<Self> {
        let ron_path: PathBuf = [NAME, THEME_DIR].iter().collect();
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

    // TODO convenient getter functions for each named color variable
    /// get @accent_color
    pub fn accent_color(&self) -> Srgba {
        self.accent.base.clone().into()
    }
    /// get @success_color
    pub fn success_color(&self) -> Srgba {
        self.success.base.clone().into()
    }
    /// get @destructive_color
    pub fn destructive_color(&self) -> Srgba {
        self.destructive.base.clone().into()
    }
    /// get @warning_color
    pub fn warning_color(&self) -> Srgba {
        self.warning.base.clone().into()
    }

    // Containers
    /// get @bg_color
    pub fn bg_color(&self) -> Srgba {
        self.background.base.clone().into()
    }
    /// get @bg_component_color
    pub fn bg_component_color(&self) -> Srgba {
        self.background.component.base.clone().into()
    }
    /// get @primary_container_color
    pub fn primary_container_color(&self) -> Srgba {
        self.primary.base.clone().into()
    }
    /// get @primary_component_color
    pub fn primary_component_color(&self) -> Srgba {
        self.primary.component.base.clone().into()
    }
    /// get @secondary_container_color
    pub fn secondary_container_color(&self) -> Srgba {
        self.secondary.base.clone().into()
    }
    /// get @secondary_component_color
    pub fn secondary_component_color(&self) -> Srgba {
        self.secondary.component.base.clone().into()
    }

    // Text
    /// get @on_bg_color
    pub fn on_bg_color(&self) -> Srgba {
        self.background.on.clone().into()
    }
    /// get @on_bg_component_color
    pub fn on_bg_component_color(&self) -> Srgba {
        self.background.component.on.clone().into()
    }
    /// get @on_primary_color
    pub fn on_primary_container_color(&self) -> Srgba {
        self.primary.on.clone().into()
    }
    /// get @on_primary_component_color
    pub fn on_primary_component_color(&self) -> Srgba {
        self.primary.component.on.clone().into()
    }
    /// get @on_secondary_color
    pub fn on_secondary_container_color(&self) -> Srgba {
        self.secondary.on.clone().into()
    }
    /// get @on_secondary_component_color
    pub fn on_secondary_component_color(&self) -> Srgba {
        self.secondary.component.on.clone().into()
    }
    /// get @accent_text_color
    pub fn accent_text_color(&self) -> Srgba {
        self.accent.base.clone().into()
    }
    /// get @success_text_color
    pub fn success_text_color(&self) -> Srgba {
        self.success.base.clone().into()
    }
    /// get @warning_text_color
    pub fn warning_text_color(&self) -> Srgba {
        self.warning.base.clone().into()
    }
    /// get @destructive_text_color
    pub fn destructive_text_color(&self) -> Srgba {
        self.destructive.base.clone().into()
    }
    /// get @on_accent_color
    pub fn on_accent_color(&self) -> Srgba {
        self.accent.on.clone().into()
    }
    /// get @on_success_color
    pub fn on_success_color(&self) -> Srgba {
        self.success.on.clone().into()
    }
    /// get @oon_warning_color
    pub fn on_warning_color(&self) -> Srgba {
        self.warning.on.clone().into()
    }
    /// get @on_destructive_color
    pub fn on_destructive_color(&self) -> Srgba {
        self.destructive.on.clone().into()
    }

    // Borders and Dividers
    /// get @bg_divider
    pub fn bg_divider(&self) -> Srgba {
        self.background.divider.clone().into()
    }
    /// get @bg_component_divider
    pub fn bg_component_divider(&self) -> Srgba {
        self.background.component.divider.clone().into()
    }
    /// get @primary_container_divider
    pub fn primary_container_divider(&self) -> Srgba {
        self.primary.divider.clone().into()
    }
    /// get @primary_component_divider
    pub fn primary_component_divider(&self) -> Srgba {
        self.primary.component.divider.clone().into()
    }
    /// get @secondary_container_divider
    pub fn secondary_container_divider(&self) -> Srgba {
        self.secondary.divider.clone().into()
    }
    /// get @secondary_component_divider
    pub fn secondary_component_divider(&self) -> Srgba {
        self.secondary.component.divider.clone().into()
    }

    /// get @window_header_bg
    pub fn window_header_bg(&self) -> Srgba {
        self.background.base.clone().into()
    }

    /// get @space_none
    pub fn space_none(&self) -> u16 {
        self.spacing.space_none
    }
    /// get @space_xxxs
    pub fn space_xxxs(&self) -> u16 {
        self.spacing.space_xxxs
    }
    /// get @space_xxs
    pub fn space_xxs(&self) -> u16 {
        self.spacing.space_xxs
    }
    /// get @space_xs
    pub fn space_xs(&self) -> u16 {
        self.spacing.space_xs
    }
    /// get @space_s
    pub fn space_s(&self) -> u16 {
        self.spacing.space_s
    }
    /// get @space_m
    pub fn space_m(&self) -> u16 {
        self.spacing.space_m
    }
    /// get @space_l
    pub fn space_l(&self) -> u16 {
        self.spacing.space_l
    }
    /// get @space_xl
    pub fn space_xl(&self) -> u16 {
        self.spacing.space_xl
    }
    /// get @space_xxl
    pub fn space_xxl(&self) -> u16 {
        self.spacing.space_xxl
    }
    /// get @space_xxxl
    pub fn space_xxxl(&self) -> u16 {
        self.spacing.space_xxxl
    }

    /// get @radius_0
    pub fn radius_0(&self) -> [u16; 4] {
        self.corner_radii.radius_0
    }
    /// get @radius_xs
    pub fn radius_xs(&self) -> [u16; 4] {
        self.corner_radii.radius_xs
    }
    /// get @radius_s
    pub fn radius_s(&self) -> [u16; 4] {
        self.corner_radii.radius_s
    }
    /// get @radius_m
    pub fn radius_m(&self) -> [u16; 4] {
        self.corner_radii.radius_m
    }
    /// get @radius_l
    pub fn radius_l(&self) -> [u16; 4] {
        self.corner_radii.radius_l
    }
    /// get @radius_xl
    pub fn radius_xl(&self) -> [u16; 4] {
        self.corner_radii.radius_xl
    }
}

impl Theme<CssColor> {
    /// get the built in light theme
    pub fn light_default() -> Self {
        LIGHT_PALETTE.clone().into()
    }

    /// get the built in dark theme
    pub fn dark_default() -> Self {
        DARK_PALETTE.clone().into()
    }

    /// get the built in high contrast dark theme
    pub fn high_contrast_dark_default() -> Self {
        CosmicPalette::HighContrastDark(DARK_PALETTE.as_ref().clone()).into()
    }

    /// get the built in high contrast light theme
    pub fn high_contrast_light_default() -> Self {
        CosmicPalette::HighContrastLight(LIGHT_PALETTE.as_ref().clone()).into()
    }

    /// convert to srgba
    pub fn into_srgba(self) -> Theme<Srgba> {
        Theme {
            name: self.name,
            background: self.background.into_srgba(),
            primary: self.primary.into_srgba(),
            secondary: self.secondary.into_srgba(),
            accent: self.accent.into_srgba(),
            success: self.success.into_srgba(),
            destructive: self.destructive.into_srgba(),
            warning: self.warning.into_srgba(),
            palette: self.palette.into(),
            is_dark: self.is_dark,
            is_high_contrast: self.is_high_contrast,
            corner_radii: self.corner_radii,
            spacing: self.spacing,
        }
    }
}

impl<C> From<CosmicPalette<C>> for Theme<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    fn from(p: CosmicPalette<C>) -> Self {
        let is_dark = p.is_dark();
        let is_high_contrast = p.is_high_contrast();
        Self {
            name: p.name().to_string(),
            background: (p.clone(), ContainerType::Background).into(),
            primary: (p.clone(), ContainerType::Primary).into(),
            secondary: (p.clone(), ContainerType::Secondary).into(),
            accent: (p.clone(), ComponentType::Accent).into(),
            success: (p.clone(), ComponentType::Success).into(),
            destructive: (p.clone(), ComponentType::Destructive).into(),
            warning: (p.clone(), ComponentType::Warning).into(),
            palette: match p {
                CosmicPalette::Dark(p) => p.into(),
                CosmicPalette::Light(p) => p.into(),
                CosmicPalette::HighContrastLight(p) => p.into(),
                CosmicPalette::HighContrastDark(p) => p.into(),
            },
            is_dark,
            is_high_contrast,
            spacing: Spacing::default(),
            corner_radii: CornerRadii::default(),
        }
    }
}

/// Helper for building customized themes
#[derive(Debug, Serialize, Deserialize)]
pub struct ThemeBuilder {
    palette: CosmicPalette<Srgba>,
    spacing: Spacing,
    corner_radii: CornerRadii,
    neutral_tint: Option<Srgb>,
    bg_color: Option<Srgba>,
    primary_container_bg: Option<Srgba>,
    text_tint: Option<Srgb>,
    accent: Option<Srgb>,
}

impl Default for ThemeBuilder {
    fn default() -> Self {
        Self {
            palette: DARK_PALETTE.to_owned().into(),
            spacing: Spacing::default(),
            corner_radii: CornerRadii::default(),
            neutral_tint: Default::default(),
            text_tint: Default::default(),
            bg_color: Default::default(),
            primary_container_bg: Default::default(),
            accent: Default::default(),
        }
    }
}

impl ThemeBuilder {
    /// Get a builder that is initialized with the default dark theme
    pub fn dark() -> Self {
        Self {
            palette: DARK_PALETTE.to_owned().into(),
            ..Default::default()
        }
    }

    /// Get a builder that is initialized with the default light theme
    pub fn light() -> Self {
        Self {
            palette: LIGHT_PALETTE.to_owned().into(),
            ..Default::default()
        }
    }

    /// Get a builder that is initialized with the default dark high contrast theme
    pub fn dark_high_contrast() -> Self {
        let palette: CosmicPalette<Srgba> = DARK_PALETTE.to_owned().into();
        Self {
            palette: CosmicPalette::HighContrastLight(palette.inner()),
            ..Default::default()
        }
    }

    /// Get a builder that is initialized with the default light high contrast theme
    pub fn light_high_contrast() -> Self {
        let palette: CosmicPalette<Srgba> = LIGHT_PALETTE.to_owned().into();
        Self {
            palette: CosmicPalette::HighContrastLight(palette.inner()),
            ..Default::default()
        }
    }

    /// set the spacing of the builder
    pub fn spacing(mut self, spacing: Spacing) -> Self {
        self.spacing = spacing;
        self
    }

    /// set the corner_radii of the builder
    pub fn corner_radii(mut self, corner_radii: CornerRadii) -> Self {
        self.corner_radii = corner_radii;
        self
    }

    /// apply a neutral tint to the palette
    pub fn neutral_tint(mut self, tint: Srgb) -> Self {
        self.neutral_tint = Some(tint);
        self
    }

    /// apply a text tint to the palette
    pub fn text_tint(mut self, tint: Srgb) -> Self {
        self.text_tint = Some(tint);
        self
    }

    /// apply a background color to the palette
    pub fn bg_color(mut self, c: Srgba) -> Self {
        self.bg_color = Some(c);
        self
    }

    /// apply a primary container background color to the palette
    pub fn primary_container_bg(mut self, c: Srgba) -> Self {
        self.primary_container_bg = Some(c);
        self
    }

    /// apply a accent color to the palette
    pub fn accent(mut self, c: Srgb) -> Self {
        self.accent = Some(c);
        self
    }

    /// build the theme
    pub fn build(self) -> Theme<Srgba> {
        let Self {
            mut palette,
            spacing,
            corner_radii,
            neutral_tint,
            text_tint,
            bg_color,
            primary_container_bg,
            accent,
        } = self;

        if let Some(accent) = accent {
            palette.as_mut().accent = accent.into();
        }

        // TODO apply the customizations

        if let Some(accent) = accent {
            palette.as_mut().accent = accent.into();
        }

        let mut theme: Theme<Srgba> = palette.into();
        theme.spacing = spacing;
        theme.corner_radii = corner_radii;
        theme
    }
}
