use crate::{
    composite::over,
    steps::{color_index, get_surface_color, get_text, steps},
    Component, Container, CornerRadii, CosmicPalette, CosmicPaletteInner, Spacing, ThemeMode,
    DARK_PALETTE, LIGHT_PALETTE, NAME,
};
use cosmic_config::{Config, CosmicConfigEntry};
use palette::{rgb::Rgb, IntoColor, Oklcha, Srgb, Srgba};
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

/// ID for the current dark `ThemeBuilder` config
pub const DARK_THEME_BUILDER_ID: &str = "com.system76.CosmicTheme.Dark.Builder";

/// ID for the current dark Theme config
pub const DARK_THEME_ID: &str = "com.system76.CosmicTheme.Dark";

/// ID for the current light `ThemeBuilder`` config
pub const LIGHT_THEME_BUILDER_ID: &str = "com.system76.CosmicTheme.Light.Builder";

/// ID for the current light Theme config
pub const LIGHT_THEME_ID: &str = "com.system76.CosmicTheme.Light";

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

#[must_use]
/// Cosmic Theme data structure with all colors and its name
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    cosmic_config::cosmic_config_derive::CosmicConfigEntry,
)]
#[version = 1]
pub struct Theme {
    /// name of the theme
    pub name: String,
    /// background element colors
    pub background: Container,
    /// primary element colors
    pub primary: Container,
    /// secondary element colors
    pub secondary: Container,
    /// accent element colors
    pub accent: Component,
    /// suggested element colors
    pub success: Component,
    /// destructive element colors
    pub destructive: Component,
    /// warning element colors
    pub warning: Component,
    /// accent button element colors
    pub accent_button: Component,
    /// suggested button element colors
    pub success_button: Component,
    /// destructive button element colors
    pub destructive_button: Component,
    /// warning button element colors
    pub warning_button: Component,
    /// icon button element colors
    pub icon_button: Component,
    /// link button element colors
    pub link_button: Component,
    /// text button element colors
    pub text_button: Component,
    /// button component styling
    pub button: Component,
    /// palette
    pub palette: CosmicPaletteInner,
    /// spacing
    pub spacing: Spacing,
    /// corner radii
    pub corner_radii: CornerRadii,
    /// is dark
    pub is_dark: bool,
    /// is high contrast
    pub is_high_contrast: bool,
    /// cosmic-comp window gaps size (outer, inner)
    pub gaps: (u32, u32),
    /// cosmic-comp active hint window outline width
    pub active_hint: u32,
    /// cosmic-comp custom window hint color
    pub window_hint: Option<Srgb>,
    /// enables blurred transparency
    pub is_frosted: bool,
    /// shade color for dialogs
    pub shade: Srgba,
}

impl Default for Theme {
    fn default() -> Self {
        Self::preferred_theme()
    }
}

/// Trait for layered themes
pub trait LayeredTheme {
    /// Set the layer of the theme
    fn set_layer(&mut self, layer: Layer);
}

