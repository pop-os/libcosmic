// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Lazily-generated SVG icon widget for Iced.

use crate::{Element, Renderer};
use derive_setters::Setters;
use iced::{
    widget::{image, svg, Image},
    ContentFit, Length,
};
use std::{
    borrow::Cow, collections::hash_map::DefaultHasher, ffi::OsStr, hash::Hash, hash::Hasher,
    path::Path, path::PathBuf,
};

pub enum Handle {
    Image(image::Handle),
    Svg(svg::Handle),
}

#[derive(Debug, Hash)]
pub enum IconSource<'a> {
    Path(Cow<'a, Path>),
    Name(Cow<'a, str>),
    Embedded(Image),
}

impl<'a> IconSource<'a> {
    /// Loads the icon as either an image or svg [`Handle`].
    #[must_use]
    pub fn load(&self, size: u16, theme: Option<&str>, svg: bool) -> Handle {
        let name_path_buffer: Option<PathBuf>;
        let icon: Option<&Path> = match self {
            IconSource::Path(path) => Some(path),
            IconSource::Name(name) => {
                let icon = crate::settings::DEFAULT_ICON_THEME.with(|default_theme| {
                    let default_theme: &str = &default_theme.borrow();
                    freedesktop_icons::lookup(name)
                        .with_size(size)
                        .with_theme(theme.unwrap_or(default_theme))
                        .with_cache()
                        .find()
                });

                name_path_buffer = if icon.is_none() {
                    freedesktop_icons::lookup(name)
                        .with_size(size)
                        .with_cache()
                        .find()
                } else {
                    icon
                };

                name_path_buffer.as_deref()
            }
            IconSource::Embedded(_) => unimplemented!(),
        };

        let is_svg = svg
            || icon
                .as_ref()
                .map_or(true, |path| path.extension() == Some(OsStr::new("svg")));

        if is_svg {
            let handle = if let Some(path) = icon {
                svg::Handle::from_path(path)
            } else {
                eprintln!("svg icon '{:?}' size {} not found", self, size);
                svg::Handle::from_memory(Vec::new())
            };

            Handle::Svg(handle)
        } else if let Some(icon) = icon {
            Handle::Image(icon.into())
        } else {
            eprintln!("icon '{:?}' size {} not found", self, size);
            Handle::Image(image::Handle::from_memory(Vec::new()))
        }
    }
}

impl<'a> From<Cow<'a, Path>> for IconSource<'a> {
    fn from(value: Cow<'a, Path>) -> Self {
        Self::Path(value)
    }
}

impl From<PathBuf> for IconSource<'static> {
    fn from(value: PathBuf) -> Self {
        Self::Path(Cow::Owned(value))
    }
}

impl<'a> From<&'a Path> for IconSource<'a> {
    fn from(value: &'a Path) -> Self {
        Self::Path(Cow::Borrowed(value))
    }
}

impl<'a> From<Cow<'a, str>> for IconSource<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self::Name(value)
    }
}

impl From<String> for IconSource<'static> {
    fn from(value: String) -> Self {
        Self::Name(value.into())
    }
}

impl<'a> From<&'a str> for IconSource<'a> {
    fn from(value: &'a str) -> Self {
        Self::Name(value.into())
    }
}

impl<'a> From<Image> for IconSource<'a> {
    fn from(value: Image) -> Self {
        Self::Embedded(value)
    }
}

/// A lazily-generated icon.
#[derive(Hash, Setters)]
pub struct Icon<'a> {
    #[setters(skip)]
    source: IconSource<'a>,
    #[setters(strip_option, into)]
    theme: Option<Cow<'a, str>>,
    style: crate::theme::Svg,
    size: u16,
    #[setters(strip_option)]
    content_fit: Option<ContentFit>,
    #[setters(strip_option)]
    width: Option<Length>,
    #[setters(strip_option)]
    height: Option<Length>,
    force_svg: bool,
}

/// A lazily-generated icon.
#[must_use]
pub fn icon<'a>(source: impl Into<IconSource<'a>>, size: u16) -> Icon<'a> {
    Icon {
        content_fit: None,
        height: None,
        source: source.into(),
        size,
        style: crate::theme::Svg::default(),
        theme: None,
        width: None,
        force_svg: false,
    }
}

impl<'a> Icon<'a> {
    #[must_use]
    fn into_element<Message: 'static>(self) -> Element<'a, Message> {
        if let IconSource::Embedded(mut image) = self.source {
            image = image
                .width(self.width.unwrap_or(Length::Units(self.size)))
                .height(self.height.unwrap_or(Length::Units(self.size)));
            if let Some(content_fit) = self.content_fit {
                image = image.content_fit(content_fit);
            }
            return image.into();
        }

        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);

        if self.theme.is_none() {
            crate::settings::DEFAULT_ICON_THEME.with(|f| f.borrow().hash(&mut hasher));
        }

        let hash = hasher.finish();

        iced_lazy::lazy(hash, move || -> Element<Message> {
            match self
                .source
                .load(self.size, self.theme.as_deref(), self.force_svg)
            {
                Handle::Svg(handle) => {
                    let mut widget = svg::Svg::<Renderer>::new(handle)
                        .style(self.style)
                        .width(self.width.unwrap_or(Length::Units(self.size)))
                        .height(self.height.unwrap_or(Length::Units(self.size)));

                    if let Some(content_fit) = self.content_fit {
                        widget = widget.content_fit(content_fit);
                    }

                    widget.into()
                }
                Handle::Image(handle) => {
                    let mut image = Image::new(handle)
                        .width(self.width.unwrap_or(Length::Units(self.size)))
                        .height(self.height.unwrap_or(Length::Units(self.size)));
                    if let Some(content_fit) = self.content_fit {
                        image = image.content_fit(content_fit);
                    }
                    image.into()
                }
            }
        })
        .into()
    }
}

impl<'a, Message: 'static> From<Icon<'a>> for Element<'a, Message> {
    fn from(icon: Icon<'a>) -> Self {
        icon.into_element::<Message>()
    }
}
