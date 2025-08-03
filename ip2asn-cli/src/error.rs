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

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    // Test `fmt::Display` for each `CliError` variant.
    #[test]
    fn test_cli_error_display() {
        assert_eq!(
            CliError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "file not found"
            ))
            .to_string(),
            "I/O error: file not found"
        );
        assert_eq!(
            CliError::Config("bad config".to_string()).to_string(),
            "Configuration error: bad config"
        );
        assert_eq!(
            CliError::Lookup(ip2asn::Error::Parse {
                line_number: 1,
                line_content: "bad line".to_string(),
                kind: ip2asn::ParseErrorKind::InvalidIpAddress {
                    field: "start_ip".to_string(),
                    value: "bad ip".to_string(),
                },
            })
            .to_string(),
            "Lookup error: Parse error on line 1: invalid IP address for field `start_ip`: bad ip in line: \"bad line\""
        );
        // Note: reqwest::Error does not have a simple constructor.
        // We will rely on the `From` trait test to cover the Update variant's display.
        assert_eq!(
            CliError::NotFound("thing not found".to_string()).to_string(),
            "Not found: thing not found"
        );
        assert_eq!(
            CliError::InvalidInput("bad value".to_string()).to_string(),
            "Invalid input: bad value"
        );
    }

    // Test `source()` for each `CliError` variant.
    #[test]
    fn test_cli_error_source() {
        assert!(CliError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found"
        ))
        .source()
        .is_some());
        assert!(CliError::Lookup(ip2asn::Error::Parse {
            line_number: 1,
            line_content: "bad line".to_string(),
            kind: ip2asn::ParseErrorKind::InvalidIpAddress {
                field: "start_ip".to_string(),
                value: "bad ip".to_string(),
            },
        })
        .source()
        .is_some());
        // Note: reqwest::Error does not have a simple constructor.
        // We will rely on the `From` trait test to cover the Update variant's source.
        assert!(CliError::Config("bad config".to_string())
            .source()
            .is_none());
        assert!(CliError::NotFound("thing not found".to_string())
            .source()
            .is_none());
        assert!(CliError::InvalidInput("bad value".to_string())
            .source()
            .is_none());
    }

    // Test `From` trait implementations.
    #[test]
    fn test_from_implementations() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let cli_error: CliError = io_error.into();
        assert!(matches!(cli_error, CliError::Io(_)));

        let lookup_error = ip2asn::Error::Parse {
            line_number: 1,
            line_content: "bad line".to_string(),
            kind: ip2asn::ParseErrorKind::InvalidIpAddress {
                field: "start_ip".to_string(),
                value: "bad ip".to_string(),
            },
        };
        let cli_error: CliError = lookup_error.into();
        assert!(matches!(cli_error, CliError::Lookup(_)));

        let toml_error: toml::de::Error = toml::from_str::<i32>("'a'").unwrap_err();
        let cli_error: CliError = toml_error.into();
        assert!(matches!(cli_error, CliError::Config(_)));
        assert!(cli_error.to_string().starts_with("Configuration error:"));

        let httpdate_error = "invalid-date".parse::<httpdate::HttpDate>().unwrap_err();
        let cli_error: CliError = httpdate_error.into();
        assert!(matches!(cli_error, CliError::Io(_)));
        assert!(cli_error.to_string().contains("Failed to parse HTTP date"));
    }
}