impl Theme {
    #[must_use]
    /// id of the theme
    pub fn id() -> &'static str {
        NAME
    }

    /// Get the config for the current dark theme
    pub fn dark_config() -> Result<Config, cosmic_config::Error> {
        Config::new(DARK_THEME_ID, Self::VERSION)
    }

    /// Get the config for the current light theme
    pub fn light_config() -> Result<Config, cosmic_config::Error> {
        Config::new(LIGHT_THEME_ID, Self::VERSION)
    }

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

    /// Convert the theme to a high-contrast variant
    pub fn to_high_contrast(&self) -> Self {
        todo!();
    }

    // TODO convenient getter functions for each named color variable
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @accent_color
    pub fn accent_color(&self) -> Srgba {
        self.accent.base
    }

    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @success_color
    pub fn success_color(&self) -> Srgba {
        self.success.base
    }

    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @destructive_color
    pub fn destructive_color(&self) -> Srgba {
        self.destructive.base
    }

    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @warning_color
    pub fn warning_color(&self) -> Srgba {
        self.warning.base
    }

    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @small_widget_divider
    pub fn small_widget_divider(&self) -> Srgba {
        self.palette.neutral_9
    }

    // Containers
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @bg_color
    pub fn bg_color(&self) -> Srgba {
        self.background.base
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @bg_component_color
    pub fn bg_component_color(&self) -> Srgba {
        self.background.component.base
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @primary_container_color
    pub fn primary_container_color(&self) -> Srgba {
        self.primary.base
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @primary_component_color
    pub fn primary_component_color(&self) -> Srgba {
        self.primary.component.base
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @secondary_container_color
    pub fn secondary_container_color(&self) -> Srgba {
        self.secondary.base
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @secondary_component_color
    pub fn secondary_component_color(&self) -> Srgba {
        self.secondary.component.base
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @button_bg_color
    pub fn button_bg_color(&self) -> Srgba {
        self.button.base
    }

    // Text
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @on_bg_color
    pub fn on_bg_color(&self) -> Srgba {
        self.background.on
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @on_bg_component_color
    pub fn on_bg_component_color(&self) -> Srgba {
        self.background.component.on
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @on_primary_color
    pub fn on_primary_container_color(&self) -> Srgba {
        self.primary.on
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @on_primary_component_color
    pub fn on_primary_component_color(&self) -> Srgba {
        self.primary.component.on
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @on_secondary_color
    pub fn on_secondary_container_color(&self) -> Srgba {
        self.secondary.on
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @on_secondary_component_color
    pub fn on_secondary_component_color(&self) -> Srgba {
        self.secondary.component.on
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @accent_text_color
    pub fn accent_text_color(&self) -> Srgba {
        self.accent.base
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @success_text_color
    pub fn success_text_color(&self) -> Srgba {
        self.success.base
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @warning_text_color
    pub fn warning_text_color(&self) -> Srgba {
        self.warning.base
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @destructive_text_color
    pub fn destructive_text_color(&self) -> Srgba {
        self.destructive.base
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @on_accent_color
    pub fn on_accent_color(&self) -> Srgba {
        self.accent.on
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @on_success_color
    pub fn on_success_color(&self) -> Srgba {
        self.success.on
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @on_warning_color
    pub fn on_warning_color(&self) -> Srgba {
        self.warning.on
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @on_destructive_color
    pub fn on_destructive_color(&self) -> Srgba {
        self.destructive.on
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @button_color
    pub fn button_color(&self) -> Srgba {
        self.button.on
    }

    // Borders and Dividers
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @bg_divider
    pub fn bg_divider(&self) -> Srgba {
        self.background.divider
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @bg_component_divider
    pub fn bg_component_divider(&self) -> Srgba {
        self.background.component.divider
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @primary_container_divider
    pub fn primary_container_divider(&self) -> Srgba {
        self.primary.divider
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @primary_component_divider
    pub fn primary_component_divider(&self) -> Srgba {
        self.primary.component.divider
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @secondary_container_divider
    pub fn secondary_container_divider(&self) -> Srgba {
        self.secondary.divider
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @button_divider
    pub fn button_divider(&self) -> Srgba {
        self.button.divider
    }

    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @window_header_bg
    pub fn window_header_bg(&self) -> Srgba {
        self.background.base
    }

    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @space_none
    pub fn space_none(&self) -> u16 {
        self.spacing.space_none
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @space_xxxs
    pub fn space_xxxs(&self) -> u16 {
        self.spacing.space_xxxs
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @space_xxs
    pub fn space_xxs(&self) -> u16 {
        self.spacing.space_xxs
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @space_xs
    pub fn space_xs(&self) -> u16 {
        self.spacing.space_xs
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @space_s
    pub fn space_s(&self) -> u16 {
        self.spacing.space_s
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @space_m
    pub fn space_m(&self) -> u16 {
        self.spacing.space_m
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @space_l
    pub fn space_l(&self) -> u16 {
        self.spacing.space_l
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @space_xl
    pub fn space_xl(&self) -> u16 {
        self.spacing.space_xl
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @space_xxl
    pub fn space_xxl(&self) -> u16 {
        self.spacing.space_xxl
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @space_xxxl
    pub fn space_xxxl(&self) -> u16 {
        self.spacing.space_xxxl
    }

    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @radius_0
    pub fn radius_0(&self) -> [f32; 4] {
        self.corner_radii.radius_0
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @radius_xs
    pub fn radius_xs(&self) -> [f32; 4] {
        self.corner_radii.radius_xs
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @radius_s
    pub fn radius_s(&self) -> [f32; 4] {
        self.corner_radii.radius_s
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @radius_m
    pub fn radius_m(&self) -> [f32; 4] {
        self.corner_radii.radius_m
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @radius_l
    pub fn radius_l(&self) -> [f32; 4] {
        self.corner_radii.radius_l
    }
    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @radius_xl
    pub fn radius_xl(&self) -> [f32; 4] {
        self.corner_radii.radius_xl
    }

    #[must_use]
    #[allow(clippy::doc_markdown)]
    /// get @shade_color
    pub fn shade_color(&self) -> Srgba {
        self.shade
    }

    /// get the active theme
    pub fn get_active() -> Result<Self, (Vec<cosmic_config::Error>, Self)> {
        let config =
            Config::new(Self::id(), Self::VERSION).map_err(|e| (vec![e], Self::default()))?;
        let is_dark = ThemeMode::is_dark(&config).map_err(|e| (vec![e], Self::default()))?;
        let config = if is_dark {
            Self::dark_config()
        } else {
            Self::light_config()
        }
        .map_err(|e| (vec![e], Self::default()))?;
        Self::get_entry(&config)
    }

    #[must_use]
    /// Rebuild the current theme with the provided accent
    pub fn with_accent(&self, c: Srgba) -> Self {
        let mut oklcha: Oklcha = c.into_color();
        let cur_oklcha: Oklcha = self.accent_color().into_color();
        oklcha.l = cur_oklcha.l;
        let adjusted_c: Srgb = oklcha.into_color();

        let is_dark = self.is_dark;

        let mut builder = if is_dark {
            ThemeBuilder::dark_config()
                .ok()
                .and_then(|h| ThemeBuilder::get_entry(&h).ok())
                .unwrap_or_else(ThemeBuilder::dark)
        } else {
            ThemeBuilder::light_config()
                .ok()
                .and_then(|h| ThemeBuilder::get_entry(&h).ok())
                .unwrap_or_else(ThemeBuilder::light)
        };
        builder = builder.accent(adjusted_c);
        builder.build()
    }

    /// choose default color palette based on preferred GTK color scheme
    pub fn gtk_prefer_colorscheme() -> Self {
        let gsettings = "/usr/bin/gsettings";

        let cmd = std::process::Command::new(gsettings)
            .arg("get")
            .arg("org.gnome.desktop.interface")
            .arg("color-scheme")
            .output();

        if let Ok(cmd) = cmd {
            let color_scheme = String::from_utf8_lossy(&cmd.stdout);

            if color_scheme.trim().contains("light") {
                return Self::light_default();
            }
        };

        Self::dark_default()
    }

    /// check current desktop environment and preferred color scheme and set it as default
    pub fn preferred_theme() -> Self {
        let current_desktop = std::env::var("XDG_CURRENT_DESKTOP");

        if let Ok(desktop) = current_desktop {
            if desktop.trim().to_lowercase().contains("gnome") {
                return Self::gtk_prefer_colorscheme();
            }
        }

        Self::dark_default()
    }
}

impl From<CosmicPalette> for Theme {
    fn from(p: CosmicPalette) -> Self {
        ThemeBuilder::palette(p).build()
    }
}

#[must_use]
/// Helper for building customized themes
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    cosmic_config::cosmic_config_derive::CosmicConfigEntry,
    PartialEq,
)]
#[version = 1]
pub struct ThemeBuilder {
    /// override the palette for the builder
    pub palette: CosmicPalette,
    /// override spacing for the builder
    pub spacing: Spacing,
    /// override corner radii for the builder
    pub corner_radii: CornerRadii,
    /// override neutral_tint for the builder
    pub neutral_tint: Option<Srgb>,
    /// override bg_color for the builder
    pub bg_color: Option<Srgba>,
    /// override the primary container bg color for the builder
    pub primary_container_bg: Option<Srgba>,
    /// override the secontary container bg color for the builder
    pub secondary_container_bg: Option<Srgba>,
    /// override the text tint for the builder
    pub text_tint: Option<Srgb>,
    /// override the accent color for the builder
    pub accent: Option<Srgb>,
    /// override the success color for the builder
    pub success: Option<Srgb>,
    /// override the warning color for the builder
    pub warning: Option<Srgb>,
    /// override the destructive color for the builder
    pub destructive: Option<Srgb>,
    /// enabled blurred transparency
    pub is_frosted: bool, // TODO handle
    /// cosmic-comp window gaps size (outer, inner)
    pub gaps: (u32, u32),
    /// cosmic-comp active hint window outline width
    pub active_hint: u32,
    /// cosmic-comp custom window hint color
    pub window_hint: Option<Srgb>,
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
            secondary_container_bg: Default::default(),
            accent: Default::default(),
            success: Default::default(),
            warning: Default::default(),
            destructive: Default::default(),
            is_frosted: false,
            // cosmic-comp theme settings
            gaps: (0, 8),
            active_hint: 3,
            window_hint: None,
        }
    }
}

impl ThemeBuilder {
    /// Get a builder that is initialized with the default dark theme
    pub fn dark() -> Self {
        Self {
            palette: DARK_PALETTE.to_owned(),
            ..Default::default()
        }
    }

    /// Get a builder that is initialized with the default light theme
    pub fn light() -> Self {
        Self {
            palette: LIGHT_PALETTE.to_owned(),
            ..Default::default()
        }
    }

    /// Get a builder that is initialized with the default dark high contrast theme
    pub fn dark_high_contrast() -> Self {
        let palette: CosmicPalette = DARK_PALETTE.to_owned();
        Self {
            palette: CosmicPalette::HighContrastDark(palette.inner()),
            ..Default::default()
        }
    }

    /// Get a builder that is initialized with the default light high contrast theme
    pub fn light_high_contrast() -> Self {
        let palette: CosmicPalette = LIGHT_PALETTE.to_owned();
        Self {
            palette: CosmicPalette::HighContrastLight(palette.inner()),
            ..Default::default()
        }
    }

    /// Get a builder that is initialized with the provided palette
    pub fn palette(palette: CosmicPalette) -> Self {
        Self {
            palette,
            ..Default::default()
        }
    }

    /// set the spacing of the builder
    pub fn spacing(mut self, spacing: Spacing) -> Self {
        self.spacing = spacing;
        self
    }

    /// set the corner radii of the builder
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

    /// apply a success color to the palette
    pub fn success(mut self, c: Srgb) -> Self {
        self.success = Some(c);
        self
    }

    /// apply a warning color to the palette
    pub fn warning(mut self, c: Srgb) -> Self {
        self.warning = Some(c);
        self
    }

    /// apply a destructive color to the palette
    pub fn destructive(mut self, c: Srgb) -> Self {
        self.destructive = Some(c);
        self
    }

    #[allow(clippy::too_many_lines)]
    /// build the theme
    pub fn build(self) -> Theme {
        let Self {
            mut palette,
            spacing,
            corner_radii,
            neutral_tint,
            text_tint,
            bg_color,
            primary_container_bg,
            secondary_container_bg,
            accent,
            success,
            warning,
            destructive,
            gaps,
            active_hint,
            window_hint,
            is_frosted,
        } = self;

        let is_dark = palette.is_dark();
        let is_high_contrast = palette.is_high_contrast();

        let accent = if let Some(accent) = accent {
            accent.into_color()
        } else {
            palette.as_ref().blue
        };

        let success = if let Some(success) = success {
            success.into_color()
        } else {
            palette.as_ref().green
        };

        let warning = if let Some(warning) = warning {
            warning.into_color()
        } else {
            palette.as_ref().yellow
        };

        let destructive = if let Some(destructive) = destructive {
            destructive.into_color()
        } else {
            palette.as_ref().red
        };

        let text_steps_array = text_tint.map(|c| steps(c, NonZeroUsize::new(100).unwrap()));

        if let Some(neutral_tint) = neutral_tint {
            let mut neutral_steps_arr = steps(neutral_tint, NonZeroUsize::new(11).unwrap());
            if !is_dark {
                neutral_steps_arr.reverse();
            }

            let p = palette.as_mut();
            p.neutral_0 = neutral_steps_arr[0];
            p.neutral_1 = neutral_steps_arr[1];
            p.neutral_2 = neutral_steps_arr[2];
            p.neutral_3 = neutral_steps_arr[3];
            p.neutral_4 = neutral_steps_arr[4];
            p.neutral_5 = neutral_steps_arr[5];
            p.neutral_6 = neutral_steps_arr[6];
            p.neutral_7 = neutral_steps_arr[7];
            p.neutral_8 = neutral_steps_arr[8];
            p.neutral_9 = neutral_steps_arr[9];
            p.neutral_10 = neutral_steps_arr[10];
        }

        let p_ref = palette.as_ref();

        let neutral_steps = steps(
            neutral_tint.unwrap_or(Rgb::new(0.0, 0.0, 0.0)),
            NonZeroUsize::new(100).unwrap(),
        );

        let bg = if let Some(bg_color) = bg_color {
            bg_color
        } else {
            p_ref.gray_1
        };

        let step_array = steps(bg, NonZeroUsize::new(100).unwrap());
        let bg_index = color_index(bg, step_array.len());

        let mut component_hovered_overlay = if bg_index < 91 {
            p_ref.neutral_10
        } else {
            p_ref.neutral_0
        };
        component_hovered_overlay.alpha = 0.1;

        let mut component_pressed_overlay = component_hovered_overlay;
        component_pressed_overlay.alpha = 0.2;

        // Standard button background is neutral 7 with 25% opacity
        let button_bg = {
            let mut color = p_ref.neutral_7;
            color.alpha = 0.25;
            color
        };

        let (mut button_hovered_overlay, mut button_pressed_overlay) =
            (p_ref.neutral_5, p_ref.neutral_2);
        button_hovered_overlay.alpha = 0.2;
        button_pressed_overlay.alpha = 0.5;

        let bg_component = get_surface_color(bg_index, 8, &step_array, is_dark, &p_ref.neutral_2);
        let on_bg_component = get_text(
            color_index(bg_component, step_array.len()),
            &step_array,
            &p_ref.neutral_8,
            text_steps_array.as_ref(),
        );

        let mut theme: Theme = Theme {
            name: palette.name().to_string(),
            shade: if palette.is_dark() {
                Srgba::new(0., 0., 0., 0.32)
            } else {
                Srgba::new(0., 0., 0., 0.08)
            },
            background: Container::new(
                Component::component(
                    bg_component,
                    accent,
                    on_bg_component,
                    component_hovered_overlay,
                    component_pressed_overlay,
                    is_high_contrast,
                    p_ref.neutral_8,
                ),
                bg,
                get_text(
                    bg_index,
                    &step_array,
                    &p_ref.neutral_8,
                    text_steps_array.as_ref(),
                ),
                get_surface_color(
                    bg_index,
                    5,
                    &neutral_steps,
                    bg_index <= 65,
                    &p_ref.neutral_6,
                ),
            ),
            primary: {
                let container_bg = if let Some(primary_container_bg_color) = primary_container_bg {
                    primary_container_bg_color
                } else {
                    get_surface_color(bg_index, 5, &step_array, is_dark, &p_ref.neutral_1)
                };

                let base_index: usize = color_index(container_bg, step_array.len());
                let component_base =
                    get_surface_color(base_index, 6, &step_array, is_dark, &p_ref.neutral_3);

                component_hovered_overlay = if base_index < 91 {
                    p_ref.neutral_10
                } else {
                    p_ref.neutral_0
                };
                component_hovered_overlay.alpha = 0.1;

                component_pressed_overlay = component_hovered_overlay;
                component_pressed_overlay.alpha = 0.2;

                let container = Container::new(
                    Component::component(
                        component_base,
                        accent,
                        get_text(
                            color_index(component_base, step_array.len()),
                            &step_array,
                            &p_ref.neutral_8,
                            text_steps_array.as_ref(),
                        ),
                        component_hovered_overlay,
                        component_pressed_overlay,
                        is_high_contrast,
                        p_ref.neutral_8,
                    ),
                    container_bg,
                    get_text(
                        base_index,
                        &step_array,
                        &p_ref.neutral_8,
                        text_steps_array.as_ref(),
                    ),
                    get_surface_color(
                        base_index,
                        5,
                        &neutral_steps,
                        base_index <= 65,
                        &p_ref.neutral_6,
                    ),
                );

                container
            },
            secondary: {
                let container_bg = if let Some(secondary_container_bg) = secondary_container_bg {
                    secondary_container_bg
                } else {
                    get_surface_color(bg_index, 10, &step_array, is_dark, &p_ref.neutral_2)
                };

                let base_index = color_index(container_bg, step_array.len());
                let secondary_component =
                    get_surface_color(base_index, 3, &step_array, is_dark, &p_ref.neutral_4);

                component_hovered_overlay = if base_index < 91 {
                    p_ref.neutral_10
                } else {
                    p_ref.neutral_0
                };
                component_hovered_overlay.alpha = 0.1;

                component_pressed_overlay = component_hovered_overlay;
                component_pressed_overlay.alpha = 0.2;

                Container::new(
                    Component::component(
                        secondary_component,
                        accent,
                        get_text(
                            color_index(secondary_component, step_array.len()),
                            &step_array,
                            &p_ref.neutral_8,
                            text_steps_array.as_ref(),
                        ),
                        component_hovered_overlay,
                        component_pressed_overlay,
                        is_high_contrast,
                        p_ref.neutral_8,
                    ),
                    container_bg,
                    get_text(
                        base_index,
                        &step_array,
                        &p_ref.neutral_8,
                        text_steps_array.as_ref(),
                    ),
                    get_surface_color(
                        base_index,
                        5,
                        &neutral_steps,
                        base_index <= 65,
                        &p_ref.neutral_6,
                    ),
                )
            },
            accent: Component::colored_component(
                accent,
                p_ref.neutral_0,
                accent,
                button_hovered_overlay,
                button_pressed_overlay,
            ),
            accent_button: Component::colored_button(
                accent,
                p_ref.neutral_1,
                p_ref.neutral_0,
                accent,
                button_hovered_overlay,
                button_pressed_overlay,
            ),
            button: Component::component(
                button_bg,
                accent,
                on_bg_component,
                button_hovered_overlay,
                button_pressed_overlay,
                is_high_contrast,
                p_ref.neutral_8,
            ),
            destructive: Component::colored_component(
                destructive,
                p_ref.neutral_0,
                accent,
                button_hovered_overlay,
                button_pressed_overlay,
            ),
            destructive_button: Component::colored_button(
                destructive,
                p_ref.neutral_1,
                p_ref.neutral_0,
                accent,
                button_hovered_overlay,
                button_pressed_overlay,
            ),
            icon_button: Component::component(
                Srgba::new(0.0, 0.0, 0.0, 0.0),
                accent,
                p_ref.neutral_8,
                button_hovered_overlay,
                button_pressed_overlay,
                is_high_contrast,
                p_ref.neutral_8,
            ),
            link_button: {
                let mut component = Component::component(
                    Srgba::new(0.0, 0.0, 0.0, 0.0),
                    accent,
                    accent,
                    Srgba::new(0.0, 0.0, 0.0, 0.0),
                    Srgba::new(0.0, 0.0, 0.0, 0.0),
                    is_high_contrast,
                    p_ref.neutral_8,
                );

                let mut on_50 = component.on;
                on_50.alpha = 0.5;

                component.on_disabled = over(on_50, component.base);
                component
            },
            success: Component::colored_component(
                success,
                p_ref.neutral_0,
                accent,
                button_hovered_overlay,
                button_pressed_overlay,
            ),
            success_button: Component::colored_button(
                success,
                p_ref.neutral_1,
                p_ref.neutral_0,
                accent,
                button_hovered_overlay,
                button_pressed_overlay,
            ),
            text_button: Component::component(
                Srgba::new(0.0, 0.0, 0.0, 0.0),
                accent,
                accent,
                button_hovered_overlay,
                button_pressed_overlay,
                is_high_contrast,
                p_ref.neutral_8,
            ),
            warning: Component::colored_component(
                warning,
                p_ref.neutral_0,
                accent,
                button_hovered_overlay,
                button_pressed_overlay,
            ),
            warning_button: Component::colored_button(
                warning,
                p_ref.neutral_10,
                p_ref.neutral_0,
                accent,
                button_hovered_overlay,
                button_pressed_overlay,
            ),
            palette: palette.inner(),
            spacing,
            corner_radii,
            is_dark,
            is_high_contrast,
            gaps,
            active_hint,
            window_hint,
            is_frosted,
        };
        theme.spacing = spacing;
        theme.corner_radii = corner_radii;
        theme
    }

    /// Get the builder for the dark config
    pub fn dark_config() -> Result<Config, cosmic_config::Error> {
        Config::new(DARK_THEME_BUILDER_ID, Self::VERSION)
    }

    /// Get the builder for the light config
    pub fn light_config() -> Result<Config, cosmic_config::Error> {
        Config::new(LIGHT_THEME_BUILDER_ID, Self::VERSION)
    }
}
