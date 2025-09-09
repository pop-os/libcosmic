// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Lazily-generated SVG icon widget for Iced.

mod named;
use std::ffi::OsStr;
use std::sync::Arc;

pub use named::{IconFallback, Named};

mod handle;
pub use handle::{Data, Handle, from_path, from_raster_bytes, from_raster_pixels, from_svg_bytes};

use crate::Element;
use derive_setters::Setters;
use iced::widget::{Image, Svg};
use iced::{ContentFit, Length, Rectangle};
use iced_core::Rotation;

/// Create an [`Icon`] from a pre-existing [`Handle`]
pub fn icon(handle: Handle) -> Icon {
    Icon {
        content_fit: ContentFit::Fill,
        handle,
        height: None,
        size: 16,
        class: crate::theme::Svg::default(),
        rotation: None,
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
    class: crate::theme::Svg,
    #[setters(skip)]
    pub(super) size: u16,
    content_fit: ContentFit,
    #[setters(strip_option)]
    width: Option<Length>,
    #[setters(strip_option)]
    height: Option<Length>,
    #[setters(strip_option)]
    rotation: Option<Rotation>,
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
    pub fn size(mut self, size: u16) -> Self {
        self.size = size;
        // ensures correct icon size variant selection
        if let Data::Name(named) = &self.handle.data {
            let mut new_named = named.clone();
            new_named.size = Some(size);
            self.handle = new_named.handle();
        }
        self
    }

    #[must_use]
    fn view<'a, Message: 'a>(self) -> Element<'a, Message> {
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
                .rotation(self.rotation.unwrap_or_default())
                .content_fit(self.content_fit)
                .into()
        };

        let from_svg = |handle| {
            Svg::<crate::Theme>::new(handle)
                .class(self.class.clone())
                .width(
                    self.width
                        .unwrap_or_else(|| Length::Fixed(f32::from(self.size))),
                )
                .height(
                    self.height
                        .unwrap_or_else(|| Length::Fixed(f32::from(self.size))),
                )
                .rotation(self.rotation.unwrap_or_default())
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

impl<'a, Message: 'a> From<Icon> for Element<'a, Message> {
    fn from(icon: Icon) -> Self {
        icon.view::<Message>()
    }
}

/// Draw an icon in the given bounds via the runtime's renderer.
pub fn draw(renderer: &mut crate::Renderer, handle: &Handle, icon_bounds: Rectangle) {
    enum IcedHandle {
        Svg(iced_core::svg::Handle),
        Image(iced_core::image::Handle),
    }

    let iced_handle = match handle.clone().data {
        Data::Name(named) => named.path().map(|path| {
            if path.extension().is_some_and(|ext| ext == OsStr::new("svg")) {
                IcedHandle::Svg(iced_core::svg::Handle::from_path(path))
            } else {
                IcedHandle::Image(iced_core::image::Handle::from_path(path))
            }
        }),

        Data::Image(handle) => Some(IcedHandle::Image(handle)),
        Data::Svg(handle) => Some(IcedHandle::Svg(handle)),
    };

    match iced_handle {
        Some(IcedHandle::Svg(handle)) => iced_core::svg::Renderer::draw_svg(
            renderer,
            iced_core::svg::Svg::new(handle),
            icon_bounds,
        ),

        Some(IcedHandle::Image(handle)) => {
            iced_core::image::Renderer::draw_image(
                renderer,
                handle,
                iced_core::image::FilterMethod::Linear,
                icon_bounds,
                iced_core::Radians::from(0),
                1.0,
                [0.0; 4],
            );
        }

        None => {}
    }
}
