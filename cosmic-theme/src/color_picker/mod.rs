use crate::{Component, Container, ContainerType, Derivation, Selection, Theme, ThemeConstraints};
use anyhow::{anyhow, Result};
use palette::{IntoColor, Lcha, Shade, Srgba};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt;

pub use exact::*;
mod exact;

// TODO derive palette from Selection?
/// Color picker derives colors and theme elements
pub trait ColorPicker<
    C: Into<Srgba> + From<Srgba> + Clone + fmt::Debug + Default + Serialize + DeserializeOwned,
>
{
    /// try to derive a color with a given contrast, grayscale setting, and lightness direction
    fn pick_color(
        &self,
        color: C,
        contrast: Option<f32>,
        grayscale: bool,
        lighten: Option<bool>,
    ) -> Result<C>;

    /// try to derive a text color with a given grayscale setting, and lightness direction
    fn pick_color_text(
        &self,
        color: C,
        grayscale: bool,
        lighten: Option<bool>,
    ) -> (C, Option<anyhow::Error>);

    /// try to derive a graphic color with a given contrast, grayscale setting, and lightness direction
    fn pick_color_graphic(
        &self,
        color: C,
        contrast: f32,
        grayscale: bool,
        lighten: Option<bool>,
    ) -> (C, Option<anyhow::Error>);

    /// get the selection for this color picker
    fn get_selection(&self) -> Selection<C>;

    /// get the constraints for this color picker
    fn get_constraints(&self) -> ThemeConstraints;

    /// derive a theme from the selection and constraints
    fn theme_derivation(&self) -> Derivation<Theme<C>> {
        let mut theme_errors = Vec::new();

        let Derivation {
            derived: background,
            errors: mut errs,
        } = self.container_derivation(ContainerType::Background);
        theme_errors.append(&mut errs);

        let Derivation {
            derived: primary,
            errors: mut errs,
        } = self.container_derivation(ContainerType::Primary);
        theme_errors.append(&mut errs);

        let Derivation {
            derived: secondary,
            mut errors,
        } = self.container_derivation(ContainerType::Secondary);
        theme_errors.append(&mut errors);

        let Derivation {
            derived: accent,
            mut errors,
        } = self.widget_derivation(self.get_selection().accent);
        theme_errors.append(&mut errors);

        let Derivation {
            derived: destructive,
            mut errors,
        } = self.widget_derivation(self.get_selection().destructive);
        theme_errors.append(&mut errors);

        let Derivation {
            derived: warning,
            mut errors,
        } = self.widget_derivation(self.get_selection().warning);
        theme_errors.append(&mut errors);

        let Derivation {
            derived: success,
            mut errors,
        } = self.widget_derivation(self.get_selection().success);
        theme_errors.append(&mut errors);

        Derivation {
            derived: Theme::new(
                background,
                primary,
                secondary,
                accent,
                destructive,
                warning,
                success,
            ),
            errors: theme_errors,
        }
    }

    /// derive a container element
    fn container_derivation(&self, container_type: ContainerType) -> Derivation<Container<C>> {
        let selection = self.get_selection();
        let constraints = self.get_constraints();

        let mut errors = Vec::new();

        let Selection {
            background,
            primary_container,
            secondary_container,
            ..
        } = selection;

        let ThemeConstraints {
            elevated_contrast_ratio,
            divider_contrast_ratio,
            divider_gray_scale,
            lighten,
            ..
        } = constraints;

        let container = match container_type {
            ContainerType::Background => background,
            ContainerType::Primary => primary_container,
            ContainerType::Secondary => secondary_container,
        };
        let (container_divider, err) = self.pick_color_graphic(
            container.clone(),
            divider_contrast_ratio,
            divider_gray_scale,
            Some(lighten),
        );
        if let Some(e) = err {
            errors.push(e);
        };

        let (container_fg, err) = self.pick_color_text(container.clone(), true, None);
        if let Some(err) = err {
            let err = anyhow!("{} => \"container text\" failed: {}", container_type, err);
            errors.push(err);
        };

        // TODO revisit this and adjust constraints for transparency
        let mut container_fg_opacity_80: Srgba = container_fg.clone().into();
        container_fg_opacity_80.alpha *= 0.8;

        let (component_default, err) = self.pick_color_graphic(
            container.clone(),
            elevated_contrast_ratio,
            false,
            Some(lighten),
        );
        if let Some(e) = err {
            let err = anyhow!(
                "{} => \"container component\" failed: {}",
                container_type,
                e
            );
            errors.push(err);
        };

        let Derivation {
            derived: container_component,
            errors: errs,
        } = self.widget_derivation(component_default);
        for e in errs {
            let err = anyhow!(
                "{} => \"container component derivation\" failed: {}",
                container_type,
                e
            );
            errors.push(err);
        }

        Derivation {
            derived: Container {
                base: container,
                divider: container_divider,
                on: container_fg,
                component: container_component,
            },
            errors,
        }
    }

    /// derive a widget
    fn widget_derivation(&self, default: C) -> Derivation<Component<C>> {
        let ThemeConstraints {
            divider_contrast_ratio,
            divider_gray_scale,
            lighten,
            ..
        } = self.get_constraints();

        let mut errors = Vec::new();

        let rgba: Srgba = default.clone().into();
        let lch = Lcha {
            color: rgba.color.into_color(),
            alpha: rgba.alpha,
        };

        // TODO define constraints for different states...
        // & add color self methods and errors if these fail
        let hover = if lighten {
            lch.lighten(0.1)
        } else {
            lch.darken(0.1)
        };

        let pressed = if lighten {
            hover.lighten(0.1)
        } else {
            hover.darken(0.1)
        };
        let pressed = C::from(Srgba {
            color: pressed.color.into_color(),
            alpha: pressed.alpha,
        });

        // TODO is this actually a different color? or just outlined?
        let selected = default.clone();

        let mut disabled: Srgba = default.clone().into();
        disabled.alpha = 0.5;

        let (divider, error) = self.pick_color_graphic(
            pressed.clone(),
            divider_contrast_ratio,
            divider_gray_scale,
            Some(lighten),
        );
        if let Some(error) = error {
            errors.push(error);
        }

        let (text, error) = self.pick_color_text(pressed.clone(), true, None);
        if let Some(error) = error {
            errors.push(error);
        }

        let (selected_text, error) = self.pick_color_text(selected.clone(), true, None);
        if let Some(error) = error {
            errors.push(error);
        }

        let mut text_opacity_80: Srgba = text.clone().into();
        text_opacity_80.alpha = 0.8;

        let mut disabled_fg = text.clone().into();
        disabled_fg.alpha = 0.5;

        Derivation {
            derived: Component {
                base: default,
                hover: C::from(Srgba {
                    color: hover.color.into_color(),
                    alpha: hover.alpha,
                }),
                pressed,
                selected: selected.clone(),
                selected_text: selected_text,
                focus: selected.clone(), // FIXME
                divider,
                on: text,
                disabled: disabled.into(),
                on_disabled: disabled_fg.into(),
            },
            errors,
        }
    }
}
