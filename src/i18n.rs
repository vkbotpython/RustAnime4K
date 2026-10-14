//! Localization.
//!
//! Strings live in `assets/i18n/<lang>.json` and are embedded at compile
//! time. Missing keys fall back to English.

use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Lang {
    #[default]
    Auto,
    English,
    Russian,
    Japanese,
    PortugueseBrazil,
    Spanish,
    German,
    French,
    Italian,
    ChineseSimplified,
    ChineseTraditional,
    Korean,
    Ukrainian,
    Polish,
    Turkish,
    Arabic,
    Hindi,
    Indonesian,
    Vietnamese,
    Thai,
    Dutch,
    Czech,
    Hungarian,
    Romanian,
    Swedish,
    Portuguese,
}

impl Lang {
    /// All selectable languages (excludes `Auto`).
    pub fn all() -> &'static [Lang] {
        use Lang::*;
        &[
            English,
            Russian,
            Japanese,
            PortugueseBrazil,
            Spanish,
            German,
            French,
            Italian,
            ChineseSimplified,
            ChineseTraditional,
            Korean,
            Ukrainian,
            Polish,
            Turkish,
            Arabic,
            Hindi,
            Indonesian,
            Vietnamese,
            Thai,
            Dutch,
            Czech,
            Hungarian,
            Romanian,
            Swedish,
            Portuguese,
        ]
    }

    pub fn from_config_value(value: Option<&str>) -> Self {
        let value = value.unwrap_or("Auto");
        if value == "Auto" {
            return Self::Auto;
        }
        Self::all()
            .iter()
            .find(|l| l.config_value() == value)
            .copied()
            .unwrap_or(Self::Auto)
    }

    pub fn config_value(self) -> &'static str {
        self.info().0
    }

    fn file_key(self) -> &'static str {
        self.info().1
    }

    /// (config display name, translation file key) per language.
    fn info(self) -> (&'static str, &'static str) {
        match self {
            Self::Auto => ("Auto", "en"),
            Self::English => ("English", "en"),
            Self::Russian => ("Русский", "ru"),
            Self::Japanese => ("Japanese", "ja"),
            Self::PortugueseBrazil => ("Português (Brasil)", "pt-BR"),
            Self::Spanish => ("Español", "es"),
            Self::German => ("Deutsch", "de"),
            Self::French => ("Français", "fr"),
            Self::Italian => ("Italiano", "it"),
            Self::ChineseSimplified => ("简体中文", "zh-CN"),
            Self::ChineseTraditional => ("繁體中文", "zh-TW"),
            Self::Korean => ("한국어", "ko"),
            Self::Ukrainian => ("Українська", "uk"),
            Self::Polish => ("Polski", "pl"),
            Self::Turkish => ("Türkçe", "tr"),
            Self::Arabic => ("العربية", "ar"),
            Self::Hindi => ("हिन्दी", "hi"),
            Self::Indonesian => ("Bahasa Indonesia", "id"),
            Self::Vietnamese => ("Tiếng Việt", "vi"),
            Self::Thai => ("ไทย", "th"),
            Self::Dutch => ("Nederlands", "nl"),
            Self::Czech => ("Čeština", "cs"),
            Self::Hungarian => ("Magyar", "hu"),
            Self::Romanian => ("Română", "ro"),
            Self::Swedish => ("Svenska", "sv"),
            Self::Portuguese => ("Português", "pt"),
        }
    }
}

fn load_table(key: &str) -> HashMap<String, String> {
    let text: &str = match key {
        "ru" => include_str!("../assets/i18n/ru.json"),
        "ja" => include_str!("../assets/i18n/ja.json"),
        "pt-BR" => include_str!("../assets/i18n/pt-BR.json"),
        "es" => include_str!("../assets/i18n/es.json"),
        "de" => include_str!("../assets/i18n/de.json"),
        "fr" => include_str!("../assets/i18n/fr.json"),
        "it" => include_str!("../assets/i18n/it.json"),
        "zh-CN" => include_str!("../assets/i18n/zh-CN.json"),
        "zh-TW" => include_str!("../assets/i18n/zh-TW.json"),
        "ko" => include_str!("../assets/i18n/ko.json"),
        "uk" => include_str!("../assets/i18n/uk.json"),
        "pl" => include_str!("../assets/i18n/pl.json"),
        "tr" => include_str!("../assets/i18n/tr.json"),
        "ar" => include_str!("../assets/i18n/ar.json"),
        "hi" => include_str!("../assets/i18n/hi.json"),
        "id" => include_str!("../assets/i18n/id.json"),
        "vi" => include_str!("../assets/i18n/vi.json"),
        "th" => include_str!("../assets/i18n/th.json"),
        "nl" => include_str!("../assets/i18n/nl.json"),
        "cs" => include_str!("../assets/i18n/cs.json"),
        "hu" => include_str!("../assets/i18n/hu.json"),
        "ro" => include_str!("../assets/i18n/ro.json"),
        "sv" => include_str!("../assets/i18n/sv.json"),
        "pt" => include_str!("../assets/i18n/pt.json"),
        _ => include_str!("../assets/i18n/en.json"),
    };
    serde_json::from_str(text).expect("i18n json must parse")
}

fn tables() -> &'static HashMap<&'static str, HashMap<String, String>> {
    static TABLES: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut m = HashMap::new();
        for key in core::iter::once("en").chain(Lang::all().iter().map(|l| l.file_key())) {
            m.entry(key).or_insert_with(|| load_table(key));
        }
        m
    })
}

