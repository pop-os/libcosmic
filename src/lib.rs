mod deref_cell;
#[cfg(feature = "layer-shell")]
pub mod wayland;
#[cfg(feature = "layer-shell")]
mod wayland_custom_surface;
pub mod x;

pub use libcosmic_widgets as widgets;
