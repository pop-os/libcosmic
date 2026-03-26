// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::Icon;
use crate::widget::{image, svg};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::hash::Hash;
use std::path::PathBuf;

#[must_use]
#[derive(Clone, Debug, Hash, derive_setters::Setters)]
pub struct Handle {
    pub symbolic: bool,
    #[setters(skip)]
    pub data: Data,
}

impl Handle {
    #[inline]
    pub fn icon(self) -> Icon {
        super::icon(self)
    }
}

#[must_use]
#[derive(Clone, Debug, Hash)]
pub enum Data {
    // Name(Named),
    Image(image::Handle),
    Svg(svg::Handle),
}

/// Create an icon handle from its path.
pub fn from_path(path: PathBuf) -> Handle {
    Handle {
        symbolic: path
            .file_stem()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.ends_with("-symbolic")),
        data: if path.extension().is_some_and(|ext| ext == OsStr::new("svg")) {
            Data::Svg(svg::Handle::from_path(path))
        } else {
            Data::Image(image::Handle::from_path(path))
        },
    }
}

/// Create an image handle from memory.
pub fn from_raster_bytes(
    bytes: impl Into<Cow<'static, [u8]>>
    + std::convert::AsRef<[u8]>
    + std::marker::Send
    + std::marker::Sync
    + 'static,
) -> Handle {
    fn inner(bytes: Cow<'static, [u8]>) -> Handle {
        Handle {
            symbolic: false,
            data: match bytes {
                Cow::Owned(b) => Data::Image(image::Handle::from_bytes(b)),
                Cow::Borrowed(b) => Data::Image(image::Handle::from_bytes(b)),
            },
        }
    }

    inner(bytes.into())
}

/// Create an image handle from RGBA data, where you must define the width and height.
pub fn from_raster_pixels(
    width: u32,
    height: u32,
    pixels: impl Into<Cow<'static, [u8]>>
    + std::convert::AsRef<[u8]>
    + std::marker::Send
    + std::marker::Sync,
) -> Handle {
    fn inner(width: u32, height: u32, pixels: Cow<'static, [u8]>) -> Handle {
        Handle {
            symbolic: false,
            data: match pixels {
                Cow::Owned(pixels) => Data::Image(image::Handle::from_rgba(width, height, pixels)),
                Cow::Borrowed(pixels) => {
                    Data::Image(image::Handle::from_rgba(width, height, pixels))
                }
            },
        }
    }

    inner(width, height, pixels.into())
}

/// Create a SVG handle from memory.
pub fn from_svg_bytes(bytes: impl Into<Cow<'static, [u8]>>) -> Handle {
    Handle {
        symbolic: true,
        data: Data::Svg(svg::Handle::from_memory(bytes)),
    }
}