/// Detects the system language.
pub fn detect_system_lang() -> Lang {
    let locale = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
    let loc = locale.to_lowercase();
    // Script/region disambiguation first.
    if loc.starts_with("zh") {
        if loc.contains("tw") || loc.contains("hk") || loc.contains("hant") || loc.contains("mo") {
            return Lang::ChineseTraditional;
        }
        return Lang::ChineseSimplified;
    }
    if loc.starts_with("pt") {
        return if loc.contains("br") {
            Lang::PortugueseBrazil
        } else {
            Lang::Portuguese
        };
    }
    let short = loc.split(['_', '-', '.']).next().unwrap_or("en");
    match short {
        "ru" => Lang::Russian,
        "ja" => Lang::Japanese,
        "es" => Lang::Spanish,
        "de" => Lang::German,
        "fr" => Lang::French,
        "it" => Lang::Italian,
        "ko" => Lang::Korean,
        "uk" => Lang::Ukrainian,
        "pl" => Lang::Polish,
        "tr" => Lang::Turkish,
        "ar" => Lang::Arabic,
        "hi" => Lang::Hindi,
        "id" | "in" => Lang::Indonesian,
        "vi" => Lang::Vietnamese,
        "th" => Lang::Thai,
        "nl" => Lang::Dutch,
        "cs" => Lang::Czech,
        "hu" => Lang::Hungarian,
        "ro" => Lang::Romanian,
        "sv" => Lang::Swedish,
        _ => Lang::English,
    }
}

pub fn resolve(config_value: Option<&str>) -> Lang {
    let lang = Lang::from_config_value(config_value);
    if lang == Lang::Auto {
        detect_system_lang()
    } else {
        lang
    }
}

/// Translates `key` for `lang`, falling back to English and then to the key.
/// Kept for dynamic (non-`'static`) keys; hot UI paths use [`t_static`].
#[allow(dead_code)]
pub fn t(lang: Lang, key: &str) -> String {
    let lang = if lang == Lang::Auto {
        detect_system_lang()
    } else {
        lang
    };
    let tables = tables();
    if let Some(s) = tables.get(lang.file_key()).and_then(|table| table.get(key)) {
        return s.clone();
    }
    if let Some(s) = tables.get("en").and_then(|table| table.get(key)) {
        return s.clone();
    }
    key.to_string()
}

/// Borrowed translation for hot UI paths: zero-alloc when the key exists
/// (all app call sites use `'static` literals). Falls back to English,
/// then to the key itself — never allocates.
pub fn t_static(lang: Lang, key: &'static str) -> &'static str {
    let lang = if lang == Lang::Auto {
        detect_system_lang()
    } else {
        lang
    };
    let tables = tables();
    if let Some(s) = tables.get(lang.file_key()).and_then(|table| table.get(key)) {
        return s.as_str();
    }
    if let Some(s) = tables.get("en").and_then(|table| table.get(key)) {
        return s.as_str();
    }
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_langs_have_all_english_keys() {
        let tables = tables();
        let en = &tables["en"];
        assert!(!en.is_empty());
        for lang in Lang::all() {
            let table = &tables[lang.file_key()];
            for key in en.keys() {
                assert!(
                    table.contains_key(key),
                    "missing {key} in {}",
                    lang.file_key()
                );
            }
        }
    }

    #[test]
    fn language_count() {
        // Auto + every entry in `all()`.
        assert_eq!(Lang::all().len(), 25);
    }

    #[test]
    fn config_values_roundtrip() {
        for lang in Lang::all() {
            assert_eq!(Lang::from_config_value(Some(lang.config_value())), *lang);
        }
        assert_eq!(Lang::from_config_value(Some("Auto")), Lang::Auto);
        assert_eq!(Lang::from_config_value(None), Lang::Auto);
        assert_eq!(Lang::from_config_value(Some("???")), Lang::Auto);
    }

    #[test]
    fn fallback_returns_key() {
        assert_eq!(t(Lang::English, "no-such-key-xyz"), "no-such-key-xyz");
    }

    #[test]
    fn static_borrow_matches_owned_and_never_allocs_on_miss() {
        // Same visible strings as the owned lookup.
        assert_eq!(t_static(Lang::Russian, "File"), "Файл");
        assert_eq!(t_static(Lang::English, "File"), "File");
        for lang in Lang::all() {
            for key in ["File", "Play", "Volume level", "Error", "Playlist"] {
                assert_eq!(t_static(*lang, key), t(*lang, key));
            }
        }
        // Miss returns the key itself, no allocation (pointer-identical).
        let key: &'static str = "no-such-key-xyz-static";
        assert!(std::ptr::eq(t_static(Lang::English, key), key));
        assert!(std::ptr::eq(t_static(Lang::Russian, key), key));
    }

    #[test]
    fn russian_translation_differs() {
        assert_eq!(t(Lang::Russian, "File"), "Файл");
        assert_eq!(t(Lang::English, "File"), "File");
    }

    #[test]
    fn config_mapping() {
        assert_eq!(Lang::from_config_value(Some("Русский")), Lang::Russian);
        assert_eq!(Lang::from_config_value(None), Lang::Auto);
        assert_eq!(Lang::from_config_value(Some("???")), Lang::Auto);
    }
}
