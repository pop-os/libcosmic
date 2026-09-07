// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use super::Icon;
use crate::widget::{image, svg};
use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs::File;
use std::hash::Hash;
use std::io::Read;
use std::path::{Path, PathBuf};

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

enum SvgSource {
    Path,
    Bytes(Vec<u8>),
}

fn svg_source(path: &Path) -> Option<SvgSource> {
    if path
        .extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("svg"))
    {
        return Some(SvgSource::Path);
    }

    let Ok(mut file) = File::open(path) else {
        return None;
    };

    let Ok(metadata) = file.metadata() else {
        return None;
    };
    const MAX_SVG_SIZE: u64 = 16 * 1024 * 1024;
    if !metadata.file_type().is_file() || metadata.len() > MAX_SVG_SIZE {
        return None;
    }

    let mut prefix = [0; 32];
    let Ok(length) = file.read(&mut prefix) else {
        return None;
    };
    let prefix = &prefix[..length];

    if ::image::guess_format(prefix).is_ok() {
        return None;
    }

    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    bytes.extend_from_slice(prefix);
    if file.read_to_end(&mut bytes).is_err() {
        return None;
    }

    let document = roxmltree::Document::parse(std::str::from_utf8(&bytes).ok()?).ok()?;
    (document.root_element().tag_name().name() == "svg").then_some(SvgSource::Bytes(bytes))
}

/// Create an icon handle from its path.
pub fn from_path(path: PathBuf) -> Handle {
    Handle {
        symbolic: path
            .file_stem()
            .and_then(OsStr::to_str)
            .is_some_and(|name| name.ends_with("-symbolic")),
        data: match svg_source(&path) {
            Some(SvgSource::Path) => Data::Svg(svg::Handle::from_path(path)),
            Some(SvgSource::Bytes(bytes)) => Data::Svg(svg::Handle::from_memory(bytes)),
            None => Data::Image(image::Handle::from_path(path)),
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
        symbolic: false,
        data: Data::Svg(svg::Handle::from_memory(bytes)),
    }
}
