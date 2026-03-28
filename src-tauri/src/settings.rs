use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub hotkey: String,
    pub max_clips: i64,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: "Shift+CmdOrCtrl+V".to_string(),
            max_clips: 500,
        }
    }
}

impl Settings {
    fn config_path(app_data_dir: &std::path::Path) -> PathBuf {
        app_data_dir.join("settings.json")
    }

    pub fn load(app_data_dir: &std::path::Path) -> Self {
        let path = Self::config_path(app_data_dir);
        if let Ok(data) = fs::read_to_string(&path) {
            serde_json::from_str(&data).unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self, app_data_dir: &std::path::Path) -> Result<(), String> {
        let path = Self::config_path(app_data_dir);
        let data = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, data).map_err(|e| e.to_string())
    }
}
