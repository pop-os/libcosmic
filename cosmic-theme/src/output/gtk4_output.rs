use crate::{
    model::{Accent, Container, ContainerType, Destructive, Widget},
    Hex, Theme, NAME,
};
use anyhow::{bail, Result};
use palette::Srgba;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt, fs::File, io::prelude::*, path::PathBuf};

pub(crate) const CSS_DIR: &'static str = "css";
pub(crate) const THEME_DIR: &'static str = "themes";

/// Trait for outputting the Theme variables as Gtk4CSS
pub trait Gtk4Output {
    /// turn the theme into css
    fn as_css(&self) -> String;
    /// Serialize the theme as RON and write the CSS to the appropriate directories
    /// Should be written in the XDG data directory for cosmic-theme
    fn write(&self) -> Result<()>;
}

impl<C> Gtk4Output for Theme<C>
where
    C: Clone
        + fmt::Debug
        + Default
        + Into<Hex>
        + Into<Srgba>
        + From<Srgba>
        + Serialize
        + DeserializeOwned,
{
    fn as_css(&self) -> String {
        let Self {
            background,
            primary,
            secondary,
            accent,
            destructive,
            ..
        } = self;
        let mut css = String::new();

        css.push_str(&background.as_css());
        css.push_str(&primary.as_css());
        css.push_str(&secondary.as_css());
        css.push_str(&accent.as_css());
        css.push_str(&destructive.as_css());

        css
    }

    fn write(&self) -> Result<()> {
        // TODO sass -> css
        let ron_str = ron::ser::to_string_pretty(self, Default::default())?;
        let css_str = self.as_css();

        let ron_path: PathBuf = [NAME, THEME_DIR].iter().collect();
        let css_path: PathBuf = [NAME, CSS_DIR].iter().collect();

        let ron_dirs = xdg::BaseDirectories::with_prefix(ron_path)?;
        let css_dirs = xdg::BaseDirectories::with_prefix(css_path)?;

        let ron_name = format!("{}.ron", &self.name);
        let css_name = format!("{}.css", &self.name);

        if let Ok(p) = ron_dirs.place_data_file(ron_name) {
            let mut f = File::create(p)?;
            f.write_all(ron_str.as_bytes())?;
        } else {
            bail!("Failed to write RON theme.")
        }

        if let Ok(p) = css_dirs.place_data_file(css_name) {
            let mut f = File::create(p)?;
            f.write_all(css_str.as_bytes())?;
        } else {
            bail!("Failed to write RON theme.")
        }

        Ok(())
    }
}

/// Trait for converting theme data into gtk4 CSS
pub trait AsGtk4Css<C>
where
    C: Copy + Into<Srgba> + From<Srgba>,
{
    /// function for converting theme data into gtk4 CSS
    fn as_css(&self) -> String;
}

impl<C> AsGtk4Css<C> for Container<C>
where
    C: Copy + Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + fmt::Display,
{
    fn as_css(&self) -> String {
        let Self {
            prefix,
            container,
            container_component,
            container_divider,
            container_fg,
            ..
        } = self;

        let prefix_lower = match prefix {
            ContainerType::Background => "background",
            ContainerType::Primary => "primary",
            ContainerType::Secondary => "secondary",
        };
        let component = widget_gtk4_css(prefix_lower, container_component);

        format!(
            r#"
@define-color {prefix_lower}_container #{{{container}}};
@define-color {prefix_lower}_container_divider #{{{container_divider}}};
@define-color {prefix_lower}_container_fg #{{{container_fg}}};
{component}
"#
        )
    }
}

impl<C> AsGtk4Css<C> for Accent<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    fn as_css(&self) -> String {
        let Accent {
            accent,
            accent_fg,
            accent_nav_handle_fg,
            suggested,
        } = self;
        let suggested = widget_gtk4_css("suggested", suggested);

        format!(
            r#"
@define-color accent #{{{accent}}};
@define-color accent_fg #{{{accent_fg}}};
@define-color accent_nav_handle_fg #{{{accent_nav_handle_fg}}};
{suggested}
"#
        )
    }
}

impl<C> AsGtk4Css<C> for Destructive<C>
where
    C: Clone + fmt::Debug + Default + Into<Srgba> + From<Srgba> + Serialize + DeserializeOwned,
{
    fn as_css(&self) -> String {
        let Destructive { destructive } = &self;
        widget_gtk4_css("destructive", destructive)
    }
}

fn widget_gtk4_css<C: fmt::Display>(
    prefix: &str,
    Widget {
        base,
        hover,
        pressed,
        focused,
        divider,
        text,
        text_opacity_80,
        disabled,
        disabled_fg,
    }: &Widget<C>,
) -> String {
    format!(
        r#"
@define-color {prefix}_widget_base #{{{base}}};
@define-color {prefix}_widget_hover #{{{hover}}};
@define-color {prefix}_widget_pressed #{{{pressed}}};
@define-color {prefix}_widget_focused #{{{focused}}};
@define-color {prefix}_widget_divider #{{{divider}}};
@define-color {prefix}_widget_fg #{{{text}}};
@define-color {prefix}_widget_fg_opacity_80 #{{{text_opacity_80}}};
@define-color {prefix}_widget_disabled #{{{disabled}}};
@define-color {prefix}_widget_disabled_fg #{{{disabled_fg}}};
"#
    )
}
