// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{Handle, Icon};
use std::{borrow::Cow, ffi::OsStr, path::PathBuf, sync::Arc};

#[derive(Debug, Clone, Default, Hash)]
/// Fallback icon to use if the icon was not found.
pub enum IconFallback {
    #[default]
    /// Default fallback using the icon name.
    Default,
    /// Fallback to specific icon names.
    Names(Vec<Cow<'static, str>>),
}

#[must_use]
#[derive(derive_setters::Setters, Clone, Debug, Hash)]
pub struct Named {
    /// Name of icon to locate in an XDG icon path.
    pub(super) name: Arc<str>,

    /// Checks for a fallback if the icon was not found.
    pub fallback: Option<IconFallback>,

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
        let symbolic = name.ends_with("-symbolic");
        Self {
            symbolic,
            name,
            fallback: Some(IconFallback::Default),
            size: None,
            scale: None,
            prefer_svg: symbolic,
        }
    }

    #[cfg(not(windows))]
    #[must_use]
    pub fn path(self) -> Option<PathBuf> {
        let name = &*self.name;
        let fallback = &self.fallback;
        let locate = |theme: &str, name| {
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

        let theme = crate::icon_theme::DEFAULT.lock().unwrap();
        let themes = if theme.as_ref() == crate::icon_theme::COSMIC {
            vec![theme.as_ref()]
        } else {
            vec![theme.as_ref(), crate::icon_theme::COSMIC]
        };

        let mut result = themes.iter().find_map(|t| locate(t, name));

        // On failure, attempt to locate fallback icon.
        if result.is_none() {
            if matches!(fallback, Some(IconFallback::Default)) {
                for new_name in name.rmatch_indices('-').map(|(pos, _)| &name[..pos]) {
                    result = themes.iter().find_map(|t| locate(t, new_name));
                    if result.is_some() {
                        break;
                    }
                }
            } else if let Some(IconFallback::Names(fallbacks)) = fallback {
                for fallback in fallbacks {
                    result = themes.iter().find_map(|t| locate(t, fallback));
                    if result.is_some() {
                        break;
                    }
                }
            }
        }

        result
    }

    #[cfg(windows)]
    #[must_use]
    pub fn path(self) -> Option<PathBuf> {
        //TODO: implement icon lookup for Windows
        None
    }

    #[inline]
    pub fn handle(self) -> Handle {
        let name = self.name.clone();
        Handle {
            symbolic: self.symbolic,
            data: if let Some(path) = self.path() {
                if path.extension().is_some_and(|ext| ext == OsStr::new("svg")) {
                    super::Data::Svg(iced_core::svg::Handle::from_path(path))
                } else {
                    super::Data::Image(iced_core::image::Handle::from_path(path))
                }
            } else {
                super::bundle::get(&name).unwrap_or_else(|| {
                    let bytes: &'static [u8] = &[];
                    super::Data::Svg(iced_core::svg::Handle::from_memory(bytes))
                })
            },
        }
    }

    #[inline]
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
    #[inline]
    fn from(builder: Named) -> Self {
        builder.handle()
    }
}

impl From<Named> for Icon {
    #[inline]
    fn from(builder: Named) -> Self {
        builder.icon()
    }
}

impl<Message: 'static> From<Named> for crate::Element<'_, Message> {
    #[inline]
    fn from(builder: Named) -> Self {
        builder.icon().into()
    }
}
