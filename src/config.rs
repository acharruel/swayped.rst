use std::{env, fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TomlConfig {
    pub mappings: Vec<Mapping>,
}

#[derive(Debug, Deserialize)]
pub struct Mapping {
    pub gesture: String,
    pub cmd: String,
    pub cmd_type: String,
    pub finger_count: Option<i32>,
}

impl TomlConfig {
    pub fn new(file: PathBuf) -> Result<Self> {
        let file = if let Some(file) = file.to_str() {
            file
        } else {
            bail!("Unrecognized configuration file path");
        };
        let content = fs::read_to_string(file)
            .context(format!("Failed to read configuration file '{}'", file))?;
        let decoded: TomlConfig =
            toml::from_str(&content).context("Failed to decode toml configuration")?;
        Ok(decoded)
    }

    pub fn config_dir() -> PathBuf {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
            .map(|p| p.join("swayped"))
            .expect("Failed to get config directory")
    }
}
