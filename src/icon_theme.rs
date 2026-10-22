// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Select the preferred icon theme.

use std::borrow::Cow;
use std::sync::Mutex;

pub const COSMIC: &str = crate::cosmic_theme::DEFAULT_ICON_THEME;

static OVERRIDE: Mutex<Option<Cow<'static, str>>> = Mutex::new(None);

/// This is the icon theme configured by the active theme unless the
/// application has set an override with [`set_default`].
#[must_use]
#[allow(clippy::missing_panics_doc)]
#[inline]
pub fn default() -> String {
    let override_theme = OVERRIDE.lock().unwrap();
    if let Some(theme) = override_theme.as_ref() {
        return theme.to_string();
    }
    drop(override_theme);

    let theme = crate::theme::active().cosmic().icon_theme.clone();
    if theme.is_empty() {
        COSMIC.to_string()
    } else {
        theme
    }
}

/// Override the icon theme selected by the active theme.
///
/// Passing an empty string clears the override, returning control to the icon
/// theme configured by the active theme.
#[allow(clippy::missing_panics_doc)]
#[cold]
pub fn set_default(name: impl Into<Cow<'static, str>>) {
    let name = name.into();
    *OVERRIDE.lock().unwrap() = if name.is_empty() { None } else { Some(name) };
}

/// Clear the application icon theme override.
#[allow(clippy::missing_panics_doc)]
#[cold]
pub fn reset_default() {
    *OVERRIDE.lock().unwrap() = None;
}
