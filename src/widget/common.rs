use crate::widget::svg;
use std::sync::OnceLock;

/// Static `svg::Handle` to the `object-select-symbolic` icon.
pub fn object_select() -> &'static svg::Handle {
    static SELECTION_ICON: OnceLock<svg::Handle> = OnceLock::new();

    SELECTION_ICON.get_or_init(|| {
        crate::widget::icon::from_name("object-select-symbolic")
            .size(16)
            .icon()
            .into_svg_handle()
            .unwrap_or_else(|| {
                let bytes: &'static [u8] = &[];
                iced_core::svg::Handle::from_memory(bytes)
            })
    })
}
