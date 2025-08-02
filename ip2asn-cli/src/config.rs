use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    #[serde(default)]
    pub auto_update: bool,
}

impl Config {
    pub fn load() -> Self {
        let config_path = if let Ok(path) = std::env::var("IP2ASN_CONFIG_PATH") {
            std::path::PathBuf::from(path)
        } else if let Some(base_dirs) = directories::BaseDirs::new() {
            base_dirs.config_dir().join("ip2asn/config.toml")
        } else {
            return Self { auto_update: false };
        };

        if config_path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&config_path) {
                if let Ok(config) = toml::from_str(&contents) {
                    return config;
                }
            }
        }

        Self { auto_update: false }
    }
}
