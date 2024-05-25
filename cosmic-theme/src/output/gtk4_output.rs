use crate::{composite::over, steps::steps, Component, Theme};
use palette::{rgb::Rgba, Darken, IntoColor, Lighten, Srgba};
use std::{
    fs::{self, File},
    io::Write,
    num::NonZeroUsize,
};

use super::{to_rgba, OutputError};

impl Theme {
    #[must_use]
    /// turn the theme into css
    pub fn as_gtk4(&self) -> String {
        let Self {
            background,
            primary,
            secondary,
            accent,
            destructive,
            warning,
            success,
            palette,
            ..
        } = self;

        let window_bg = to_rgba(background.base);
        let window_fg = to_rgba(background.on);

        let view_bg = to_rgba(primary.base);
        let view_fg = to_rgba(primary.on);

        let headerbar_bg = to_rgba(background.base);
        let headerbar_fg = to_rgba(background.on);
        let headerbar_border_color = to_rgba(background.divider);

        let sidebar_bg = to_rgba(primary.base);
        let sidebar_fg = to_rgba(primary.on);
        let sidebar_shade = to_rgba(if self.is_dark {
            Rgba::new(0.0, 0.0, 0.0, 0.08)
        } else {
            Rgba::new(0.0, 0.0, 0.0, 0.32)
        });
        let backdrop_overlay = Srgba::new(1.0, 1.0, 1.0, if self.is_dark { 0.08 } else { 0.32 });
        let sidebar_backdrop = to_rgba(over(backdrop_overlay, primary.base));

        let secondary_sidebar_bg = to_rgba(secondary.base);
        let secondary_sidebar_fg = to_rgba(secondary.on);
        let secondary_sidebar_shade = to_rgba(if self.is_dark {
            Rgba::new(0.0, 0.0, 0.0, 0.08)
        } else {
            Rgba::new(0.0, 0.0, 0.0, 0.32)
        });
        let secondary_sidebar_backdrop = to_rgba(over(backdrop_overlay, secondary.base));

        let headerbar_backdrop = to_rgba(background.base);

        let card_bg = to_rgba(background.component.base);
        let card_fg = to_rgba(background.component.on);

        let thumbnail_bg = to_rgba(background.component.base);
        let thumbnail_fg = to_rgba(background.component.on);

        let dialog_bg = to_rgba(primary.base);
        let dialog_fg = to_rgba(primary.on);

        let popover_bg = to_rgba(background.component.base);
        let popover_fg = to_rgba(background.component.on);

        let shade = to_rgba(if self.is_dark {
            Rgba::new(0.0, 0.0, 0.0, 0.32)
        } else {
            Rgba::new(0.0, 0.0, 0.0, 0.08)
        });

        let window_control_hover_bg = to_rgba(if self.is_dark {
            Rgba::new(255.0, 255.0, 255.0, 0.12)
        } else {
            Rgba::new(0.0, 0.0, 0.0, 0.12)
        });

        let mut inverted_bg_divider = background.base;
        inverted_bg_divider.alpha = 0.5;
        let scrollbar_outline = to_rgba(inverted_bg_divider);

        let mut css = format! {r#"
@define-color window_bg_color {window_bg};
@define-color window_fg_color {window_fg};

@define-color view_bg_color {view_bg};
@define-color view_fg_color {view_fg};

@define-color headerbar_bg_color {headerbar_bg};
@define-color headerbar_fg_color {headerbar_fg};
@define-color headerbar_border_color_color {headerbar_border_color};
@define-color headerbar_backdrop_color {headerbar_backdrop};

@define-color sidebar_bg_color {sidebar_bg};
@define-color sidebar_fg_color {sidebar_fg};
@define-color sidebar_shade_color {sidebar_shade};
@define-color sidebar_backdrop_color {sidebar_backdrop};

@define-color secondary_sidebar_bg_color {secondary_sidebar_bg};
@define-color secondary_sidebar_fg_color {secondary_sidebar_fg};
@define-color secondary_sidebar_shade_color {secondary_sidebar_shade};
@define-color secondary_sidebar_backdrop_color {secondary_sidebar_backdrop};

@define-color card_bg_color {card_bg};
@define-color card_fg_color {card_fg};

@define-color thumbnail_bg_color {thumbnail_bg};
@define-color thumbnail_fg_color {thumbnail_fg};

@define-color dialog_bg_color {dialog_bg};
@define-color dialog_fg_color {dialog_fg};

@define-color popover_bg_color {popover_bg};
@define-color popover_fg_color {popover_fg};

@define-color shade_color {shade};
@define-color scrollbar_outline_color {scrollbar_outline};

.close, .maximize, .minimize {{
	background: transparent;
}}
.close:not(:hover) > image,
.maximize:not(:hover) > image,
.minimize:not(:hover) > image {{
	background: transparent;
}}
.close > image, 
.maximize > image, 
.minimize > image {{
	color: @accent_bg_color;
	border-radius: 100%;
}}
.close:hover > image, 
.maximize:hover > image, 
.minimize:hover > image {{
	background: {window_control_hover_bg};
}}
"#};

        css.push_str(&component_gtk4_css("accent", accent));
        css.push_str(&component_gtk4_css("destructive", destructive));
        css.push_str(&component_gtk4_css("warning", warning));
        css.push_str(&component_gtk4_css("success", success));
        css.push_str(&component_gtk4_css("accent", accent));
        css.push_str(&component_gtk4_css("error", destructive));

        css.push_str(&color_css("blue", palette.blue));
        css.push_str(&color_css("green", palette.green));
        css.push_str(&color_css("yellow", palette.yellow));
        css.push_str(&color_css("red", palette.red));
        css.push_str(&color_css("orange", palette.ext_orange));
        css.push_str(&color_css("purple", palette.ext_purple));
        let neutral_steps = steps(palette.neutral_5, NonZeroUsize::new(10).unwrap());
        for (i, c) in neutral_steps[..5].iter().enumerate() {
            css.push_str(&format!("@define-color light_{i} {};\n", to_rgba(*c)));
        }
        for (i, c) in neutral_steps[5..].iter().enumerate() {
            css.push_str(&format!("@define-color dark_{i} {};\n", to_rgba(*c)));
        }
        css
    }

    /// write the CSS to the appropriate directory
    /// Should be written in the XDG config directory for gtk-4.0
    ///
    /// # Errors
    ///
    /// Returns an `OutputError` if there is an error writing the CSS file.
    pub fn write_gtk4(&self) -> Result<(), OutputError> {
        let css_str = self.as_gtk4();
        let Some(config_dir) = dirs::config_dir() else {
            return Err(OutputError::MissingConfigDir);
        };

        let name = if self.is_dark {
            "dark.css"
        } else {
            "light.css"
        };

        let config_dir = config_dir.join("gtk-4.0").join("cosmic");
        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir).map_err(OutputError::Io)?;
        }

