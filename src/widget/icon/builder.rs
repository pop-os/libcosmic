// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{handle, Handle, Icon};
use std::path::PathBuf;

#[must_use]
#[derive(derive_setters::Setters)]
pub struct Builder<'a> {
    /// Name of icon to locate in an XDG icon path.
    name: &'a str,

    /// Checks for a fallback if the icon was not found.
    fallback: bool,

    /// Restrict the lookup to a given scale.
    #[setters(strip_option)]
    scale: Option<u16>,

    /// Restrict the lookup to a given size.
    #[setters(strip_option)]
    size: Option<u16>,

    /// Prioritizes SVG over PNG
    prefer_svg: bool,
}

impl<'a> Builder<'a> {
    pub const fn new(name: &'a str) -> Self {
        Self {
            name,
            fallback: true,
            size: None,
            scale: None,
            prefer_svg: false,
        }
    }

    #[must_use]
    pub fn path(mut self) -> Option<PathBuf> {
        crate::icon_theme::DEFAULT.with(|theme| {
            let theme = theme.borrow();

            let locate = || {
                let mut lookup = freedesktop_icons::lookup(self.name)
                    .with_theme(theme.as_ref())
                    .with_cache();

                if let Some(scale) = self.scale {
                    lookup = lookup.with_scale(scale);
                }

                if let Some(size) = self.size {
                    lookup = lookup.with_size(size);
                }

                if self.prefer_svg {
                    lookup = lookup.force_svg();
                }

                lookup.find()
            };

            let mut result = locate();

            // On failure, attempt to locate fallback icon.
            if result.is_none() && self.fallback {
                let name = std::mem::take(&mut self.name);

                for name in name.rmatch_indices('-').map(|(pos, _)| &name[..pos]) {
                    self.name = name;
                    result = locate();
                    if result.is_some() {
                        break;
                    }
                }
            }

            result
        })
    }

    pub fn handle(self) -> Handle {
        if let Some(path) = self.path() {
            handle::from_path(path)
        } else {
            let bytes: &'static [u8] = &[];
            handle::from_svg_bytes(bytes)
        }
    }

    pub fn icon(self) -> Icon {
        let size = self.size;

        let icon = super::icon(self.handle());

        match size {
            Some(size) => icon.size(size),
            None => icon,
        }
    }
}

impl<'a> From<Builder<'a>> for Handle {
    fn from(builder: Builder<'a>) -> Self {
        builder.handle()
    }
}

impl<'a> From<Builder<'a>> for Icon {
    fn from(builder: Builder<'a>) -> Self {
        builder.icon()
    }
}
