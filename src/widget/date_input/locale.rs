// Copyright 2026 System76 <info@system76.com>
// SPDX-License-Identifier: MPL-2.0

use std::sync::LazyLock;

use super::format::Format;

static LOCALE_FORMATS: LazyLock<&'static [Format<'static>]> = LazyLock::new(|| {
    static DMY: &[Format<'static>] = &[Format::strftime("%d/%m/%Y", "DD/MM/YYYY")];
    static DMY_DOT: &[Format<'static>] = &[Format::strftime("%d.%m.%Y", "DD.MM.YYYY")];
    static MDY: &[Format<'static>] = &[Format::strftime("%m/%d/%Y", "MM/DD/YYYY")];
    static YMD: &[Format<'static>] = &[Format::strftime("%Y/%m/%d", "YYYY/MM/DD")];
    static ISO: &[Format<'static>] = &[Format::ISO];

    let locale = locale();
    let region = region(&locale).unwrap_or_default().to_ascii_uppercase();
    match region.as_str() {
        "US" | "PH" => MDY,
        "CN" | "JP" | "KR" | "TW" => YMD,
        "CZ" | "DK" | "EE" | "FI" | "HU" | "LT" | "LV" | "NO" | "PL" | "SE" | "SK" => ISO,
        "AT" | "CH" | "DE" => DMY_DOT,
        "" if locale.starts_with("en") => MDY,
        "" => ISO,
        _ => DMY,
    }
});

fn env_locale(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|value| !value.is_empty())
}

fn config_locale<'a>(contents: &'a str, key: &str) -> Option<&'a str> {
    contents.lines().find_map(|line| {
        let (name, value) = line.trim().split_once('=')?;
        let value = value.trim().trim_matches(['\'', '"']);

        (name == key && !value.is_empty()).then_some(value)
    })
}

/// Returns the locale used for date formatting.
///
/// `LC_TIME` is the POSIX category for date and time representation, so it
/// takes priority over the general language locale. `sys-locale` does not read
/// `LC_TIME`, so it is only used as a fallback.
fn locale() -> String {
    if let Some(locale) = env_locale("LC_ALL").or_else(|| env_locale("LC_TIME")) {
        return locale;
    }

    for path in ["/etc/locale.conf", "/etc/default/locale"] {
        let Ok(contents) = std::fs::read_to_string(path) else {
            continue;
        };
        if let Some(locale) =
            config_locale(&contents, "LC_ALL").or_else(|| config_locale(&contents, "LC_TIME"))
        {
            return locale.to_owned();
        }
    }

    sys_locale::get_locale().unwrap_or_default()
}

fn region(locale: &str) -> Option<&str> {
    let locale = locale.split(['.', '@']).next()?;
    locale.rsplit_once(['-', '_']).map(|(_, region)| region)
}

pub(super) fn locale_formats() -> &'static [Format<'static>] {
    *LOCALE_FORMATS
}
