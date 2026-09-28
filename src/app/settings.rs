// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Configure a new COSMIC application.

use crate::{Theme, font};
use iced_core::Font;
use iced_core::layout::Limits;

/// Configure a new COSMIC application.
#[allow(clippy::struct_excessive_bools)]
#[must_use]
#[derive(derive_setters::Setters)]
pub struct Settings {
    /// Produces a smoother result in some widgets, at a performance cost.
    pub(crate) antialiasing: bool,

    /// Autosize the window to fit its contents
    #[cfg(wayland_platform)]
    pub(crate) autosize: bool,

    /// Set the application to not create a main window
    pub(crate) no_main_window: bool,

    /// Whether the window should have a border, a title bar, etc. or not.
    pub(crate) client_decorations: bool,

    /// Enables debug features in cosmic/iced.
    pub(crate) debug: bool,

    /// The default [`Font`] to be used.
    pub(crate) default_font: Font,

    /// Name of the icon theme to search by default.
    #[setters(skip)]
    pub(crate) default_icon_theme: Option<String>,

    /// Default size for widgets that inherit renderer typography. COSMIC typography
    /// presets and controls with their own sizes do not use this override.
    pub(crate) default_text_size: f32,

    /// Set the default mmap threshold for malloc with mallopt.
    pub(crate) default_mmap_threshold: Option<i32>,

    /// Whether the window should be resizable or not.
    /// and the size of the window border which can be dragged for a resize
    pub(crate) resizable: Option<f64>,

    /// Scale factor to use by default.
    pub(crate) scale_factor: f32,

    /// Initial size of the window.
    pub(crate) size: iced::Size,

    /// Limitations of the window size
    pub(crate) size_limits: Limits,

    /// The theme to apply to the application.
    pub(crate) theme: Theme,

    /// Whether the window should be transparent.
    pub(crate) transparent: bool,

    /// Whether the application window should close when the exit button is pressed
    pub(crate) exit_on_close: bool,

    /// Whether the application should act as a daemon
    pub(crate) is_daemon: bool,
}

impl Settings {
    /// Sets the default icon theme, passing an empty string will unset the theme.
    pub fn default_icon_theme(mut self, value: impl Into<String>) -> Self {
        let value: String = value.into();
        self.default_icon_theme = if value.is_empty() { None } else { Some(value) };
        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            antialiasing: true,
            #[cfg(wayland_platform)]
            autosize: false,
            no_main_window: false,
            client_decorations: true,
            debug: false,
            default_font: font::default(),
            default_icon_theme: None,
            default_text_size: f32::from(crate::config::font_size()),
            default_mmap_threshold: Some(128 * 1024),
            resizable: Some(8.0),
            scale_factor: std::env::var("COSMIC_SCALE")
                .ok()
                .and_then(|scale| scale.parse::<f32>().ok())
                .unwrap_or(1.0),
            size: iced::Size::new(1024.0, 768.0),
            size_limits: Limits::NONE.min_height(1.0).min_width(1.0),
            theme: crate::theme::system_preference(),
            transparent: true,
            exit_on_close: true,
            is_daemon: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Settings;
    use crate::{Element, config::COSMIC_TK};
    use iced::{
        Pixels, Size,
        advanced::{layout, widget::Tree},
    };

    #[test]
    fn font_size_reaches_renderer_defaults_and_preserves_app_override() {
        crate::test_process::run(
            "app::settings::tests::font_size_reaches_renderer_defaults_and_preserves_app_override",
            || {
                fn height<'a>(settings: Settings, widget: impl Into<Element<'a, ()>>) -> f32 {
                    let renderer = iced_tiny_skia::Renderer::new(
                        settings.default_font,
                        Pixels(settings.default_text_size),
                    );
                    #[cfg(feature = "wgpu")]
                    let renderer = crate::Renderer::Secondary(renderer);
                    let mut renderer = renderer;
                    let mut element = widget.into();
                    let mut tree = Tree::new(element.as_widget());
                    element
                        .as_widget_mut()
                        .layout(
                            &mut tree,
                            &mut renderer,
                            &layout::Limits::new(Size::ZERO, Size::new(1000.0, 1000.0)),
                        )
                        .size()
                        .height
                }

                COSMIC_TK.write().unwrap().font_size = 21;
                let configured_height = height(
                    Settings::default(),
                    iced::widget::text("Font size").line_height(1.5),
                );
                let override_height = height(
                    Settings::default().default_text_size(16.0),
                    iced::widget::text("Font size").line_height(1.5),
                );
                let cosmic_override_height = height(
                    Settings::default().default_text_size(16.0),
                    crate::widget::text("Font size").line_height(1.5),
                );
                COSMIC_TK.write().unwrap().font_size = 14;
                assert_eq!(configured_height, 31.5);
                assert_eq!(override_height, 24.0);
                assert_eq!(cosmic_override_height, 24.0);
                let widgets: [fn() -> Element<'static, ()>; 2] = [
                    || crate::widget::text_input("", "Font size").into(),
                    || {
                        crate::widget::toggler(false)
                            .label(String::from("Font size"))
                            .into()
                    },
                ];
                for widget in widgets {
                    let original = height(Settings::default().default_text_size(16.0), widget());
                    COSMIC_TK.write().unwrap().font_size = 21;
                    let enlarged = height(Settings::default().default_text_size(16.0), widget());
                    COSMIC_TK.write().unwrap().font_size = 14;
                    assert_eq!(
                        enlarged, original,
                        "renderer-default controls must preserve the application override"
                    );
                }
            },
        );
    }
}
