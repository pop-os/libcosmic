// Copyright 2025 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Embedded icons for platforms which do not support icon themes yet.

/// Icon bundling is not enabled on unix platforms.
#[cfg(unix)]
pub fn get(icon_name: &str) -> Option<super::Data> {
    None
}

#[cfg(not(unix))]
/// Get a bundled icon on non-unix platforms.
pub fn get(icon_name: &str) -> Option<super::Data> {
    ICONS
        .get(icon_name)
        .map(|bytes| super::Data::Svg(crate::iced::widget::svg::Handle::from_memory(*bytes)))
}

#[cfg(not(unix))]
include!(concat!(env!("OUT_DIR"), "/bundled_icons.rs"));
