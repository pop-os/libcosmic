//! A text input widget from iced widgets plus some added details.

pub mod cursor;
pub mod editor;
mod input;
mod style;
pub mod value;

pub use input::*;
pub use style::{Appearance as TextInputAppearance, StyleSheet as TextInputStyleSheet};
