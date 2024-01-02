// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Lazily-generated SVG icon widget for Iced.

mod named;
use std::ffi::OsStr;
use std::sync::Arc;

pub use named::{IconFallback, Named};

mod handle;
pub use handle::{from_path, from_raster_bytes, from_raster_pixels, from_svg_bytes, Data, Handle};

use crate::{Element, Renderer};
use derive_setters::Setters;
use iced::widget::{Image, Svg};
use iced::{ContentFit, Length};

/// Create an [`Icon`] from a pre-existing [`Handle`]
pub fn icon(handle: Handle) -> Icon {
    Icon {
        content_fit: ContentFit::Fill,
        handle,
        height: None,
        size: 16,
        style: crate::theme::Svg::default(),
        width: None,
    }
}

/// Create an icon handle from its XDG icon name.
pub fn from_name(name: impl Into<Arc<str>>) -> Named {
    Named::new(name)
}

/// An image which may be an SVG or PNG.
#[must_use]
#[derive(Clone, Setters)]
pub struct Icon {
    #[setters(skip)]
    handle: Handle,
    style: crate::theme::Svg,
    pub(super) size: u16,
    content_fit: ContentFit,
    #[setters(strip_option)]
    width: Option<Length>,
    #[setters(strip_option)]
    height: Option<Length>,
}

impl Icon {
    #[must_use]
    pub fn into_svg_handle(self) -> Option<crate::widget::svg::Handle> {
        match self.handle.data {
            Data::Name(named) => {
                if let Some(path) = named.path() {
                    if path.extension().is_some_and(|ext| ext == OsStr::new("svg")) {
                        return Some(iced_core::svg::Handle::from_path(path));
                    }
                }
            }

            Data::Image(_) => (),
            Data::Svg(handle) => return Some(handle),
        }

        None
    }

    #[must_use]
    fn into_element<Message: 'static>(self) -> Element<'static, Message> {
        let from_image = |handle| {
            Image::new(handle)
                .width(
                    self.width
                        .unwrap_or_else(|| Length::Fixed(f32::from(self.size))),
                )
                .height(
                    self.height
                        .unwrap_or_else(|| Length::Fixed(f32::from(self.size))),
                )
                .content_fit(self.content_fit)
                .into()
        };

        let from_svg = |handle| {
            Svg::<Renderer>::new(handle)
                .style(self.style.clone())
                .width(
                    self.width
                        .unwrap_or_else(|| Length::Fixed(f32::from(self.size))),
                )
                .height(
                    self.height
                        .unwrap_or_else(|| Length::Fixed(f32::from(self.size))),
                )
                .content_fit(self.content_fit)
                .symbolic(self.handle.symbolic)
                .into()
        };

        match self.handle.data {
            Data::Name(named) => {
                if let Some(path) = named.path() {
                    if path.extension().is_some_and(|ext| ext == OsStr::new("svg")) {
                        from_svg(iced_core::svg::Handle::from_path(path))
                    } else {
                        from_image(iced_core::image::Handle::from_path(path))
                    }
                } else {
                    let bytes: &'static [u8] = &[];
                    from_svg(iced_core::svg::Handle::from_memory(bytes))
                }
            }

            Data::Image(handle) => from_image(handle),
            Data::Svg(handle) => from_svg(handle),
        }
    }
}

impl<Message: 'static> From<Icon> for Element<'static, Message> {
    fn from(icon: Icon) -> Self {
        icon.into_element::<Message>()
    }
}
