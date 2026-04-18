use std::path::PathBuf;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub path: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub last_used: Option<i64>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    pub config_path: Option<String>,
    pub username: Option<String>,
    /// Password stored as base64 (not encryption — just avoids plaintext on disk)
    pub password_b64: Option<String>,
    #[serde(default)]
    pub remember_credentials: bool,
    #[serde(default)]
    pub enable_logs: bool,
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default)]
    pub profiles: Vec<Profile>,
}

fn default_theme() -> String {
    "Dark".to_string()
}

impl Settings {
    pub fn password(&self) -> String {
        self.password_b64
            .as_ref()
            .and_then(|b| BASE64.decode(b).ok())
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_default()
    }

    pub fn set_password(&mut self, password: &str) {
        if password.is_empty() {
            self.password_b64 = None;
        } else {
            self.password_b64 = Some(BASE64.encode(password.as_bytes()));
        }
    }
}

fn settings_path() -> Option<PathBuf> {
    let config_dir = dirs::config_dir()?;
    let app_dir = config_dir.join("openvpn-gui-linux");
    Some(app_dir.join("settings.toml"))
}

pub fn load() -> Settings {
    let Some(path) = settings_path() else {
        return Settings::default();
    };

    let Ok(content) = std::fs::read_to_string(&path) else {
        return Settings::default();
    };

    let mut settings: Settings = toml::from_str(&content).unwrap_or_default();

    // Migrate legacy single-profile setups: lift `config_path` into the list
    // the first time the user opens this build.
    if settings.profiles.is_empty()
        && let Some(legacy) = &settings.config_path
    {
        settings.profiles.push(Profile {
            path: legacy.clone(),
            display_name: None,
            last_used: None,
        });
    }

    settings
}

pub fn save(settings: &Settings) {
    let Some(path) = settings_path() else {
        return;
    };

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    if let Ok(content) = toml::to_string_pretty(settings) {
        let _ = std::fs::write(&path, content);
    }
}
