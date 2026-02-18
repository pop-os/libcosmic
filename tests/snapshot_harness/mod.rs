// Copyright 2024 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

//! Snapshot testing harness for COSMIC widgets.
//!
//! Renders widgets to pixel buffers using the tiny-skia software renderer
//! and compares against stored reference images. Provides deterministic,
//! headless visual regression testing without requiring a display server.

use cosmic::iced_core::{self, Rectangle, Size, layout, mouse, renderer};
use cosmic::iced_core::widget::Tree;

use std::path::{Path, PathBuf};

/// Maximum per-pixel color channel difference allowed before a mismatch
/// is flagged. Accounts for minor anti-aliasing and rounding differences.
const PIXEL_TOLERANCE: u8 = 2;

/// Maximum fraction of pixels allowed to differ before the snapshot is
/// considered a failure (0.01 = 1%).
const MISMATCH_THRESHOLD: f64 = 0.01;

/// Test harness for rendering COSMIC widgets to pixel buffers.
pub struct SnapshotHarness {
    width: u32,
    height: u32,
}

impl SnapshotHarness {
    /// Create a new harness with the given pixel dimensions.
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    /// Render an element and compare it against a stored reference snapshot.
    ///
    /// If no reference exists, the rendered image is saved as the new reference.
    /// If a reference exists, the rendered image is compared pixel-by-pixel.
    pub fn snapshot<'a>(
        &self,
        name: &str,
        element: cosmic::Element<'a, ()>,
    ) {
        let pixels = self.render_element(element);
        let snapshot_dir = snapshot_dir();

        std::fs::create_dir_all(&snapshot_dir)
            .expect("Failed to create snapshots directory");

        let reference_path = snapshot_dir.join(format!("{name}.png"));
        let actual_path = snapshot_dir.join(format!("{name}.actual.png"));
        let diff_path = snapshot_dir.join(format!("{name}.diff.png"));

        // Save the current render
        save_rgba_png(&actual_path, &pixels, self.width, self.height);

        if reference_path.exists() {
            // Compare against reference
            let reference = load_png_rgba(&reference_path);
            let result = compare_images(
                &reference.data,
                reference.width,
                reference.height,
                &pixels,
                self.width,
                self.height,
            );

            match result {
                CompareResult::Match => {
                    // Clean up actual file on success
                    let _ = std::fs::remove_file(&actual_path);
                    let _ = std::fs::remove_file(&diff_path);
                }
                CompareResult::SizeMismatch {
                    ref_w,
                    ref_h,
                    act_w,
                    act_h,
                } => {
                    panic!(
                        "Snapshot '{name}' size mismatch: reference is {ref_w}x{ref_h}, \
                         actual is {act_w}x{act_h}. \
                         Actual saved to: {}\n\
                         To update, delete the reference and re-run.",
                        actual_path.display()
                    );
                }
                CompareResult::PixelMismatch {
                    mismatch_count,
                    total_pixels,
                    diff_image,
                } => {
                    let pct = (mismatch_count as f64 / total_pixels as f64) * 100.0;
                    save_rgba_png(&diff_path, &diff_image, self.width, self.height);
                    panic!(
                        "Snapshot '{name}' has {mismatch_count}/{total_pixels} pixels \
                         different ({pct:.2}%). Threshold: {:.2}%\n\
                         Actual: {}\n\
                         Diff:   {}\n\
                         To update, delete the reference and re-run.",
                        MISMATCH_THRESHOLD * 100.0,
                        actual_path.display(),
                        diff_path.display(),
                    );
                }
            }
        } else {
            // No reference exists - promote actual to reference
            std::fs::rename(&actual_path, &reference_path).expect("Failed to save reference");
            eprintln!(
                "Created new snapshot reference: {}",
                reference_path.display()
            );
        }
    }

    /// Render a COSMIC element to an RGBA pixel buffer.
    fn render_element<'a>(&self, mut element: cosmic::Element<'a, ()>) -> Vec<u8> {
        let w = self.width as f32;
        let h = self.height as f32;

        // Build the widget tree
        let mut tree = Tree::new(element.as_widget());

        // Create a tiny-skia renderer
        let mut renderer = iced_tiny_skia::Renderer::new(
            iced_core::Font::DEFAULT,
            iced_core::Pixels(14.0),
        );

        // Compute layout
        let limits = layout::Limits::new(Size::ZERO, Size::new(w, h));
        let node = element.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout_obj = layout::Layout::new(&node);

        // Get theme
        let cosmic_theme = cosmic::theme::active();

        // Draw widget
        let viewport = Rectangle {
            x: 0.0,
            y: 0.0,
            width: w,
            height: h,
        };

        let style = renderer::Style {
            text_color: iced_core::Color::WHITE,
            icon_color: iced_core::Color::WHITE,
            scale_factor: 1.0,
        };

        element.as_widget().draw(
            &tree,
            &mut renderer,
            &cosmic_theme,
            &style,
            layout_obj,
            mouse::Cursor::Unavailable,
            &viewport,
        );

        // Render to pixel buffer
        let phys_w = self.width;
        let phys_h = self.height;
        let mut pixmap = tiny_skia::Pixmap::new(phys_w, phys_h)
            .expect("Failed to create pixmap");

        let viewport_obj = iced_tiny_skia::graphics::Viewport::with_logical_size(
            Size::new(w, h),
            1.0,
        );

        let mut mask = tiny_skia::Mask::new(phys_w, phys_h)
            .expect("Failed to create mask");

        let damage = vec![Rectangle {
            x: 0.0,
            y: 0.0,
            width: w,
            height: h,
        }];

        let bg_color = iced_core::Color::from_rgb(0.15, 0.15, 0.15);
        let overlay: &[String] = &[];

        renderer.draw(
            &mut pixmap.as_mut(),
            &mut mask,
            &viewport_obj,
            &damage,
            bg_color,
            overlay,
        );

        pixmap.data().to_vec()
    }
}

