#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

//! Cosmic theme library.
//!
//! Provides utilities for creating custom cosmic themes.
//!

pub use model::*;

mod model;

#[cfg(feature = "export")]
mod output;

/// composite colors in srgb
pub mod composite;
/// get color steps
pub mod steps;

/// name of cosmic theme
pub const NAME: &str = "com.system76.CosmicTheme";

pub use palette;
