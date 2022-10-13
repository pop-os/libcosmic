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

// Fallbacks to use per script
pub fn script_fallback(script: &Script, locale: &str) -> &'static [&'static str] {
    match script {
        Script::Bengali => &["Nirmala UI"],
        Script::Devanagari => &["Nirmala UI"],
        Script::Gujarati => &["Nirmala UI"],
        Script::Gurmukhi => &["Nirmala UI"],
        Script::Tamil => &["Nirmala UI"],
        Script::Telugu => &["Nirmala UI"],
        Script::Thaana => &["MV Boli"],
        _ => &[],
    }
}
