pub mod circular;
pub mod linear;
pub mod style;

/// A spinner / throbber widget that can be used to indicate that some operation is in progress.
pub fn indeterminate_circular() -> circular::Circular<crate::Theme> {
    circular::Circular::new()
}

/// A linear throbber widget that can be used to indicate that some operation is in progress.
pub fn indeterminate_linear() -> linear::Linear<crate::Theme> {
    linear::Linear::new()
}

/// A circular progress spinner widget that can be used to indicate the progress of some operation.
pub fn determinate_circular(progress: f32) -> circular::Circular<crate::Theme> {
    circular::Circular::new().progress(progress)
}

/// A linear progress bar widget that can be used to indicate the progress of some operation.
pub fn determinate_linear(progress: f32) -> linear::Linear<crate::Theme> {
    linear::Linear::new().progress(progress)
}
