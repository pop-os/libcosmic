use crate::{
    composite::over, steps::*, Component, Container, CornerRadii, CosmicPalette,
    CosmicPaletteInner, Spacing, ThemeMode, DARK_PALETTE, LIGHT_PALETTE, NAME,
};
use cosmic_config::{Config, CosmicConfigEntry};
use palette::{IntoColor, Srgb, Srgba};
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

/// ID for the current dark ThemeBuilder config
pub const DARK_THEME_BUILDER_ID: &str = "com.system76.CosmicTheme.Dark.Builder";

/// ID for the current dark Theme config
pub const DARK_THEME_ID: &str = "com.system76.CosmicTheme.Dark";

/// ID for the current light ThemeBuilder config
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

/// Cosmic Theme data structure with all colors and its name
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    cosmic_config::cosmic_config_derive::CosmicConfigEntry,
)]
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
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark_default()
    }
}

/// Trait for layered themes
pub trait LayeredTheme {
    /// Set the layer of the theme
    fn set_layer(&mut self, layer: Layer);
}

impl Theme {
    /// version of the theme
    pub fn version() -> u64 {
        1
    }

    /// id of the theme
    pub fn id() -> &'static str {
        NAME
    }

    /// Get the config for the current dark theme
    pub fn dark_config() -> Result<Config, cosmic_config::Error> {
        Config::new(DARK_THEME_ID, Self::version())
    }

    /// Get the config for the current light theme
    pub fn light_config() -> Result<Config, cosmic_config::Error> {
        Config::new(LIGHT_THEME_ID, Self::version())
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
    /// get @accent_color
    pub fn accent_color(&self) -> Srgba {
        self.accent.base.clone()
    }
    /// get @success_color
    pub fn success_color(&self) -> Srgba {
        self.success.base.clone()
    }
    /// get @destructive_color
    pub fn destructive_color(&self) -> Srgba {
        self.destructive.base.clone()
    }
    /// get @warning_color
    pub fn warning_color(&self) -> Srgba {
        self.warning.base.clone()
    }

    // Containers
    /// get @bg_color
    pub fn bg_color(&self) -> Srgba {
        self.background.base.clone()
    }
    /// get @bg_component_color
    pub fn bg_component_color(&self) -> Srgba {
        self.background.component.base.clone()
    }
    /// get @primary_container_color
    pub fn primary_container_color(&self) -> Srgba {
        self.primary.base.clone()
    }
    /// get @primary_component_color
    pub fn primary_component_color(&self) -> Srgba {
        self.primary.component.base.clone()
    }
    /// get @secondary_container_color
    pub fn secondary_container_color(&self) -> Srgba {
        self.secondary.base.clone()
    }
    /// get @secondary_component_color
    pub fn secondary_component_color(&self) -> Srgba {
        self.secondary.component.base.clone()
    }
    /// get @button_bg_color
    pub fn button_bg_color(&self) -> Srgba {
        self.button.base.clone()
    }

    // Text
    /// get @on_bg_color
    pub fn on_bg_color(&self) -> Srgba {
        self.background.on.clone()
    }
    /// get @on_bg_component_color
    pub fn on_bg_component_color(&self) -> Srgba {
        self.background.component.on.clone()
    }
    /// get @on_primary_color
    pub fn on_primary_container_color(&self) -> Srgba {
        self.primary.on.clone()
    }
    /// get @on_primary_component_color
    pub fn on_primary_component_color(&self) -> Srgba {
        self.primary.component.on.clone()
    }
    /// get @on_secondary_color
    pub fn on_secondary_container_color(&self) -> Srgba {
        self.secondary.on.clone()
    }
    /// get @on_secondary_component_color
    pub fn on_secondary_component_color(&self) -> Srgba {
        self.secondary.component.on.clone()
    }
    /// get @accent_text_color
    pub fn accent_text_color(&self) -> Srgba {
        self.accent.base.clone()
    }
    /// get @success_text_color
    pub fn success_text_color(&self) -> Srgba {
        self.success.base.clone()
    }
    /// get @warning_text_color
    pub fn warning_text_color(&self) -> Srgba {
        self.warning.base.clone()
    }
    /// get @destructive_text_color
    pub fn destructive_text_color(&self) -> Srgba {
        self.destructive.base.clone()
    }
    /// get @on_accent_color
    pub fn on_accent_color(&self) -> Srgba {
        self.accent.on.clone()
    }
    /// get @on_success_color
    pub fn on_success_color(&self) -> Srgba {
        self.success.on.clone()
    }
    /// get @oon_warning_color
    pub fn on_warning_color(&self) -> Srgba {
        self.warning.on.clone()
    }
    /// get @on_destructive_color
    pub fn on_destructive_color(&self) -> Srgba {
        self.destructive.on.clone()
    }
    /// get @button_color
    pub fn button_color(&self) -> Srgba {
        self.button.on.clone()
    }

    // Borders and Dividers
    /// get @bg_divider
    pub fn bg_divider(&self) -> Srgba {
        self.background.divider.clone()
    }
    /// get @bg_component_divider
    pub fn bg_component_divider(&self) -> Srgba {
        self.background.component.divider.clone()
    }
    /// get @primary_container_divider
    pub fn primary_container_divider(&self) -> Srgba {
        self.primary.divider.clone()
    }
    /// get @primary_component_divider
    pub fn primary_component_divider(&self) -> Srgba {
        self.primary.component.divider.clone()
    }
    /// get @secondary_container_divider
    pub fn secondary_container_divider(&self) -> Srgba {
        self.secondary.divider.clone()
    }
    /// get @button_divider
    pub fn button_divider(&self) -> Srgba {
        self.button.divider.clone()
    }

    /// get @window_header_bg
    pub fn window_header_bg(&self) -> Srgba {
        self.background.base.clone()
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
    pub fn radius_0(&self) -> [f32; 4] {
        self.corner_radii.radius_0
    }
    /// get @radius_xs
    pub fn radius_xs(&self) -> [f32; 4] {
        self.corner_radii.radius_xs
    }
    /// get @radius_s
    pub fn radius_s(&self) -> [f32; 4] {
        self.corner_radii.radius_s
    }
    /// get @radius_m
    pub fn radius_m(&self) -> [f32; 4] {
        self.corner_radii.radius_m
    }
    /// get @radius_l
    pub fn radius_l(&self) -> [f32; 4] {
        self.corner_radii.radius_l
    }
    /// get @radius_xl
    pub fn radius_xl(&self) -> [f32; 4] {
        self.corner_radii.radius_xl
    }

    /// get the active theme
    pub fn get_active() -> Result<Self, (Vec<cosmic_config::Error>, Self)> {
        let config =
            Config::new(Self::id(), Self::version()).map_err(|e| (vec![e], Self::default()))?;
        let is_dark = ThemeMode::is_dark(&config).map_err(|e| (vec![e], Self::default()))?;
        let config = if is_dark {
            Self::dark_config()
        } else {
            Self::light_config()
        }
        .map_err(|e| (vec![e], Self::default()))?;
        Self::get_entry(&config)
    }
}

impl From<CosmicPalette> for Theme {
    fn from(p: CosmicPalette) -> Self {
        ThemeBuilder::palette(p).build()
    }
}

/// Helper for building customized themes
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    cosmic_config::cosmic_config_derive::CosmicConfigEntry,
    PartialEq,
)]
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
            gaps: (0, 4),
            active_hint: 4,
            window_hint: None,
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
        let palette: CosmicPalette = DARK_PALETTE.to_owned().into();
        Self {
            palette: CosmicPalette::HighContrastDark(palette.inner()),
            ..Default::default()
        }
    }

    /// Get a builder that is initialized with the default light high contrast theme
    pub fn light_high_contrast() -> Self {
        let palette: CosmicPalette = LIGHT_PALETTE.to_owned().into();
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
            palette.as_ref().blue.to_owned()
        };

        let success = if let Some(success) = success {
            success.into_color()
        } else {
            palette.as_ref().green.to_owned()
        };

        let warning = if let Some(warning) = warning {
            warning.into_color()
        } else {
            palette.as_ref().yellow.to_owned()
        };

        let destructive = if let Some(destructive) = destructive {
            destructive.into_color()
        } else {
            palette.as_ref().red.to_owned()
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

        let bg = if let Some(bg_color) = bg_color {
            bg_color
        } else {
            p_ref.gray_1.clone()
        };

        let step_array = steps(bg, NonZeroUsize::new(100).unwrap());
        let bg_index = color_index(bg, step_array.len());

        let primary_container_bg = if let Some(primary_container_bg_color) = primary_container_bg {
            primary_container_bg_color
        } else {
            get_surface_color(bg_index, 5, &step_array, is_dark, &p_ref.neutral_1)
        };

        let secondary_container_bg = if let Some(secondary_container_bg) = secondary_container_bg {
            secondary_container_bg
        } else {
            get_surface_color(bg_index, 10, &step_array, is_dark, &p_ref.neutral_2)
        };

        let bg_component = get_surface_color(bg_index, 8, &step_array, is_dark, &p_ref.neutral_2);
        let on_bg_component = get_text(
            color_index(bg_component, step_array.len()),
            &step_array,
            is_dark,
            &p_ref.neutral_8,
            text_steps_array.as_ref(),
        );

        let mut component_hovered_overlay = p_ref.neutral_0.clone();
        component_hovered_overlay.alpha = 0.1;

        let mut component_pressed_overlay = p_ref.neutral_0.clone();
        component_pressed_overlay.alpha = 0.2;

        let bg_component = Component::component(
            bg_component,
            accent,
            on_bg_component,
            component_hovered_overlay,
            component_pressed_overlay,
            is_high_contrast,
            p_ref.neutral_8,
        );

        let primary_index = color_index(primary_container_bg, step_array.len());
        let primary_component =
            get_surface_color(primary_index, 6, &step_array, is_dark, &p_ref.neutral_3);
        let on_primary_component = get_text(
            color_index(primary_component, step_array.len()),
            &step_array,
            is_dark,
            &p_ref.neutral_8,
            text_steps_array.as_ref(),
        );
        let primary_component = Component::component(
            primary_component,
            accent,
            on_primary_component,
            component_hovered_overlay,
            component_pressed_overlay,
            is_high_contrast,
            p_ref.neutral_8,
        );

        let secondary_index = color_index(secondary_container_bg, step_array.len());
        let secondary_component =
            get_surface_color(secondary_index, 3, &step_array, is_dark, &p_ref.neutral_4);
        let on_secondary_component = get_text(
            color_index(secondary_component, step_array.len()),
            &step_array,
            is_dark,
            &p_ref.neutral_8,
            text_steps_array.as_ref(),
        );
        let secondary_component = Component::component(
            secondary_component,
            accent,
            on_secondary_component,
            component_hovered_overlay,
            component_pressed_overlay,
            is_high_contrast,
            p_ref.neutral_8,
        );
        let neutral_7 = p_ref.neutral_7;

        // Standard button background is neutral 7 with 25% opacity
        let button_bg = {
            let mut color = neutral_7.clone();
            color.alpha = 0.25;
            color
        };

        let (mut button_hovered_overlay, mut button_pressed_overlay) =
            (p_ref.neutral_5, p_ref.neutral_2);
        button_hovered_overlay.alpha = 0.2;
        button_pressed_overlay.alpha = 0.5;

        let mut theme: Theme = Theme {
            name: palette.name().to_string(),
            primary: Container::new(
                primary_component,
                primary_container_bg,
                get_text(
                    primary_index,
                    &step_array,
                    is_dark,
                    &p_ref.neutral_8,
                    text_steps_array.as_ref(),
                ),
            ),
            secondary: Container::new(
                secondary_component,
                secondary_container_bg,
                get_text(
                    secondary_index,
                    &step_array,
                    is_dark,
                    &p_ref.neutral_8,
                    text_steps_array.as_ref(),
                ),
            ),
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
            background: Container::new(
                bg_component,
                bg,
                get_text(
                    bg_index,
                    &step_array,
                    is_dark,
                    &p_ref.neutral_8,
                    text_steps_array.as_ref(),
                ),
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

                let mut on_50 = component.on.clone();
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
        Config::new(DARK_THEME_BUILDER_ID, Self::version())
    }

    /// Get the builder for the light config
    pub fn light_config() -> Result<Config, cosmic_config::Error> {
        Config::new(LIGHT_THEME_BUILDER_ID, Self::version())
    }

    /// version of the theme builder
    pub fn version() -> u64 {
        1
    }
}
