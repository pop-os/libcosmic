// Copyright 2019 H�ctor Ram�n, Iced contributors
// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: MIT

//! A text input widget from iced widgets plus some added details.

pub mod cursor;
pub mod editor;
mod input;
mod style;
pub mod value;

pub use crate::theme::TextInput as Style;
pub use input::*;
pub use style::{Appearance, StyleSheet};
