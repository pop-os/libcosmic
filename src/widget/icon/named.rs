// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{Handle, Icon};
use std::{path::PathBuf, sync::Arc};

#[must_use]
#[derive(derive_setters::Setters, Clone, Debug, Hash)]
pub struct Named {
    /// Name of icon to locate in an XDG icon path.
    pub(super) name: Arc<str>,

    /// Checks for a fallback if the icon was not found.
    pub fallback: bool,

    /// Restrict the lookup to a given scale.
    #[setters(strip_option)]
    pub scale: Option<u16>,

    /// Restrict the lookup to a given size.
    #[setters(strip_option)]
    pub size: Option<u16>,

    /// Whether the icon is symbolic or not.
    pub symbolic: bool,

    /// Prioritizes SVG over PNG
    pub prefer_svg: bool,
}

impl Named {
    pub fn new(name: impl Into<Arc<str>>) -> Self {
        let name = name.into();
        Self {
            symbolic: name.ends_with("-symbolic"),
            name,
            fallback: true,
            size: None,
            scale: None,
            prefer_svg: false,
        }
    }

    #[cfg(not(windows))]
    #[must_use]
    pub fn path(self) -> Option<PathBuf> {
        let mut name = &*self.name;
        crate::icon_theme::DEFAULT.with(|theme| {
            let theme = theme.borrow();

            let locate = || {
                let mut lookup = freedesktop_icons::lookup(name)
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
                for new_name in name.rmatch_indices('-').map(|(pos, _)| &name[..pos]) {
                    name = new_name;
                    result = locate();
                    if result.is_some() {
                        break;
                    }
                }
            }

            result
        })
    }

    #[cfg(windows)]
    #[must_use]
    pub fn path(self) -> Option<PathBuf> {
        //TODO: implement icon lookup for Windows
        None
    }

    pub fn handle(self) -> Handle {
        Handle {
            symbolic: self.symbolic,
            data: super::Data::Name(self),
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

impl From<Named> for Handle {
    fn from(builder: Named) -> Self {
        builder.handle()
    }
}

impl From<Named> for Icon {
    fn from(builder: Named) -> Self {
        builder.icon()
    }
}

impl<'a, Message: 'static> From<Named> for crate::Element<'a, Message> {
    fn from(builder: Named) -> Self {
        builder.icon().into()
    }
}
