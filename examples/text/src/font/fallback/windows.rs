use unicode_script::Script;

// Fallbacks to use after any script specific fallbacks
pub fn common_fallback() -> &'static [&'static str] {
    //TODO: abstract style (sans/serif/monospaced)
    &[
        "Segoe UI",
        "Segoe UI Emoji",
        "Segoe UI Symbol",
        "Segoe UI Historic",
        //TODO: Add CJK script here for doublewides?
    ]
}

fn han_unification(locale: &str) -> &'static [&'static str] {
    //TODO!
    match locale {
        // Japan
        "ja" => &[],
        // Korea
        "ko" => &[],
        // China
        "zh-CN" => &[],
        // Hong Kong
        "zh-HK" => &[],
        // Taiwan
        "zh-TW" => &[],
        // Simplified Chinese is the default
        _ => &[],
    }
}

// Fallbacks to use per script
pub fn script_fallback(script: &Script, locale: &str) -> &'static [&'static str] {
    //TODO: better match https://github.com/chromium/chromium/blob/master/third_party/blink/renderer/platform/fonts/win/font_fallback_win.cc#L99
    match script {
        Script::Bengali => &["Nirmala UI"],
        Script::Canadian_Aboriginal => &["Gadugi"],
        Script::Cherokee => &["Gadugi"],
        Script::Devanagari => &["Nirmala UI"],
        Script::Ethiopic => &["Ebrima"],
        Script::Gujarati => &["Nirmala UI"],
        Script::Gurmukhi => &["Nirmala UI"],
        Script::Han => han_unification(locale),
        Script::Hangul => &["Malgun Gothic"],
        Script::Hiragana => &["Meiryo"]
        Script::Kannada => &["Nirmala UI"],
        Script::Katakana => &["Meiryo"]
        Script::Khmer => &["Leelawadee UI"],
        Script::Lao => &["Leelawadee UI"],
        Script::Malayalam => &["Nirmala UI"],
        Script::Mongolian => &["Mongolian Baiti"],
        Script::Myanmar => &["Myanmar Text"],
        Script::Oriya => &["Nirmala UI"],
        Script::Sinhala => &["Nirmala UI"],
        Script::Tamil => &["Nirmala UI"],
        Script::Telugu => &["Nirmala UI"],
        Script::Thaana => &["MV Boli"],
        Script::Thai => &["Leelawadee UI"],
        Script::Tibetan => &["Microsoft Himalaya"],
        Script::Yi => &["Microsoft Yi Baiti"],
        _ => &[],
    }
}
