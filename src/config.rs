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
        // Atomic write: a crash/power loss mid-write must not leave a torn
        // `config.json`. Temp file + rename in the same directory.
        let text = serde_json::to_string_pretty(&self.values)?;
        let tmp = self.path.with_extension("json.tmp");
        fs::write(&tmp, text)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }

    pub fn set(&mut self, key: &str, value: Value) {
        self.values.insert(key.to_string(), value);
    }

    /// Inserts `key/value` into the object stored under `object_key`
    /// without cloning the whole table (`write_file_preset` used to clone
    /// every stored file preset on each preset choice — O(N) per apply).
    /// Caps the object at `max_entries`, evicting an arbitrary entry when
    /// full so per-file tables (`filePresets`) stay bounded.
    pub fn set_nested_capped(
        &mut self,
        object_key: &str,
        key: &str,
        value: Value,
        max_entries: usize,
    ) {
        let entry = self
            .values
            .entry(object_key.to_string())
            .or_insert_with(|| Value::Object(Default::default()));
        let Value::Object(map) = entry else {
            *entry = Value::Object([(key.to_string(), value)].into_iter().collect());
            return;
        };
        map.insert(key.to_string(), value);
        while map.len() > max_entries {
            if let Some(first) = map.keys().next().cloned() {
                map.remove(&first);
            } else {
                break;
            }
        }
    }

    /// Removes a stored key (used when stored state is corrupt).
    #[allow(dead_code)]
    pub fn delete(&mut self, key: &str) {
        self.values.remove(key);
    }

    /// Prepends `value` to the string-array stored under `key` (MRU order),
    /// dropping duplicates and truncating to `max_entries`. In-place, no
    /// whole-table clone, so histories stay bounded (see `MAX_HISTORY_ENTRIES`).
    pub fn push_history_capped(&mut self, key: &str, value: &str, max_entries: usize) {
        let value = value.trim();
        if value.is_empty() {
            return;
        }
        let entry = self
            .values
            .entry(key.to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        let Value::Array(arr) = entry else {
            *entry = Value::Array(vec![Value::String(value.to_string())]);
            return;
        };
        arr.retain(|v| v.as_str() != Some(value));
        arr.insert(0, Value::String(value.to_string()));
        if arr.len() > max_entries {
            arr.truncate(max_entries);
        }
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

    #[test]
    fn save_is_atomic_and_readable() {
        let path = tmp_path("atomic.json");
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(path.with_extension("json.tmp"));
        let mut cfg = ConfigManager::with_path(path.clone());
        cfg.set("volume", json!(80));
        cfg.save().unwrap();
        // No temp leftover, main file parses.
        assert!(!path.with_extension("json.tmp").exists());
        let text = fs::read_to_string(&path).unwrap();
        let map: HashMap<String, Value> = serde_json::from_str(&text).unwrap();
        assert_eq!(map.get("volume"), Some(&json!(80)));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn nested_capped_inserts_without_cloning_and_bounds_growth() {
        let path = tmp_path("nested.json");
        let _ = fs::remove_file(&path);
        let mut cfg = ConfigManager::with_path(path.clone());
        cfg.set_nested_capped("filePresets", "a.mkv", json!({"disabled": true}), 3);
        cfg.set_nested_capped("filePresets", "b.mkv", json!({"disabled": true}), 3);
        assert_eq!(
            cfg.get("filePresets").unwrap().as_object().unwrap().len(),
            2
        );
        // Existing entries survive new inserts (no whole-table replace).
        cfg.set_nested_capped("filePresets", "c.mkv", json!({"disabled": true}), 3);
        cfg.set_nested_capped("filePresets", "d.mkv", json!({"disabled": true}), 3);
        let map = cfg.get("filePresets").unwrap().as_object().unwrap();
        assert_eq!(map.len(), 3);
        assert!(map.contains_key("d.mkv"));
        // Overwriting a non-object value replaces it with an object.
        cfg.set("filePresets", json!("garbage"));
        cfg.set_nested_capped("filePresets", "e.mkv", json!({"disabled": true}), 3);
        let map = cfg.get("filePresets").unwrap().as_object().unwrap();
        assert_eq!(map.len(), 1);
        assert!(map.contains_key("e.mkv"));
        let _ = fs::remove_file(&path);
    }
}
