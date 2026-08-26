//! Display names for YouTube language / translation codes.

pub fn language_name(code: &str) -> String {
    let key = code.trim();
    for (c, name) in NAMES {
        if c.eq_ignore_ascii_case(key) {
            return (*name).to_string();
        }
    }
    let base = key.split(['-', '_']).next().unwrap_or(key);
    for (c, name) in NAMES {
        if c.eq_ignore_ascii_case(base) {
            return format!("{name} ({key})");
        }
    }
    key.to_string()
}

/// Languages YouTube can auto-translate captions into (subset of the ~125 list).
pub fn translation_catalog() -> Vec<(String, String)> {
    TRANSLATIONS
        .iter()
        .map(|(c, n)| ((*c).to_string(), (*n).to_string()))
        .collect()
}

const NAMES: &[(&str, &str)] = &[
    ("en", "English"),
    ("en-US", "English (US)"),
    ("en-GB", "English (UK)"),
    ("es", "Spanish"),
    ("es-419", "Spanish (Latin America)"),
    ("es-ES", "Spanish (Spain)"),
    ("pt", "Portuguese"),
    ("pt-BR", "Portuguese (Brazil)"),
    ("pt-PT", "Portuguese (Portugal)"),
    ("fr", "French"),
    ("de", "German"),
    ("de-DE", "German"),
    ("it", "Italian"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("zh", "Chinese"),
    ("zh-Hans", "Chinese (Simplified)"),
    ("zh-Hant", "Chinese (Traditional)"),
    ("zh-CN", "Chinese (Simplified)"),
    ("zh-TW", "Chinese (Traditional)"),
    ("ar", "Arabic"),
    ("hi", "Hindi"),
    ("ru", "Russian"),
    ("id", "Indonesian"),
    ("tr", "Turkish"),
    ("nl", "Dutch"),
    ("pl", "Polish"),
    ("sv", "Swedish"),
    ("uk", "Ukrainian"),
    ("vi", "Vietnamese"),
    ("th", "Thai"),
    ("cs", "Czech"),
    ("ro", "Romanian"),
    ("hu", "Hungarian"),
    ("el", "Greek"),
    ("he", "Hebrew"),
    ("da", "Danish"),
    ("fi", "Finnish"),
    ("no", "Norwegian"),
    ("nb", "Norwegian"),
    ("sk", "Slovak"),
    ("bg", "Bulgarian"),
    ("hr", "Croatian"),
    ("sr", "Serbian"),
    ("ca", "Catalan"),
    ("ms", "Malay"),
    ("fil", "Filipino"),
    ("tl", "Filipino"),
    ("bn", "Bengali"),
    ("ta", "Tamil"),
    ("te", "Telugu"),
    ("ur", "Urdu"),
    ("fa", "Persian"),
    ("sw", "Swahili"),
];

const TRANSLATIONS: &[(&str, &str)] = &[
    ("en", "English"),
    ("es", "Spanish"),
    ("pt", "Portuguese"),
    ("fr", "French"),
    ("de", "German"),
    ("it", "Italian"),
    ("ja", "Japanese"),
    ("ko", "Korean"),
    ("zh-Hans", "Chinese (Simplified)"),
    ("zh-Hant", "Chinese (Traditional)"),
    ("ar", "Arabic"),
    ("hi", "Hindi"),
    ("ru", "Russian"),
    ("id", "Indonesian"),
    ("tr", "Turkish"),
    ("nl", "Dutch"),
    ("pl", "Polish"),
    ("sv", "Swedish"),
    ("uk", "Ukrainian"),
    ("vi", "Vietnamese"),
    ("th", "Thai"),
    ("cs", "Czech"),
    ("ro", "Romanian"),
    ("hu", "Hungarian"),
    ("el", "Greek"),
    ("he", "Hebrew"),
    ("da", "Danish"),
    ("fi", "Finnish"),
    ("no", "Norwegian"),
    ("ca", "Catalan"),
    ("ms", "Malay"),
    ("fil", "Filipino"),
    ("bn", "Bengali"),
    ("ta", "Tamil"),
    ("fa", "Persian"),
];
