// SPDX-License-Identifier: GPL-3.0-only

use i18n_embed::{
    DefaultLocalizer, LanguageLoader, Localizer,
    fluent::{FluentLanguageLoader, fluent_language_loader},
};
use rust_embed::RustEmbed;
use std::sync::{LazyLock, OnceLock};

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

pub static LANGUAGE_LOADER: LazyLock<FluentLanguageLoader> = LazyLock::new(|| {
    let loader: FluentLanguageLoader = fluent_language_loader!();

    loader
        .load_fallback_language(&Localizations)
        .expect("Error while loading fallback language");

    loader
});

static LOCALIZATION_INITIALIZED: OnceLock<()> = OnceLock::new();

#[macro_export]
macro_rules! fl {
    ($message_id:literal) => {{
        $crate::localize::localize();
        i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id)
    }};
    ($message_id:literal, $($args:expr),*) => {{
        $crate::localize::localize();
        i18n_embed_fl::fl!($crate::localize::LANGUAGE_LOADER, $message_id, $($args), *)
    }};
}

// Get the `Localizer` to be used for localizing this library.
pub fn localizer() -> Box<dyn Localizer> {
    Box::from(DefaultLocalizer::new(&*LANGUAGE_LOADER, &Localizations))
}

pub fn localize() {
    LOCALIZATION_INITIALIZED.get_or_init(|| {
        let localizer = localizer();
        let requested_languages = i18n_embed::DesktopLanguageRequester::requested_languages();
        if let Err(error) = localizer.select(&requested_languages) {
            eprintln!("Error while loading language for libcosmic {}", error);
        }
    });
}
