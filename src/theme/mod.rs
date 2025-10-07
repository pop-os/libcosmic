// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Contains the [`Theme`] type and its widget stylesheet implementations.

#[cfg(feature = "xdg-portal")]
pub mod portal;
pub mod style;

use cosmic_config::CosmicConfigEntry;
use cosmic_config::config_subscription;
use cosmic_theme::Component;
use cosmic_theme::LayeredTheme;
use cosmic_theme::Spacing;
use cosmic_theme::ThemeMode;
use iced_futures::Subscription;
use iced_runtime::{Appearance, DefaultStyle};
use std::sync::{Arc, LazyLock, Mutex};
pub use style::*;

pub type CosmicColor = ::palette::rgb::Srgba;
pub type CosmicComponent = cosmic_theme::Component;
pub type CosmicTheme = cosmic_theme::Theme;

pub static COSMIC_DARK: LazyLock<CosmicTheme> = LazyLock::new(CosmicTheme::dark_default);

pub static COSMIC_HC_DARK: LazyLock<CosmicTheme> =
    LazyLock::new(CosmicTheme::high_contrast_dark_default);

pub static COSMIC_LIGHT: LazyLock<CosmicTheme> = LazyLock::new(CosmicTheme::light_default);

pub static COSMIC_HC_LIGHT: LazyLock<CosmicTheme> =
    LazyLock::new(CosmicTheme::high_contrast_light_default);

pub static TRANSPARENT_COMPONENT: LazyLock<Component> = LazyLock::new(|| Component {
    base: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
    hover: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
    pressed: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
    selected: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
    selected_text: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
    focus: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
    disabled: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
    on: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
    on_disabled: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
    divider: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
    border: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
    disabled_border: CosmicColor::new(0.0, 0.0, 0.0, 0.0),
});

pub(crate) static THEME: Mutex<Theme> = Mutex::new(Theme {
    theme_type: ThemeType::Dark,
    layer: cosmic_theme::Layer::Background,
});

/// Currently-defined theme.
#[inline]
#[allow(clippy::missing_panics_doc)]
pub fn active() -> Theme {
    THEME.lock().unwrap().clone()
}

/// Currently-defined theme type.
#[inline]
#[allow(clippy::missing_panics_doc)]
pub fn active_type() -> ThemeType {
    THEME.lock().unwrap().theme_type.clone()
}

/// Preferred interface spacing parameters defined by the active theme.
#[inline]
pub fn spacing() -> Spacing {
    active().cosmic().spacing
}

/// Whether the active theme has a dark preference.
#[inline]
#[must_use]
pub fn is_dark() -> bool {
    active_type().is_dark()
}

/// Whether the active theme is high contrast.
#[inline]
#[must_use]
pub fn is_high_contrast() -> bool {
    active_type().is_high_contrast()
}

// /// Watches for changes to the system's theme preference.
// #[cold]
// pub fn subscription(is_dark: bool) -> Subscription<crate::theme::Theme> {
//     config_subscription::<_, crate::cosmic_theme::Theme>(
//         (
//             std::any::TypeId::of::<crate::cosmic_theme::Theme>(),
//             is_dark,
//         ),
//         if is_dark {
//             cosmic_theme::DARK_THEME_ID
//         } else {
//             cosmic_theme::LIGHT_THEME_ID
//         }
//         .into(),
//         crate::cosmic_theme::Theme::VERSION,
//     )
//     .map(|res| {
//         for error in res.errors.into_iter().filter(cosmic_config::Error::is_err) {
//             tracing::error!(
//                 ?error,
//                 "error while watching system theme preference changes"
//             );
//         }

//         Theme::system(Arc::new(res.config))
//     })
// }

pub fn system_dark() -> Theme {
    let Ok(helper) = crate::cosmic_theme::Theme::dark_config() else {
        return Theme::dark();
    };

    let t = crate::cosmic_theme::Theme::get_entry(&helper).unwrap_or_else(|(errors, theme)| {
        for error in errors.into_iter().filter(cosmic_config::Error::is_err) {
            tracing::error!(?error, "error loading system dark theme");
        }
        theme
    });

    Theme::system(Arc::new(t))
}

pub fn system_light() -> Theme {
    let Ok(helper) = crate::cosmic_theme::Theme::light_config() else {
        return Theme::light();
    };

    let t = crate::cosmic_theme::Theme::get_entry(&helper).unwrap_or_else(|(errors, theme)| {
        for error in errors.into_iter().filter(cosmic_config::Error::is_err) {
            tracing::error!(?error, "error loading system light theme");
        }
        theme
    });

    Theme::system(Arc::new(t))
}

