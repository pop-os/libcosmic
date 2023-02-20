use crate::font;
use std::cell::RefCell;

thread_local! {
    /// The fallback icon theme to search if no icon theme was specified.
    pub(crate) static DEFAULT_ICON_THEME: RefCell<String> = RefCell::new(String::from("Pop"));
}

/// The fallback icon theme to search if no icon theme was specified.
#[must_use]
pub fn default_icon_theme() -> String {
    DEFAULT_ICON_THEME.with(|f| f.borrow().clone())
}

/// Set the fallback icon theme to search when loading system icons.
pub fn set_default_icon_theme(name: impl Into<String>) {
    DEFAULT_ICON_THEME.with(|f| *f.borrow_mut() = name.into());
}

/// Default iced settings for COSMIC applications.
#[must_use]
pub fn settings<Flags: Default>() -> iced::Settings<Flags> {
    settings_with_flags(Flags::default())
}

/// Default iced settings for COSMIC applications.
#[must_use]
pub fn settings_with_flags<Flags>(flags: Flags) -> iced::Settings<Flags> {
    iced::Settings {
        default_font: match font::FONT {
            iced::Font::Default => None,
            iced::Font::External { bytes, .. } => Some(bytes),
        },
        default_text_size: 18,
        ..iced::Settings::with_flags(flags)
    }
}
