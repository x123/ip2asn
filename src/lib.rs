mod range;
mod types;

use std::net::IpAddr;

/// A read-optimized, in-memory map for IP address to ASN lookups.
/// Construction is handled by the `Builder`.
pub struct IpAsnMap { /* private fields */ }

/// A builder for configuring and loading an `IpAsnMap`.
pub struct Builder { /* private fields */ }

/// A lightweight, read-only view into the ASN information for an IP address.
/// This struct is returned by the `lookup` method.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AsnInfoView<'a> {
    pub asn: u32,
    pub country_code: &'a str,
    pub organization: &'a str,
}

/// The primary error type for the crate.
#[derive(Debug)]
pub enum Error {
    /// An error occurred during an I/O operation.
    Io(std::io::Error),

    /// A line in the data source was malformed (only in strict mode).
    Parse {
        line_number: usize,
        line_content: String,
        kind: ParseErrorKind,
    },

    /// An error occurred during serialization or deserialization of the map.
    #[cfg(feature = "serde")]
    Serialization(String),
}

#[derive(Debug)]
pub enum ParseErrorKind {
    /// The line did not have the expected number of columns.
    IncorrectColumnCount { expected: usize, found: usize },
    /// A field could not be parsed as a valid IP address.
    InvalidIpAddress { field: String, value: String },
    /// The ASN field could not be parsed as a valid number.
    InvalidAsnNumber { value: String },
    /// The start IP address was greater than the end IP address.
    InvalidRange { start_ip: IpAddr, end_ip: IpAddr },
    /// The start and end IPs were of different families.
    IpFamilyMismatch,
}

/// A non-fatal warning for a skipped line during parsing.
#[derive(Debug)]
pub enum Warning {
    /// A line in the data source could not be parsed and was skipped.
    Parse {
        line_number: usize,
        line_content: String,
        message: String,
    },
    /// A line contained a start IP and end IP of different families.
    IpFamilyMismatch {
        line_number: usize,
        line_content: String,
    },
}
