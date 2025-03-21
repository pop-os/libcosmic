// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Select the preferred icon theme.

use std::borrow::Cow;
use std::sync::Mutex;

pub const COSMIC: &str = "Cosmic";

pub(crate) static DEFAULT: Mutex<Cow<'static, str>> = Mutex::new(Cow::Borrowed(COSMIC));

/// The fallback icon theme to search if no icon theme was specified.
#[must_use]
#[allow(clippy::missing_panics_doc)]
#[inline]
pub fn default() -> String {
    DEFAULT.lock().unwrap().to_string()
}

/// Set the fallback icon theme to search when loading system icons.
#[allow(clippy::missing_panics_doc)]
#[cold]
pub fn set_default(name: impl Into<Cow<'static, str>>) {
    *DEFAULT.lock().unwrap() = name.into();
}