/// Loads the preferred system theme from `cosmic-config`.
pub fn system_preference() -> Theme {
    let Ok(mode_config) = ThemeMode::config() else {
        return Theme::dark();
    };

    let Ok(is_dark) = ThemeMode::is_dark(&mode_config) else {
        return Theme::dark();
    };
    if is_dark {
        system_dark()
    } else {
        system_light()
    }
}

#[must_use]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ThemeType {
    #[default]
    Dark,
    Light,
    HighContrastDark,
    HighContrastLight,
    Custom(Arc<CosmicTheme>),
    System {
        prefer_dark: Option<bool>,
        theme: Arc<CosmicTheme>,
    },
}

impl ThemeType {
    /// Whether the theme has a dark preference.
    #[must_use]
    #[inline]
    pub fn is_dark(&self) -> bool {
        match self {
            Self::Dark | Self::HighContrastDark => true,
            Self::Light | Self::HighContrastLight => false,
            Self::Custom(theme) | Self::System { theme, .. } => theme.is_dark,
        }
    }

    /// Whether the theme has a high contrast.
    #[inline]
    #[must_use]
    pub fn is_high_contrast(&self) -> bool {
        match self {
            Self::Dark | Self::Light => false,
            Self::HighContrastDark | Self::HighContrastLight => true,
            Self::Custom(theme) | Self::System { theme, .. } => theme.is_high_contrast,
        }
    }

    #[inline]
    /// Prefer dark or light theme.
    /// If `None`, the system preference is used.
    pub fn prefer_dark(&mut self, new_prefer_dark: Option<bool>) {
        if let Self::System { prefer_dark, .. } = self {
            *prefer_dark = new_prefer_dark;
        }
    }
}

#[must_use]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Theme {
    pub theme_type: ThemeType,
    pub layer: cosmic_theme::Layer,
}

impl Theme {
    #[inline]
    pub fn cosmic(&self) -> &cosmic_theme::Theme {
        match self.theme_type {
            ThemeType::Dark => &COSMIC_DARK,
            ThemeType::Light => &COSMIC_LIGHT,
            ThemeType::HighContrastDark => &COSMIC_HC_DARK,
            ThemeType::HighContrastLight => &COSMIC_HC_LIGHT,
            ThemeType::Custom(ref t) | ThemeType::System { theme: ref t, .. } => t.as_ref(),
        }
    }

    #[inline]
    pub fn dark() -> Self {
        Self {
            theme_type: ThemeType::Dark,
            ..Default::default()
        }
    }

    #[inline]
    pub fn light() -> Self {
        Self {
            theme_type: ThemeType::Light,
            ..Default::default()
        }
    }

    #[inline]
    pub fn dark_hc() -> Self {
        Self {
            theme_type: ThemeType::HighContrastDark,
            ..Default::default()
        }
    }

    #[inline]
    pub fn light_hc() -> Self {
        Self {
            theme_type: ThemeType::HighContrastLight,
            ..Default::default()
        }
    }

    #[inline]
    pub fn custom(theme: Arc<CosmicTheme>) -> Self {
        Self {
            theme_type: ThemeType::Custom(theme),
            ..Default::default()
        }
    }

    #[inline]
    pub fn system(theme: Arc<CosmicTheme>) -> Self {
        Self {
            theme_type: ThemeType::System {
                theme,
                prefer_dark: None,
            },
            ..Default::default()
        }
    }

    #[inline]
    /// get current container
    /// can be used in a component that is intended to be a child of a `CosmicContainer`
    pub fn current_container(&self) -> &cosmic_theme::Container {
        match self.layer {
            cosmic_theme::Layer::Background => &self.cosmic().background,
            cosmic_theme::Layer::Primary => &self.cosmic().primary,
            cosmic_theme::Layer::Secondary => &self.cosmic().secondary,
        }
    }

    #[inline]
    /// set the theme
    pub fn set_theme(&mut self, theme: ThemeType) {
        self.theme_type = theme;
    }
}

impl LayeredTheme for Theme {
    #[inline]
    fn set_layer(&mut self, layer: cosmic_theme::Layer) {
        self.layer = layer;
    }
}

impl DefaultStyle for Theme {
    fn default_style(&self) -> Appearance {
        let cosmic = self.cosmic();
        Appearance {
            icon_color: cosmic.bg_color().into(),
            background_color: cosmic.bg_color().into(),
            text_color: cosmic.on_bg_color().into(),
        }
    }
}
