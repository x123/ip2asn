//! Contains the logic for parsing a single line of the `iptoasn.com` TSV data.

use crate::ParseErrorKind;
use std::net::IpAddr;
use std::str::FromStr;

/// A temporary struct holding the successfully parsed fields from a data line.
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedLine<'a> {
    /// The start of the IP address range.
    pub start_ip: IpAddr,
    /// The end of the IP address range.
    pub end_ip: IpAddr,
    /// The Autonomous System Number (ASN).
    pub asn: u32,
    /// The two-letter ISO 3166-1 alpha-2 country code.
    pub country_code: [u8; 2],
    /// The common name of the organization that owns the IP range.
    pub organization: &'a str,
}

/// Parses a single line of tab-separated `ip2asn` data.
///
/// The expected format is: `START_IP\tEND_IP\tASN\tCOUNTRY_CODE\tORGANIZATION`
///
/// # Errors
///
/// Returns a `ParseErrorKind` if the line is malformed, such as having an
/// incorrect number of columns, invalid IP addresses, or an invalid range.
pub fn parse_line(line: &str) -> Result<ParsedLine, ParseErrorKind> {
    const EXPECTED_COLUMNS: usize = 5;
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() != EXPECTED_COLUMNS {
        return Err(ParseErrorKind::IncorrectColumnCount {
            expected: EXPECTED_COLUMNS,
            found: parts.len(),
        });
    }

    let start_ip_str = parts[0];
    let end_ip_str = parts[1];
    let asn_str = parts[2];
    let country_code_str = parts[3];
    let organization = parts[4];

    let start_ip =
        IpAddr::from_str(start_ip_str).map_err(|_| ParseErrorKind::InvalidIpAddress {
            field: "start_ip".to_string(),
            value: start_ip_str.to_string(),
        })?;

    let end_ip = IpAddr::from_str(end_ip_str).map_err(|_| ParseErrorKind::InvalidIpAddress {
        field: "end_ip".to_string(),
        value: end_ip_str.to_string(),
    })?;

    let asn = u32::from_str(asn_str).map_err(|_| ParseErrorKind::InvalidAsnNumber {
        value: asn_str.to_string(),
    })?;

    if start_ip.is_ipv4() != end_ip.is_ipv4() {
        return Err(ParseErrorKind::IpFamilyMismatch);
    }

    if start_ip > end_ip {
        return Err(ParseErrorKind::InvalidRange { start_ip, end_ip });
    }

    let country_code = match country_code_str {
        "None" | "Unknown" | "" => [b'Z'; 2], // Normalize to 'ZZ'
        s if s.len() == 2 => {
            let mut bytes = [0u8; 2];
            bytes.copy_from_slice(s.as_bytes());
            bytes
        }
        _ => {
            return Err(ParseErrorKind::InvalidCountryCode {
                value: country_code_str.to_string(),
            });
        }
    };

    Ok(ParsedLine {
        start_ip,
        end_ip,
        asn,
        country_code,
        organization,
    })
}
