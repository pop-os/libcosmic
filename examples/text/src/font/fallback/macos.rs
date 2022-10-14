use unicode_script::Script;

// Fallbacks to use after any script specific fallbacks
pub fn common_fallback() -> &'static [&'static str] {
    &[
        ".SF NS",
        "Apple Color Emoji",
    ]
}

// Fallbacks to never use
pub fn forbidden_fallback() -> &'static [&'static str] {
    &[]
}

// Fallbacks to use per script
pub fn script_fallback(script: &Script, locale: &str) -> &'static [&'static str] {
    //TODO: abstract style (sans/serif/monospaced)
    match script {
        Script::Adlam => &["Noto Sans Adlam"],
        Script::Armenian => &["Noto Sans Armenian"],
        Script::Buhid => &["Noto Sans Buhid"],
        Script::Chakma => &["Noto Sans Chakma"],
        Script::Gothic => &["Noto Sans Gothic"],
        Script::Hanunoo => &["Noto Sans Hanunoo"],
        Script::Javanese => &["Noto Sans Javanese"],
        Script::Kannada => &["Noto Sans Kannada"],
        Script::Mongolian => &["Noto Sans Mongolian"],
        Script::Myanmar => &["Noto Sans Myanmar"],
        Script::Oriya => &["Noto Sans Oriya"],
        Script::Syriac => &["Noto Sans Syriac"],
        Script::Tagalog => &["Noto Sans Tagalog"],
        Script::Tagbanwa => &["Noto Sans Tagbanwa"],
        Script::Tai_Le => &["Noto Sans Tai Le"],
        Script::Tai_Tham => &["Noto Sans Tai Tham"],
        Script::Tai_Viet => &["Noto Sans Tai Viet"],
        Script::Thaana => &["Noto Sans Thaana"],
        Script::Tifinagh => &["Noto Sans Tifinagh"],
        Script::Vai => &["Noto Sans Vai"],
        //TODO: Use han_unification?
        Script::Yi => &["Noto Sans Yi"],
        _ => &[],
    }
}
