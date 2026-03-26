use crate::widget::svg;
use std::sync::OnceLock;

pub fn object_select() -> &'static svg::Handle {
    static SELECTION_ICON: OnceLock<svg::Handle> = OnceLock::new();

    SELECTION_ICON.get_or_init(|| {
        iced_core::svg::Handle::from_memory(icetron_assets::icons::system::CHECK_LINE)
    })
}
