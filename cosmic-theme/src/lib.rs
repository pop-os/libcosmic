#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

//! Cosmic theme library.
//!
//! Provides utilities for creating custom cosmic themes.
//!

pub use model::*;
pub use output::*;

mod model;
mod output;

/// composite colors in srgb
pub mod composite;
/// get color steps
pub mod steps;
/// utilities
pub mod util;

/// name of cosmic theme
pub const NAME: &'static str = "com.system76.CosmicTheme";
/// Name of the theme directory
pub const THEME_DIR: &str = "themes";
/// name of the palette directory
pub const PALETTE_DIR: &str = "palettes";

pub use palette;
