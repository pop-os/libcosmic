// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::cell::RefCell;

thread_local! {
    /// The fallback icon theme to search if no icon theme was specified.
    pub(crate) static DEFAULT: RefCell<String> = RefCell::new(String::from("Pop"));
}

/// The fallback icon theme to search if no icon theme was specified.
#[must_use]
pub fn default() -> String {
    DEFAULT.with(|f| f.borrow().clone())
}

/// Set the fallback icon theme to search when loading system icons.
pub fn set_default(name: impl Into<String>) {
    DEFAULT.with(|f| *f.borrow_mut() = name.into());
}
