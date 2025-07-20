mod interner;
pub mod parser;
pub mod range;
pub mod types;

use crate::interner::StringInterner;
use crate::parser::{ParsedLine, parse_line};
use crate::range::range_to_cidrs;
use crate::types::AsnRecord;
use flate2::read::GzDecoder;
use ip_network_table::IpNetworkTable;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::IpAddr;
use std::path::Path;

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

#[derive(Debug, PartialEq, Eq)]
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
    /// The country code was not a 2-character string.
    InvalidCountryCode { value: String },
}

/// A read-optimized, in-memory map for IP address to ASN lookups.
/// Construction is handled by the `Builder`.
pub struct IpAsnMap {
    table: IpNetworkTable<AsnRecord>,
    organizations: Vec<String>,
}

impl fmt::Debug for IpAsnMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IpAsnMap")
            .field("organizations", &self.organizations.len())
            .finish_non_exhaustive()
    }
}

impl IpAsnMap {
    /// Looks up an IP address, returning a view into its ASN information if found.
    pub fn lookup(&self, ip: IpAddr) -> Option<AsnInfoView> {
        self.table.longest_match(ip).map(|(_, record)| {
            let organization = &self.organizations[record.organization_idx as usize];
            AsnInfoView {
                asn: record.asn,
                country_code: std::str::from_utf8(&record.country_code).unwrap_or_default(),
                organization,
            }
        })
    }
}

/// A builder for configuring and loading an `IpAsnMap`.
#[derive(Default)]
pub struct Builder<'a> {
    source: Option<Box<dyn BufRead + 'a>>,
}

impl<'a> Builder<'a> {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the builder to load data from a file path.
    ///
    /// Gzip decompression is handled automatically by inspecting the file's magic bytes.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let file = File::open(path.as_ref()).map_err(Error::Io)?;
        let mut reader = BufReader::new(file);

        // Check for gzip magic number
        let is_gzipped = {
            let header = reader.fill_buf().map_err(Error::Io)?;
            header.starts_with(&[0x1f, 0x8b])
        };

        let source: Box<dyn BufRead + 'a> = if is_gzipped {
            Box::new(BufReader::new(GzDecoder::new(reader)))
        } else {
            Box::new(reader)
        };

        Ok(Self {
            source: Some(source),
        })
    }

    /// Creates a new builder that will read data from the given source.
    pub fn with_source(source: impl BufRead + 'a) -> Self {
        Self {
            source: Some(Box::new(source)),
        }
    }

    /// Builds the `IpAsnMap`, consuming the builder.
    ///
    /// This method reads from the source, parses each line, interns strings,
    /// converts IP ranges to CIDRs, and inserts them into the final lookup table.
    pub fn build(self) -> Result<IpAsnMap, Error> {
        let source = self
            .source
            .ok_or_else(|| Error::Io(std::io::Error::other("No data source provided")))?;

        let mut interner = StringInterner::new();
        let mut table = IpNetworkTable::new();

        for line_result in source.lines() {
            let line = line_result.map_err(Error::Io)?;
            let parsed: ParsedLine = match parse_line(&line) {
                Ok(p) => p,
                Err(_) => continue, // For now, skip errors.
            };

            let org_idx = interner.get_or_intern(parsed.organization);

            let record = AsnRecord {
                asn: parsed.asn,
                country_code: parsed.country_code,
                organization_idx: org_idx,
            };

            for cidr in range_to_cidrs(parsed.start_ip, parsed.end_ip) {
                table.insert(cidr, record);
            }
        }

        let organizations = interner.into_vec();
        Ok(IpAsnMap {
            table,
            organizations,
        })
    }
}

/// A lightweight, read-only view into the ASN information for an IP address.
/// This struct is returned by the `lookup` method.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub struct AsnInfoView<'a> {
    pub asn: u32,
    pub country_code: &'a str,
    pub organization: &'a str,
}
