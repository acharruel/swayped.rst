use std::fs;

use anyhow::Result;
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
    pub fn new(file: &str) -> Result<Self> {
        let content = fs::read_to_string(file).expect("Failed to read toml configuration file");
        let decoded: TomlConfig =
            toml::from_str(&content).expect("Failed to decode toml configuration");

        Ok(decoded)
    }
}
