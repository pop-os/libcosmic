use unicode_script::Script;

// Fallbacks to use after any script specific fallbacks
pub fn common_fallback() -> &'static [&'static str] {
    //TODO: abstract style (sans/serif/monospaced)
    &[
        "Segoe UI",
        "FreeSans",
        "Segoe UI Symbol",
        "Segoe UI Emoji",
        //TODO: Add CJK script here for doublewides?
    ]
}

// Fallbacks to use per script
pub fn script_fallback(script: &Script, locale: &str) -> &'static [&'static str] {
    //TODO: per-script fallbacks
    &[]
}
