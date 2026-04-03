// Copyright 2022 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Lazily-generated SVG icon widget for Iced.

mod bundle;
mod named;
use std::sync::Arc;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use fast_image_resize::images::Image as FrImage;
use fast_image_resize::{
    create_srgb_mapper, FilterType, MulDiv, PixelType, ResizeAlg, ResizeOptions, Resizer,
};

pub use named::{IconFallback, Named};

mod handle;
pub use handle::{from_path, from_raster_bytes, from_raster_pixels, from_svg_bytes, Data, Handle};

use crate::Element;
use derive_setters::Setters;
use iced::widget::{image as image_widget, Svg};
use iced::{ContentFit, Length, Radians, Rectangle, Size};
use iced_core::widget::Tree;
use iced_core::{layout, mouse, renderer, Layout, Rotation, Widget};

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
            Data::Image(_) => (),
            Data::Svg(handle) => return Some(handle),
        }

        None
    }

    #[must_use]
    pub fn size(mut self, size: u16) -> Self {
        self.size = size;
        self
    }

    #[must_use]
    fn view<'a, Message: 'a>(self) -> Element<'a, Message> {
        let from_image = |handle| {
            RasterIcon {
                handle,
                width: self
                    .width
                    .unwrap_or_else(|| Length::Fixed(f32::from(self.size))),
                height: self
                    .height
                    .unwrap_or_else(|| Length::Fixed(f32::from(self.size))),
                content_fit: self.content_fit,
                rotation: self.rotation.unwrap_or_default(),
            }
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
    draw_with_scale(renderer, handle, icon_bounds, 1.0);
}

/// Draw an icon in the given bounds via the runtime's renderer.
pub fn draw_with_scale(
    renderer: &mut crate::Renderer,
    handle: &Handle,
    icon_bounds: Rectangle,
    scale_factor: f32,
) {
    match handle.clone().data {
        Data::Svg(handle) => {
            iced_core::svg::Renderer::draw_svg(
                renderer,
                iced_core::svg::Svg::new(handle),
                icon_bounds,
                icon_bounds,
            );
        }

        Data::Image(handle) => {
            let physical_width = (icon_bounds.width * scale_factor).ceil().max(1.0) as u32;
            let physical_height = (icon_bounds.height * scale_factor).ceil().max(1.0) as u32;
            let handle =
                resized_raster_handle(&handle, physical_width, physical_height).unwrap_or(handle);

            iced_core::image::Renderer::draw_image(
                renderer,
                iced_core::Image {
                    handle,
                    filter_method: iced_core::image::FilterMethod::Linear,
                    rotation: Radians(0.),
                    border_radius: [0.0; 4].into(),
                    opacity: 1.0,
                    snap: true,
                },
                icon_bounds,
                icon_bounds,
            );
        }
    }
}

#[derive(Clone)]
struct RasterIcon {
    handle: image_widget::Handle,
    width: Length,
    height: Length,
    content_fit: ContentFit,
    rotation: Rotation,
}

impl<Message> Widget<Message, crate::Theme, crate::Renderer> for RasterIcon {
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        renderer: &crate::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        image_widget::layout(
            renderer,
            limits,
            &self.handle,
            self.width,
            self.height,
            None,
            self.content_fit,
            self.rotation,
            false,
            [0.0; 4],
        )
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut crate::Renderer,
        _theme: &crate::Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let scale_factor = style.scale_factor as f32;
        let physical_width = (bounds.width * scale_factor).ceil().max(1.0) as u32;
        let physical_height = (bounds.height * scale_factor).ceil().max(1.0) as u32;
        let handle = resized_raster_handle(&self.handle, physical_width, physical_height)
            .unwrap_or_else(|| self.handle.clone());

        image_widget::draw(
            renderer,
            layout,
            &handle,
            None,
            [0.0; 4].into(),
            self.content_fit,
            image_widget::FilterMethod::Linear,
            self.rotation,
            1.0,
            1.0,
        );
    }
}

impl<Message> From<RasterIcon> for Element<'_, Message> {
    fn from(icon: RasterIcon) -> Self {
        Element::new(icon)
    }
}

#[cfg(test)]
mod tests {
    use super::resized_raster_handle;

    #[test]
    fn lanczos_resize_produces_requested_dimensions() {
        let handle = iced::advanced::image::Handle::from_rgba(
            2,
            2,
            vec![
                255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
            ],
        );

        let resized = resized_raster_handle(&handle, 6, 5).expect("resized handle");

        match resized {
            iced::advanced::image::Handle::Rgba { width, height, .. } => {
                assert_eq!((width, height), (6, 5));
            }
            other => panic!("unexpected handle: {other:?}"),
        }
    }

    #[test]
    fn premultiplied_resize_avoids_transparent_color_bleed() {
        let handle = iced::advanced::image::Handle::from_rgba(
            2,
            1,
            vec![
                255, 0, 0, 255, //
                0, 0, 255, 0,
            ],
        );

        let resized = resized_raster_handle(&handle, 1, 1).expect("resized handle");
        let pixels = crate::iced::advanced::graphics::image::load(&resized)
            .expect("resized image")
            .into_raw();

        assert_eq!(pixels.len(), 4);
        assert!(
            pixels[0] > pixels[2],
            "expected red channel to dominate: {pixels:?}"
        );
        assert!(pixels[3] > 0, "expected non-zero alpha: {pixels:?}");
    }
}

fn resized_raster_handle(
    handle: &iced::advanced::image::Handle,
    width: u32,
    height: u32,
) -> Option<iced::advanced::image::Handle> {
    type CacheKey = (iced::advanced::image::Id, u32, u32);

    static CACHE: OnceLock<Mutex<HashMap<CacheKey, iced::advanced::image::Handle>>> =
        OnceLock::new();

    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = (handle.id(), width, height);

    if let Some(handle) = cache.lock().ok().and_then(|cache| cache.get(&key).cloned()) {
        return Some(handle);
    }

    let buffer = crate::iced::advanced::graphics::image::load(handle).ok()?;
    let (src_width, src_height) = (buffer.width(), buffer.height());
    let raw = buffer.into_raw().to_vec();
    let mut source = FrImage::from_vec_u8(src_width, src_height, raw, PixelType::U8x4).ok()?;

    let mapper = create_srgb_mapper();
    mapper.forward_map_inplace(&mut source).ok()?;

    let mul_div = MulDiv::new();
    mul_div.multiply_alpha_inplace(&mut source).ok()?;

    let upscale = 4;
    let hi_width = width.checked_mul(upscale)?;
    let hi_height = height.checked_mul(upscale)?;
    let mut hi = FrImage::new(hi_width, hi_height, PixelType::U8x4);
    let mut resizer = Resizer::new();
    let upsample =
        ResizeOptions::new().resize_alg(ResizeAlg::SuperSampling(FilterType::Lanczos3, 4));
    let downsample = ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::Lanczos3));

    resizer.resize(&source, &mut hi, Some(&upsample)).ok()?;

    mul_div.divide_alpha_inplace(&mut hi).ok()?;
    mapper.backward_map_inplace(&mut hi).ok()?;

    let mut destination = FrImage::new(width, height, PixelType::U8x4);
    resizer
        .resize(&hi, &mut destination, Some(&downsample))
        .ok()?;

    let resized_pixels = destination.into_vec();

    let resized = iced::advanced::image::Handle::from_rgba(width, height, resized_pixels);

    if let Ok(mut cache) = cache.lock() {
        let _ = cache.insert(key, resized.clone());
    }

    Some(resized)
}
