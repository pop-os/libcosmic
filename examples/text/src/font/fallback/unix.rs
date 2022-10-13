use unicode_script::Script;

// Fallbacks to use after any script specific fallbacks
pub fn common_fallback() -> &'static [&'static str] {
    //TODO: abstract style (sans/serif/monospaced)
    &[
        "Fira Sans",
        "DejaVu Sans",
        "DejaVu Serif",
        "Noto Color Emoji",
    ]
}

// Fallbacks to use per script
pub fn script_fallback(script: &Script, locale: &str) -> &'static [&'static str] {
    //TODO: abstract style (sans/serif/monospaced)
    match script {
        Script::Han => match locale {
            // Japan
            "ja" => &["Noto Sans CJK JA"],
            // Korea
            "ko" => &["Noto Sans CJK KR"],
            // China
            "zh-CN" => &["Noto Sans CJK SC"],
            // Hong Kong
            "zh-HK" => &["Noto Sans CJK HK"],
            // Taiwan
            "zh-TW" => &["Noto Sans CJK TC"],
            // Simplified Chinese is the default
            _ => &["Noto Sans CJK SC"],
        }
        _ => &[],
    }
}
