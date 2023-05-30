#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

//! Cosmic theme library.
//!
//! Provides utilities for creating custom cosmic themes.
//!

#[cfg(feature = "contrast-derivation")]
pub use color_picker::*;
pub use config::*;
#[cfg(feature = "hex-color")]
pub use hex_color::*;
pub use model::*;
pub use output::*;
pub use theme_provider::*;
#[cfg(feature = "contrast-derivation")]
mod color_picker;
mod config;
#[cfg(feature = "hex-color")]
mod hex_color;
mod model;
mod output;
mod theme_provider;
/// utilities
pub mod util;

/// name of cosmic theme
pub const NAME: &'static str = "com.system76.CosmicTheme";
/// Name of the theme directory
pub const THEME_DIR: &str = "themes";
/// name of the palette directory
pub const PALETTE_DIR: &str = "palettes";

pub use palette;

/// theme derivation from an image
#[cfg(feature = "theme-from-image")]
pub mod theme_from_image {
    use image::EncodableLayout;
    use kmeans_colors::{get_kmeans_hamerly, Kmeans, Sort};
    use palette::{rgb::Srgba, Pixel};
    use palette::{IntoColor, Lab};
    use std::path::Path;

    /// Create a palette from an image
    /// The palette is sorted by how often a color occurs in the image, most often first
    pub fn theme_from_image<P: AsRef<Path>>(path: P) -> Option<Vec<Srgba>> {
        // calculate kmeans colors from file
        // let pixbuf = Pixbuf::from_file(path);
        let img = image::open(path);
        match img {
            Ok(img) => {
                let lab: Vec<Lab> = Srgba::from_raw_slice(img.to_rgba8().into_raw().as_bytes())
                    .iter()
                    .map(|x| x.color.into_format().into_color())
                    .collect();

                let mut result = Kmeans::new();

                // TODO random seed
                for i in 0..2 {
                    let run_result = get_kmeans_hamerly(5, 20, 5.0, false, &lab, i as u64);
                    if run_result.score < result.score {
                        result = run_result;
                    }
                }
                let mut res = Lab::sort_indexed_colors(&result.centroids, &result.indices);
                res.sort_unstable_by(|a, b| (b.percentage).partial_cmp(&a.percentage).unwrap());
                let colors: Vec<Srgba> = res.iter().map(|x| x.centroid.into_color()).collect();
                Some(colors)
            }
            Err(err) => {
                eprintln!("{}", err);
                None
            }
        }
    }
}
