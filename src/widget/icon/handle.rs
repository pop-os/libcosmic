// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::{Builder, Icon};
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
    pub variant: Variant,
}

impl Handle {
    pub fn icon(self) -> Icon {
        super::icon(self)
    }
}

#[must_use]
#[derive(Clone, Debug, Hash)]
pub enum Variant {
    Image(image::Handle),
    Svg(svg::Handle),
}

/// Create an icon handle from its XDG icon name.
pub fn from_name(name: &str) -> Builder {
    Builder::new(name)
}

/// Create an icon handle from its path.
pub fn from_path(path: PathBuf) -> Handle {
    Handle {
        symbolic: path
            .file_stem()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.ends_with("-symbolic")),
        variant: if path.extension().is_some_and(|ext| ext == OsStr::new("svg")) {
            Variant::Svg(svg::Handle::from_path(path))
        } else {
            Variant::Image(image::Handle::from_path(path))
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
    Handle {
        symbolic: false,
        variant: Variant::Image(image::Handle::from_memory(bytes)),
    }
}

/// Create an image handle from RGBA data, where you must define the width and height.
pub fn from_raster_pixels(
    width: u32,
    height: u32,
    pixels: impl Into<Cow<'static, [u8]>>
        + std::convert::AsRef<[u8]>
        + std::marker::Send
        + std::marker::Sync
        + 'static,
) -> Handle {
    Handle {
        symbolic: false,
        variant: Variant::Image(image::Handle::from_pixels(width, height, pixels)),
    }
}

/// Create a SVG handle from memory.
pub fn from_svg_bytes(bytes: impl Into<Cow<'static, [u8]>>) -> Handle {
    Handle {
        symbolic: false,
        variant: Variant::Svg(svg::Handle::from_memory(bytes)),
    }
}
