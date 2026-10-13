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
}

impl Lang {
    pub fn from_config_value(value: Option<&str>) -> Self {
        match value {
            Some("Русский") => Self::Russian,
            Some("English") => Self::English,
            Some("Japanese") => Self::Japanese,
            Some("Português (Brasil)") => Self::PortugueseBrazil,
            _ => Self::Auto,
        }
    }

    pub fn config_value(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::English => "English",
            Self::Russian => "Русский",
            Self::Japanese => "Japanese",
            Self::PortugueseBrazil => "Português (Brasil)",
        }
    }

    fn file_key(self) -> &'static str {
        match self {
            Self::Auto | Self::English => "en",
            Self::Russian => "ru",
            Self::Japanese => "ja",
            Self::PortugueseBrazil => "pt-BR",
        }
    }
}

fn load_table(key: &str) -> HashMap<String, String> {
    let text: &str = match key {
        "ru" => include_str!("../assets/i18n/ru.json"),
        "ja" => include_str!("../assets/i18n/ja.json"),
        "pt-BR" => include_str!("../assets/i18n/pt-BR.json"),
        _ => include_str!("../assets/i18n/en.json"),
    };
    serde_json::from_str(text).expect("i18n json must parse")
}

fn tables() -> &'static HashMap<&'static str, HashMap<String, String>> {
    static TABLES: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();
    TABLES.get_or_init(|| {
        HashMap::from([
            ("en", load_table("en")),
            ("ru", load_table("ru")),
            ("ja", load_table("ja")),
            ("pt-BR", load_table("pt-BR")),
        ])
    })
}

/// Detects the system language.
pub fn detect_system_lang() -> Lang {
    let locale = sys_locale::get_locale().unwrap_or_else(|| "en".to_string());
    let short = locale.split(['_', '-', '.']).next().unwrap_or("en");
    match short.to_lowercase().as_str() {
        "ru" => Lang::Russian,
        "ja" => Lang::Japanese,
        "pt" => Lang::PortugueseBrazil,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_langs_have_all_english_keys() {
        let tables = tables();
        let en = &tables["en"];
        assert!(!en.is_empty());
        for lang in ["ru", "ja", "pt-BR"] {
            for key in en.keys() {
                assert!(tables[lang].contains_key(key), "missing {key} in {lang}");
            }
        }
    }

    #[test]
    fn fallback_returns_key() {
        assert_eq!(t(Lang::English, "no-such-key-xyz"), "no-such-key-xyz");
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
