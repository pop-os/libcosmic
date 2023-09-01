// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Select the preferred icon theme.

use std::borrow::Cow;
use std::cell::RefCell;

thread_local! {
    /// The fallback icon theme to search if no icon theme was specified.
    pub(crate) static DEFAULT: RefCell<Cow<'static, str>> = RefCell::new("Cosmic".into());
}

/// The fallback icon theme to search if no icon theme was specified.
#[must_use]
pub fn default() -> String {
    DEFAULT.with(|theme| theme.borrow().to_string())
}

/// Set the fallback icon theme to search when loading system icons.
pub fn set_default(name: impl Into<Cow<'static, str>>) {
    DEFAULT.with(|theme| *theme.borrow_mut() = name.into());
}
