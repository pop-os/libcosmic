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

#[derive(Clone, Debug, Hash)]
pub enum Handle {
    Image(image::Handle),
    Svg(svg::Handle),
}

#[derive(Clone, Debug, Hash)]
pub enum IconSource<'a> {
    Path(Cow<'a, Path>),
    Name(Cow<'a, str>),
    Handle(Handle),
}

impl<'a> IconSource<'a> {
    /// Loads the icon as either an image or svg [`Handle`].
    #[must_use]
    pub fn load(&self, size: u16, theme: Option<&str>, svg: bool) -> Handle {
        let name_path_buffer: Option<PathBuf>;
        let icon: Option<&Path> = match self {
            IconSource::Handle(handle) => return handle.clone(),
            IconSource::Path(ref path) => Some(path),
            #[cfg(unix)]
            IconSource::Name(ref name) => {
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
            // TODO: Icon loading mechanism for non-Unix systems
            #[cfg(not(unix))]
            IconSource::Name(_) => None,
        };

        let is_svg = svg
            || icon
                .as_ref()
                .map_or(true, |path| path.extension() == Some(OsStr::new("svg")));

        if is_svg {
            let handle = if let Some(path) = icon {
                svg::Handle::from_path(path)
            } else {
                eprintln!("svg icon '{self:?}' size {size} not found");
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

    /// Get a handle to a raster image from a path.
    pub fn raster_from_path(path: impl Into<PathBuf>) -> Self {
        IconSource::Handle(Handle::Image(image::Handle::from_path(path)))
    }

    /// Get a handle to a raster image from memory.
    pub fn raster_from_memory(
        bytes: impl Into<Cow<'static, [u8]>>
            + std::convert::AsRef<[u8]>
            + std::marker::Send
            + std::marker::Sync
            + 'static,
    ) -> Self {
        IconSource::Handle(Handle::Image(image::Handle::from_memory(bytes)))
    }

    /// Get a handle to a raster image from RGBA data, where you must define the width and height.
    pub fn raster_from_pixels(
        width: u32,
        height: u32,
        pixels: impl Into<Cow<'static, [u8]>>
            + std::convert::AsRef<[u8]>
            + std::marker::Send
            + std::marker::Sync
            + 'static,
    ) -> Self {
        IconSource::Handle(Handle::Image(image::Handle::from_pixels(
            width, height, pixels,
        )))
    }

    /// Get a handle to a SVG from a path.
    pub fn svg_from_path(path: impl Into<PathBuf>) -> Self {
        IconSource::Handle(Handle::Svg(svg::Handle::from_path(path)))
    }

    /// Get a handle to a SVG from memory.
    pub fn svg_from_memory(bytes: impl Into<Cow<'static, [u8]>>) -> Self {
        IconSource::Handle(Handle::Svg(svg::Handle::from_memory(bytes)))
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

impl From<image::Handle> for IconSource<'static> {
    fn from(handle: image::Handle) -> Self {
        Self::Handle(Handle::Image(handle))
    }
}

impl From<svg::Handle> for IconSource<'static> {
    fn from(handle: svg::Handle) -> Self {
        Self::Handle(Handle::Svg(handle))
    }
}

/// A lazily-generated icon.
#[derive(Setters)]
pub struct Icon<'a> {
    #[setters(skip)]
    source: IconSource<'a>,
    #[setters(strip_option, into)]
    theme: Option<Cow<'a, str>>,
    style: crate::theme::Svg,
    size: u16,
    content_fit: ContentFit,
    #[setters(strip_option)]
    width: Option<Length>,
    #[setters(strip_option)]
    height: Option<Length>,
    force_svg: bool,
}

// TODO what to do here
impl Hash for Icon<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.theme.hash(state);
        self.style.hash(state);
        self.size.hash(state);
        self.content_fit.hash(state);
        self.force_svg.hash(state);
    }
}

/// A lazily-generated icon.
#[must_use]
pub fn icon<'a>(source: impl Into<IconSource<'a>>, size: u16) -> Icon<'a> {
    Icon {
        content_fit: ContentFit::Fill,
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
    fn raster_element<Message: 'static>(&self, handle: image::Handle) -> Element<'static, Message> {
        Image::new(handle)
            .width(self.width.unwrap_or(Length::Fixed(self.size as f32)))
            .height(self.height.unwrap_or(Length::Fixed(self.size as f32)))
            .content_fit(self.content_fit)
            .into()
    }

    fn svg_element<Message: 'static>(&self, handle: svg::Handle) -> Element<'static, Message> {
        svg::Svg::<Renderer>::new(handle)
            .style(self.style.clone())
            .width(self.width.unwrap_or(Length::Fixed(self.size as f32)))
            .height(self.height.unwrap_or(Length::Fixed(self.size as f32)))
            .content_fit(self.content_fit)
            .into()
    }

    #[must_use]
    fn into_element<Message: 'static>(mut self) -> Element<'a, Message> {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);

        if self.theme.is_none() {
            crate::settings::DEFAULT_ICON_THEME.with(|f| f.borrow().hash(&mut hasher));
        }

        let hash = hasher.finish();

        let mut source = IconSource::Name(Cow::Borrowed(""));
        std::mem::swap(&mut source, &mut self.source);

        iced::widget::lazy(hash, move |_| -> Element<Message> {
            match source.load(self.size, self.theme.as_deref(), self.force_svg) {
                Handle::Svg(handle) => self.svg_element(handle),
                Handle::Image(handle) => self.raster_element(handle),
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