        let mut file = File::create(config_dir.join(name)).map_err(OutputError::Io)?;
        file.write_all(css_str.as_bytes())
            .map_err(OutputError::Io)?;

        Ok(())
    }

    /// Apply gtk color variable settings
    pub fn apply_gtk(is_dark: bool) -> Result<(), OutputError> {
        let Some(config_dir) = dirs::config_dir() else {
            return Err(OutputError::MissingConfigDir);
        };

        let gtk4 = config_dir.join("gtk-4.0");
        let gtk3 = config_dir.join("gtk-3.0");

        fs::create_dir_all(&gtk4).map_err(OutputError::Io)?;
        fs::create_dir_all(&gtk3).map_err(OutputError::Io)?;

        let cosmic_css = gtk4
            .join("cosmic")
            .join(if is_dark { "dark.css" } else { "light.css" });

        let gtk4_dest = gtk4.join("gtk.css");
        let gtk3_dest = gtk3.join("gtk.css");

        #[cfg(target_family = "unix")]
        for gtk_dest in [&gtk4_dest, &gtk3_dest] {
            use std::fs::metadata;
            use std::os::unix::fs::symlink;

            let mut gtk_dest_bak = gtk_dest.clone();
            gtk_dest_bak.set_extension("css.bak");

            if gtk_dest.exists() {
                if metadata(&gtk_dest)
                    .map_err(OutputError::Io)?
                    .file_type()
                    .is_symlink()
                {
                    fs::remove_file(&gtk_dest).map_err(OutputError::Io)?;
                } else {
                    fs::rename(&gtk_dest, gtk_dest_bak).map_err(OutputError::Io)?;
                }
            }

            symlink(&cosmic_css, gtk_dest).map_err(OutputError::Io)?;
        }
        Ok(())
    }

    /// Reset the applied gtk css
    pub fn reset_gtk() -> Result<(), OutputError> {
        let Some(config_dir) = dirs::config_dir() else {
            return Err(OutputError::MissingConfigDir);
        };

        let gtk4 = config_dir.join("gtk-4.0");
        let gtk3 = config_dir.join("gtk-3.0");
        let gtk4_dest = gtk4.join("gtk.css");
        let gtk3_dest = gtk3.join("gtk.css");

        let res = fs::remove_file(gtk3_dest);
        fs::remove_file(gtk4_dest).map_err(OutputError::Io)?;
        Ok(res.map_err(OutputError::Io)?)
    }
}

fn component_gtk4_css(prefix: &str, c: &Component) -> String {
    format!(
        r#"
@define-color {prefix}_color {};
@define-color {prefix}_bg_color {};
@define-color {prefix}_fg_color {};
"#,
        to_rgba(c.base),
        to_rgba(c.base),
        to_rgba(c.on),
    )
}

fn color_css(prefix: &str, c_3: Srgba) -> String {
    let oklch: palette::Oklch = c_3.into_color();
    let c_2: Srgba = oklch.lighten(0.1).into_color();
    let c_1: Srgba = oklch.lighten(0.2).into_color();
    let c_4: Srgba = oklch.darken(0.1).into_color();
    let c_5: Srgba = oklch.darken(0.2).into_color();
    let c_1 = to_rgba(c_1);
    let c_2 = to_rgba(c_2);
    let c_3 = to_rgba(c_3);
    let c_4 = to_rgba(c_4);
    let c_5 = to_rgba(c_5);

    format! {r#"
@define-color {prefix}_1 {c_1};
@define-color {prefix}_2 {c_2};
@define-color {prefix}_3 {c_3};
@define-color {prefix}_4 {c_4};
@define-color {prefix}_5 {c_5};
"#}
}
