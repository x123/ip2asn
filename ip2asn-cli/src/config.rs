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

#[cfg(test)]
mod tests {
    use super::*;
    use lazy_static::lazy_static;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::{tempdir, NamedTempFile};

    lazy_static! {
        static ref CONFIG_TEST_MUTEX: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn test_load_valid_config() {
        let _guard = CONFIG_TEST_MUTEX.lock().unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "auto_update = true").unwrap();
        std::env::set_var("IP2ASN_CONFIG_PATH", file.path());

        let config = Config::load().unwrap();
        assert!(config.auto_update);

        std::env::remove_var("IP2ASN_CONFIG_PATH");
    }

    #[test]
    fn test_load_malformed_config() {
        let _guard = CONFIG_TEST_MUTEX.lock().unwrap();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "auto_update = not-a-boolean").unwrap();
        std::env::set_var("IP2ASN_CONFIG_PATH", file.path());

        let result = Config::load();
        assert!(matches!(result, Err(CliError::Config(_))));

        std::env::remove_var("IP2ASN_CONFIG_PATH");
    }

    #[test]
    fn test_load_missing_config_file() {
        let _guard = CONFIG_TEST_MUTEX.lock().unwrap();
        // Ensure the env var points to a non-existent file
        std::env::set_var("IP2ASN_CONFIG_PATH", "/tmp/this/path/should/not/exist.toml");

        let config = Config::load().unwrap();
        // Should return default config
        assert!(!config.auto_update);

        std::env::remove_var("IP2ASN_CONFIG_PATH");
    }

    #[test]
    fn test_load_no_config_path_env_var() {
        let _guard = CONFIG_TEST_MUTEX.lock().unwrap();
        std::env::remove_var("IP2ASN_CONFIG_PATH");

        // Create a temporary, empty home directory to ensure no config is found.
        let temp_dir = tempdir().unwrap();
        std::env::set_var("HOME", temp_dir.path());

        let config = Config::load().unwrap();
        // Should return default config
        assert!(!config.auto_update);

        std::env::remove_var("HOME");
    }
}
