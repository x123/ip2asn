use crate::error::CliError;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default)]
    pub auto_update: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { auto_update: false }
    }
}

impl Config {
    pub fn load() -> Result<Self, CliError> {
        let config_path = if let Ok(path) = std::env::var("IP2ASN_CONFIG_PATH") {
            std::path::PathBuf::from(path)
        } else if let Some(home_dir) = home::home_dir() {
            home_dir.join(".config/ip2asn/config.toml")
        } else {
            return Ok(Self::default());
        };

        if !config_path.exists() {
            return Ok(Self::default());
        }

        let contents = std::fs::read_to_string(&config_path).map_err(|e| {
            CliError::Config(format!(
                "Failed to read config file at {}: {}",
                config_path.display(),
                e
            ))
        })?;

        toml::from_str(&contents).map_err(|e| {
            CliError::Config(format!(
                "Failed to parse config file at {}: {}",
                config_path.display(),
                e
            ))
        })
    }
}
