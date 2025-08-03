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
    pub fn load(path: Option<&std::path::Path>) -> Result<Self, CliError> {
        let config_path = match path {
            Some(p) => p.to_path_buf(),
            None => {
                if let Ok(p_str) = std::env::var("IP2ASN_CONFIG_PATH") {
                    std::path::PathBuf::from(p_str)
                } else if let Some(home_dir) = home::home_dir() {
                    home_dir.join(".config/ip2asn/config.toml")
                } else {
                    return Ok(Self::default());
                }
            }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_load_valid_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "auto_update = true").unwrap();
        let config = Config::load(Some(file.path())).unwrap();
        assert!(config.auto_update);
    }

    #[test]
    fn test_load_malformed_config() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "auto_update = not-a-boolean").unwrap();
        let result = Config::load(Some(file.path()));
        assert!(matches!(result, Err(CliError::Config(_))));
    }

    #[test]
    fn test_load_missing_config_file() {
        let config = Config::load(Some(std::path::Path::new(
            "/tmp/this/path/should/not/exist.toml",
        )))
        .unwrap();
        // Should return default config
        assert!(!config.auto_update);
    }

    #[test]
    fn test_load_no_config_path_env_var() {
        // Set a temporary home directory to ensure no real config is found.
        let temp_dir = tempfile::tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let config = Config::load(None).unwrap();
        // Should return default config because the temp home is empty.
        assert!(!config.auto_update);

        std::env::remove_var("HOME");
    }
}
