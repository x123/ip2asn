use std::fmt;

#[derive(Debug)]
pub enum CliError {
    Io(std::io::Error),
    Config(String),
    Lookup(ip2asn::Error),
    Update(reqwest::Error),
    NotFound(String),
    InvalidInput(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Io(e) => write!(f, "I/O error: {}", e),
            CliError::Config(e) => write!(f, "Configuration error: {}", e),
            CliError::Lookup(e) => write!(f, "Lookup error: {}", e),
            CliError::Update(e) => write!(f, "Update error: {}", e),
            CliError::NotFound(e) => write!(f, "Not found: {}", e),
            CliError::InvalidInput(e) => write!(f, "Invalid input: {}", e),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CliError::Io(e) => Some(e),
            CliError::Lookup(e) => Some(e),
            CliError::Update(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for CliError {
    fn from(err: std::io::Error) -> Self {
        CliError::Io(err)
    }
}

impl From<ip2asn::Error> for CliError {
    fn from(err: ip2asn::Error) -> Self {
        CliError::Lookup(err)
    }
}

impl From<reqwest::Error> for CliError {
    fn from(err: reqwest::Error) -> Self {
        CliError::Update(err)
    }
}

impl From<toml::de::Error> for CliError {
    fn from(err: toml::de::Error) -> Self {
        CliError::Config(err.to_string())
    }
}

impl From<httpdate::Error> for CliError {
    fn from(err: httpdate::Error) -> Self {
        // Wrap httpdate::Error in a generic IO error, as it's a parsing failure
        // during the update process. This avoids adding another error variant.
        CliError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to parse HTTP date: {}", err),
        ))
    }
}
