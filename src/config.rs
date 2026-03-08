use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const CONFIG_PATH: &str = "/etc/hourglass/config.toml";

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub users: HashMap<String, UserConfig>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserConfig {
    pub daily_limit_minutes: u32,
}

impl Config {
    pub fn load() -> Config {
        let path = Path::new(CONFIG_PATH);
        if !path.exists() {
            return Config::default();
        }
        let text = fs::read_to_string(path).unwrap_or_default();
        toml::from_str(&text).unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Path::new(CONFIG_PATH);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }
        fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}
