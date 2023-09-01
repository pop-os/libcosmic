// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Lazily-generated SVG icon widget for Iced.

mod builder;
pub use builder::Builder;

pub mod handle;
pub use handle::Handle;

use crate::{Element, Renderer};
use derive_setters::Setters;
use iced::widget::{Image, Svg};
use iced::{ContentFit, Length};
use std::borrow::Cow;
use std::path::PathBuf;

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

/// Create an [`Icon`] from its path.
pub fn from_path(path: PathBuf) -> Icon {
    icon(handle::from_path(path))
}

/// Create an image [`Icon`] from memory.
pub fn from_raster_bytes(
    bytes: impl Into<Cow<'static, [u8]>>
        + std::convert::AsRef<[u8]>
        + std::marker::Send
        + std::marker::Sync
        + 'static,
) -> Icon {
    icon(handle::from_raster_bytes(bytes))
}

/// Create an image [`Icon`] from RGBA data, where you must define the width and height.
pub fn from_raster_pixels(
    width: u32,
    height: u32,
    pixels: impl Into<Cow<'static, [u8]>>
        + std::convert::AsRef<[u8]>
        + std::marker::Send
        + std::marker::Sync
        + 'static,
) -> Icon {
    icon(handle::from_raster_pixels(width, height, pixels))
}

/// Create a SVG [`Icon`] from memory.
pub fn from_svg_bytes(bytes: impl Into<Cow<'static, [u8]>>) -> Icon {
    icon(handle::from_svg_bytes(bytes))
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
    fn into_element<Message: 'static>(self) -> Element<'static, Message> {
        match self.handle.variant {
            handle::Variant::Image(handle) => Image::new(handle)
                .width(self.width.unwrap_or(Length::Fixed(f32::from(self.size))))
                .height(self.height.unwrap_or(Length::Fixed(f32::from(self.size))))
                .content_fit(self.content_fit)
                .into(),
            handle::Variant::Svg(handle) => Svg::<Renderer>::new(handle)
                .style(self.style.clone())
                .width(self.width.unwrap_or(Length::Fixed(f32::from(self.size))))
                .height(self.height.unwrap_or(Length::Fixed(f32::from(self.size))))
                .content_fit(self.content_fit)
                .symbolic(self.handle.symbolic)
                .into(),
        }
    }
}

impl<Message: 'static> From<Icon> for Element<'static, Message> {
    fn from(icon: Icon) -> Self {
        icon.into_element::<Message>()
    }
}
