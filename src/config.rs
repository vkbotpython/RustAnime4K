//! JSON config manager.
//!
//! Location:
//! `%APPDATA%/Anime4K Player/config.json` on Windows,
//! `~/.config/Anime4K Player/config.json` elsewhere.

use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

pub const APP_DIR_NAME: &str = "Anime4K Player";

pub fn config_dir() -> PathBuf {
    #[cfg(windows)]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(APP_DIR_NAME)
    }
    #[cfg(not(windows))]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join(APP_DIR_NAME)
    }
}

pub struct ConfigManager {
    path: PathBuf,
    values: HashMap<String, Value>,
}

impl ConfigManager {
    pub fn new(filename: &str) -> Self {
        let mut mgr = Self {
            path: config_dir().join(filename),
            values: HashMap::new(),
        };
        mgr.load();
        mgr
    }

    #[cfg(test)]
    pub fn with_path(path: PathBuf) -> Self {
        let mut mgr = Self {
            path,
            values: HashMap::new(),
        };
        mgr.load();
        mgr
    }

    fn load(&mut self) {
        let Ok(text) = fs::read_to_string(&self.path) else {
            return;
        };
        match serde_json::from_str::<HashMap<String, Value>>(&text) {
            Ok(map) => self.values = map,
            Err(err) => eprintln!("config parse error ({}): {err}", self.path.display()),
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("cannot create {}", parent.display()))?;
        }
        let text = serde_json::to_string_pretty(&self.values)?;
        fs::write(&self.path, text)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.values.insert(key.to_string(), value);
    }

    /// Removes a stored key (used when stored state is corrupt).
    #[allow(dead_code)]
    pub fn delete(&mut self, key: &str) {
        self.values.remove(key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tmp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("anime4k-player-test-{}-{name}", std::process::id()))
    }

    #[test]
    fn set_get_delete_roundtrip() {
        let path = tmp_path("config.json");
        let _ = fs::remove_file(&path);
        let mut cfg = ConfigManager::with_path(path.clone());
        assert!(cfg.get("volume").is_none());
        cfg.set("volume", json!(80));
        assert_eq!(cfg.get("volume"), Some(&json!(80)));
        cfg.save().unwrap();

        let cfg2 = ConfigManager::with_path(path.clone());
        assert_eq!(cfg2.get("volume"), Some(&json!(80)));

        let mut cfg2 = cfg2;
        cfg2.delete("volume");
        assert!(cfg2.get("volume").is_none());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn broken_json_does_not_panic() {
        let path = tmp_path("broken.json");
        let _ = fs::create_dir_all(path.parent().unwrap());
        fs::write(&path, "{not json").unwrap();
        let cfg = ConfigManager::with_path(path.clone());
        assert!(cfg.get("anything").is_none());
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn missing_file_is_empty() {
        let cfg = ConfigManager::with_path(tmp_path("does-not-exist.json"));
        assert!(cfg.get("x").is_none());
    }
}