/// Directory where snapshot reference images are stored.
fn snapshot_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests").join("snapshots")
}

/// Save RGBA pixel data as a PNG file.
fn save_rgba_png(path: &Path, data: &[u8], width: u32, height: u32) {
    let img = image::RgbaImage::from_raw(width, height, data.to_vec())
        .expect("Invalid image dimensions");
    img.save(path)
        .unwrap_or_else(|e| panic!("Failed to save PNG to {}: {e}", path.display()));
}

/// Loaded PNG image data.
struct PngImage {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

/// Load a PNG file as RGBA pixel data.
fn load_png_rgba(path: &Path) -> PngImage {
    let img = image::open(path)
        .unwrap_or_else(|e| panic!("Failed to load PNG from {}: {e}", path.display()))
        .to_rgba8();
    let width = img.width();
    let height = img.height();
    PngImage {
        data: img.into_raw(),
        width,
        height,
    }
}

/// Result of comparing two images.
enum CompareResult {
    Match,
    SizeMismatch {
        ref_w: u32,
        ref_h: u32,
        act_w: u32,
        act_h: u32,
    },
    PixelMismatch {
        mismatch_count: usize,
        total_pixels: usize,
        diff_image: Vec<u8>,
    },
}

/// Compare two RGBA images pixel-by-pixel.
fn compare_images(
    reference: &[u8],
    ref_w: u32,
    ref_h: u32,
    actual: &[u8],
    act_w: u32,
    act_h: u32,
) -> CompareResult {
    if ref_w != act_w || ref_h != act_h {
        return CompareResult::SizeMismatch {
            ref_w,
            ref_h,
            act_w,
            act_h,
        };
    }

    let total_pixels = (ref_w * ref_h) as usize;
    let mut mismatch_count = 0;
    let mut diff_image = vec![0u8; reference.len()];

    for i in 0..total_pixels {
        let offset = i * 4;
        let r_diff = (reference[offset] as i16 - actual[offset] as i16).unsigned_abs() as u8;
        let g_diff =
            (reference[offset + 1] as i16 - actual[offset + 1] as i16).unsigned_abs() as u8;
        let b_diff =
            (reference[offset + 2] as i16 - actual[offset + 2] as i16).unsigned_abs() as u8;
        let a_diff =
            (reference[offset + 3] as i16 - actual[offset + 3] as i16).unsigned_abs() as u8;

        if r_diff > PIXEL_TOLERANCE
            || g_diff > PIXEL_TOLERANCE
            || b_diff > PIXEL_TOLERANCE
            || a_diff > PIXEL_TOLERANCE
        {
            mismatch_count += 1;
            // Highlight differences in red
            diff_image[offset] = 255;
            diff_image[offset + 1] = 0;
            diff_image[offset + 2] = 0;
            diff_image[offset + 3] = 255;
        } else {
            // Dim matching pixels
            diff_image[offset] = actual[offset] / 3;
            diff_image[offset + 1] = actual[offset + 1] / 3;
            diff_image[offset + 2] = actual[offset + 2] / 3;
            diff_image[offset + 3] = 255;
        }
    }

    let mismatch_fraction = mismatch_count as f64 / total_pixels as f64;
    if mismatch_fraction <= MISMATCH_THRESHOLD {
        CompareResult::Match
    } else {
        CompareResult::PixelMismatch {
            mismatch_count,
            total_pixels,
            diff_image,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_identical_images() {
        let data = vec![128u8; 4 * 10 * 10];
        let result = compare_images(&data, 10, 10, &data, 10, 10);
        assert!(matches!(result, CompareResult::Match));
    }

    #[test]
    fn compare_different_sizes() {
        let small = vec![0u8; 4 * 5 * 5];
        let big = vec![0u8; 4 * 10 * 10];
        let result = compare_images(&small, 5, 5, &big, 10, 10);
        assert!(matches!(result, CompareResult::SizeMismatch { .. }));
    }

    #[test]
    fn compare_within_tolerance() {
        let a = vec![100u8; 4 * 4 * 4];
        let mut b = a.clone();
        // Change one channel by 1 (within tolerance of 2)
        b[0] = 101;
        let result = compare_images(&a, 4, 4, &b, 4, 4);
        assert!(matches!(result, CompareResult::Match));
    }

    #[test]
    fn compare_exceeds_tolerance() {
        let a = vec![100u8; 4 * 2 * 2]; // 4 pixels
        let mut b = a.clone();
        // Make all pixels differ significantly
        for i in (0..b.len()).step_by(4) {
            b[i] = 200;
        }
        let result = compare_images(&a, 2, 2, &b, 2, 2);
        assert!(matches!(result, CompareResult::PixelMismatch { .. }));
    }
}
