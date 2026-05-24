use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(default)]
    pub keystore: KeystoreConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum KeystoreConfig {
    #[default]
    #[serde(rename = "file")]
    File,
    #[serde(rename = "keepassxc")]
    KeePassXc {
        db_path: String,
        entry: String,
    },
    #[serde(rename = "keepass-rs")]
    KeePassRs {
        db_path: String,
        entry: String,
    },
}

impl Default for Config {
    fn default() -> Self {
        Self { keystore: KeystoreConfig::File }
    }
}

impl Config {
    pub fn config_dir() -> PathBuf {
        let from_env = std::env::var("RR_CONFIG_DIR").ok().map(PathBuf::from);
        from_env.unwrap_or_else(|| {
            let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("~/.config"));
            base.join("reseau-racine")
        })
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if !path.exists() {
            return Self::default();
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };
        toml::from_str(&content).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let content = toml::to_string_pretty(self)?;
        std::fs::write(Self::config_path(), content)?;
        Ok(())
    }
}
