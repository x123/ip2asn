#![deny(missing_docs)]
//! A high-performance, memory-efficient Rust crate for mapping IP addresses to
//! Autonomous System (AS) information with sub-microsecond lookups.
//!
//! The core of the crate is the [`IpAsnMap`], a read-optimized data structure
//! constructed using a flexible [`Builder`]. The builder can ingest data from
//! files, network streams, or in-memory buffers, and can operate in either a
//! strict mode that errors on malformed lines or a resilient mode that skips
//! them. Lookups are performed in sub-microsecond time, returning a lightweight
//! view of the ASN details. The crate provides detailed [`Error`] and [`Warning`]
//! types for robust error handling.
//!
//! All public data structures are marked as `#[non_exhaustive]` to allow for
//! future additions without breaking backward compatibility. This means you must
//! use the `..` syntax in match patterns on enums and struct instantiations.
//!
//! # Quick Start
//!
//! ```rust
//! use ip2asn::{Builder, IpAsnMap};
//! use ip_network::IpNetwork;
//! use std::net::IpAddr;
//!
//! fn main() -> Result<(), ip2asn::Error> {
//!     // A small, in-memory TSV data source for the example.
//!     let data = "31.13.64.0\t31.13.127.255\t32934\tUS\tFACEBOOK-AS";
//!
//!     // Build the map from a source that implements `io::Read`.
//!     let map = Builder::new()
//!         .with_source(data.as_bytes())?
//!         .build()?;
//!
//!     // Perform a lookup.
//!     let ip: IpAddr = "31.13.100.100".parse().unwrap();
//!     if let Some(info) = map.lookup(ip) {
//!         assert_eq!(info.network, "31.13.64.0/18".parse::<IpNetwork>().unwrap());
//!         assert_eq!(info.asn, 32934);
//!         assert_eq!(info.country_code, "US");
//!         assert_eq!(info.organization, "FACEBOOK-AS");
//!         println!(
//!             "{} -> AS{} {} ({}) in {}",
//!             ip, info.asn, info.organization, info.country_code, info.network
//!         );
//!     }
//!
//!     Ok(())
//! }
//! ```
mod interner;
/// Line-by-line parsing logic for IP-to-ASN data.
pub mod parser;
/// IP range to CIDR conversion logic.
pub mod range;
/// Core data structures for ASN records.
pub mod types;

use crate::interner::StringInterner;
use crate::parser::{parse_line, ParsedLine};
use crate::range::range_to_cidrs;
use crate::types::AsnRecord;
use flate2::read::GzDecoder;
use ip_network::IpNetwork;
use ip_network_table::IpNetworkTable;
use std::error::Error as StdError;
use std::fmt;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{BufRead, BufReader};
use std::net::IpAddr;
use std::path::Path;

/// The primary error type for the crate.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An error occurred during an I/O operation.
    Io(std::io::Error),

    /// An error occurred during an HTTP request.
    #[cfg(feature = "fetch")]
    Http(reqwest::Error),

    /// A line in the data source was malformed (only in strict mode).
    Parse {
        /// The 1-based line number where the error occurred.
        line_number: usize,
        /// The content of the line that failed to parse.
        line_content: String,
        /// The specific type of parsing error.
        kind: ParseErrorKind,
    },
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            #[cfg(feature = "fetch")]
            Error::Http(e) => Some(e),
            Error::Parse { .. } => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "I/O error: {e}"),
            #[cfg(feature = "fetch")]
            Error::Http(e) => write!(f, "HTTP error: {e}"),
            Error::Parse {
                line_number,
                line_content,
                kind,
            } => write!(
                f,
                "Parse error on line {line_number}: {kind} in line: \"{line_content}\""
            ),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

#[cfg(feature = "fetch")]
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err)
    }
}

/// A non-fatal warning for a skipped line during parsing.
#[derive(Debug)]
#[non_exhaustive]
pub enum Warning {
    /// A line in the data source could not be parsed and was skipped.
    Parse {
        /// The 1-based line number where the warning occurred.
        line_number: usize,
        /// The content of the line that was skipped.
        line_content: String,
        /// A message describing the parse error.
        message: String,
    },
    /// A line contained a start IP and end IP of different families.
    IpFamilyMismatch {
        /// The 1-based line number where the warning occurred.
        line_number: usize,
        /// The content of the line that was skipped.
        line_content: String,
    },
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Warning::Parse {
                line_number,
                line_content,
                message,
            } => write!(
                f,
                "Parse warning on line {line_number}: {message} in line: \"{line_content}\""
            ),
            Warning::IpFamilyMismatch {
                line_number,
                line_content,
            } => write!(
                f,
                "IP family mismatch on line {line_number}: \"{line_content}\""
            ),
        }
    }
}

/// The specific kind of error that occurred during line parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ParseErrorKind {
    /// The line did not have the expected number of columns.
    IncorrectColumnCount {
        /// The expected number of columns.
        expected: usize,
        /// The actual number of columns found.
        found: usize,
    },
    /// A field could not be parsed as a valid IP address.
    InvalidIpAddress {
        /// The name of the field that failed parsing (e.g., "start_ip").
        field: String,
        /// The value that could not be parsed.
        value: String,
    },
    /// The ASN field could not be parsed as a valid number.
    InvalidAsnNumber {
        /// The value that could not be parsed as an ASN.
        value: String,
    },
    /// The start IP address was greater than the end IP address.
    InvalidRange {
        /// The start IP of the invalid range.
        start_ip: IpAddr,
        /// The end IP of the invalid range.
        end_ip: IpAddr,
    },
    /// The start and end IPs were of different families.
    IpFamilyMismatch,
    /// The country code was not a 2-character string.
    InvalidCountryCode {
        /// The value that could not be parsed as a country code.
        value: String,
    },
}

impl fmt::Display for ParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseErrorKind::IncorrectColumnCount { expected, found } => {
                write!(f, "expected {expected} columns, but found {found}")
            }
            ParseErrorKind::InvalidIpAddress { field, value } => {
                write!(f, "invalid IP address for field `{field}`: {value}")
            }
            ParseErrorKind::InvalidAsnNumber { value } => {
                write!(f, "invalid ASN: {value}")
            }
            ParseErrorKind::InvalidRange { start_ip, end_ip } => {
                write!(f, "start IP {start_ip} is greater than end IP {end_ip}")
            }
            ParseErrorKind::IpFamilyMismatch => {
                write!(f, "start and end IPs are of different families")
            }
            ParseErrorKind::InvalidCountryCode { value } => {
                write!(f, "invalid country code: {value}")
            }
        }
    }
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

impl Default for IpAsnMap {
    /// Creates a new, empty `IpAsnMap`.
    fn default() -> Self {
        Self {
            table: IpNetworkTable::new(),
            organizations: Vec::new(),
        }
    }
}

impl IpAsnMap {
    /// Creates a new, empty `IpAsnMap`.
    ///
    /// This is a convenience method equivalent to `IpAsnMap::default()`.
    ///
    /// # Example
    ///
    /// ```
    /// use ip2asn::IpAsnMap;
    ///
    /// let map = IpAsnMap::new();
    /// assert!(map.lookup("1.1.1.1".parse().unwrap()).is_none());
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a new `Builder` for constructing an `IpAsnMap`.
    ///
    /// This is a convenience method equivalent to `Builder::new()`.
    pub fn builder() -> Builder<'static> {
        Builder::new()
    }

    /// Looks up an IP address, returning a view into its ASN information if found.
    ///
    /// The lookup is a longest-prefix match, ensuring the most specific
    /// network range is returned. The returned `AsnInfoView` includes the
    /// matching network block itself.
    pub fn lookup(&self, ip: IpAddr) -> Option<AsnInfoView> {
        self.table.longest_match(ip).map(|(network, record)| {
            let organization = &self.organizations[record.organization_idx as usize];
            AsnInfoView {
                network,
                asn: record.asn,
                country_code: std::str::from_utf8(&record.country_code).unwrap_or_default(),
                organization,
            }
        })
    }

    /// Looks up an IP address, returning an owned `AsnInfo` struct if found.
    ///
    /// This method is an alternative to `lookup` that returns an owned `AsnInfo`
    /// struct rather than a view. This is useful in async contexts or when the
    /// result needs to be stored or sent across threads.
    ///
    /// # Example
    ///
    /// ```
    /// # use ip2asn::{Builder, IpAsnMap};
    /// # use std::net::IpAddr;
    /// #
    /// # fn main() -> Result<(), ip2asn::Error> {
    /// # let data = "1.0.0.0\t1.0.0.255\t13335\tAU\tCLOUDFLARENET";
    /// # let map = Builder::new().with_source(data.as_bytes())?.build()?;
    /// let ip: IpAddr = "1.0.0.1".parse().unwrap();
    ///
    /// if let Some(info) = map.lookup_owned(ip) {
    ///     // The `info` object is owned and can be stored or sent across threads.
    ///     assert_eq!(info.asn, 13335);
    ///     println!("Owned info: {}", info);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn lookup_owned(&self, ip: IpAddr) -> Option<AsnInfo> {
        self.lookup(ip).map(AsnInfo::from)
    }
}

/// An owned struct containing ASN information for an IP address.
///
/// This struct is returned by the `lookup_owned` method and is useful when
/// the information needs to be stored or moved, as it does not contain any
/// lifetimes.
///
/// It implements common traits like `Clone`, `Eq`, `Ord`, and `Hash`, and can
/// be serialized with `serde` if the `serde` feature is enabled.
///
/// # Example
///
/// ```
/// # use ip2asn::{AsnInfo, Builder};
/// # use ip_network::IpNetwork;
/// # use std::net::IpAddr;
/// #
/// # fn main() -> Result<(), ip2asn::Error> {
/// # let data = "1.0.0.0\t1.0.0.255\t13335\tAU\tCLOUDFLARENET";
/// # let map = Builder::new().with_source(data.as_bytes())?.build()?;
/// let ip: IpAddr = "1.0.0.1".parse().unwrap();
/// let owned_info = map.lookup_owned(ip).unwrap();
///
/// // You can clone it, hash it, sort it, etc.
/// let cloned_info = owned_info.clone();
/// assert_eq!(owned_info, cloned_info);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct AsnInfo {
    /// The matching IP network block for the looked-up address.
    pub network: IpNetwork,
    /// The Autonomous System Number (ASN).
    pub asn: u32,
    /// The two-letter ISO 3166-1 alpha-2 country code.
    pub country_code: String,
    /// The common name of the organization that owns the IP range.
    pub organization: String,
}

impl PartialEq for AsnInfo {
    fn eq(&self, other: &Self) -> bool {
        self.network == other.network
            && self.asn == other.asn
            && self.country_code == other.country_code
            && self.organization == other.organization
    }
}

impl PartialOrd for AsnInfo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AsnInfo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.asn
            .cmp(&other.asn)
            .then_with(|| self.network.cmp(&other.network))
            .then_with(|| self.country_code.cmp(&other.country_code))
            .then_with(|| self.organization.cmp(&other.organization))
    }
}

impl Hash for AsnInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.network.hash(state);
        self.asn.hash(state);
        self.country_code.hash(state);
        self.organization.hash(state);
    }
}

impl fmt::Display for AsnInfo {
    /// Formats the `AsnInfo` for display.
    ///
    /// # Example
    ///
    /// ```
    /// # use ip2asn::{Builder, IpAsnMap};
    /// # use std::net::IpAddr;
    /// #
    /// # fn main() -> Result<(), ip2asn::Error> {
    /// # let data = "1.0.0.0\t1.0.0.255\t13335\tAU\tCLOUDFLARENET";
    /// # let map = Builder::new().with_source(data.as_bytes())?.build()?;
    /// let ip: IpAddr = "1.0.0.1".parse().unwrap();
    /// let info = map.lookup_owned(ip).unwrap();
    ///
    /// let display_str = info.to_string();
    /// assert_eq!(display_str, "AS13335 CLOUDFLARENET (AU) in 1.0.0.0/24");
    /// # Ok(())
    /// # }
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AS{} {} ({}) in {}",
            self.asn, self.organization, self.country_code, self.network
        )
    }
}

impl<'a> From<AsnInfoView<'a>> for AsnInfo {
    fn from(view: AsnInfoView<'a>) -> Self {
        Self {
            network: view.network,
            asn: view.asn,
            country_code: view.country_code.to_string(),
            organization: view.organization.to_string(),
        }
    }
}

/// A builder for configuring and loading an `IpAsnMap`.
#[derive(Default)]
pub struct Builder<'a> {
    source: Option<Box<dyn BufRead + Send + 'a>>,
    strict: bool,
    on_warning: Option<Box<dyn Fn(Warning) + Send + 'a>>,
}

impl<'a> fmt::Debug for Builder<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builder")
            .field("source", &self.source.as_ref().map(|_| "Some(...)"))
            .field("strict", &self.strict)
            .field("on_warning", &self.on_warning.as_ref().map(|_| "Some(...)"))
            .finish()
    }
}

impl<'a> Builder<'a> {
    /// Creates a new builder with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Configures the builder to load data from a file path.
    ///
    /// Gzip decompression is handled automatically by inspecting the file's magic bytes.
    pub fn from_path<P: AsRef<Path>>(mut self, path: P) -> Result<Self, Error> {
        let file = File::open(path.as_ref())?;
        let reader = BufReader::new(file);
        self.source = Some(self.create_source_from_reader(reader)?);
        Ok(self)
    }

    /// Configures the builder to load data from a source implementing `BufRead`.
    ///
    /// This is the most flexible way to provide data, accepting any reader,
    /// such as an in-memory buffer or a network stream.
    ///
    /// Gzip decompression is handled automatically by inspecting the stream's magic bytes.
    pub fn with_source(mut self, source: impl BufRead + Send + 'a) -> Result<Self, Error> {
        self.source = Some(self.create_source_from_reader(source)?);
        Ok(self)
    }

    /// Configures the builder to load data from a URL.
    ///
    /// This method is only available when the `fetch` feature is enabled.
    /// Gzip decompression is handled automatically by inspecting the stream's magic bytes.
    #[cfg(feature = "fetch")]
    pub fn from_url(mut self, url: &str) -> Result<Self, Error> {
        let response = reqwest::blocking::get(url)?;
        let response = response.error_for_status()?;
        let reader = BufReader::new(response);
        self.source = Some(self.create_source_from_reader(reader)?);
        Ok(self)
    }

    /// Enables strict parsing mode.
    ///
    /// If called, `build()` will return an `Err` on the first parse failure.
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Sets a callback function to be invoked for each skipped line in resilient mode.
    pub fn on_warning<F>(mut self, callback: F) -> Self
    where
        F: Fn(Warning) + Send + 'a,
    {
        self.on_warning = Some(Box::new(callback));
        self
    }

    fn create_source_from_reader(
        &self,
        mut reader: impl BufRead + Send + 'a,
    ) -> Result<Box<dyn BufRead + Send + 'a>, Error> {
        let is_gzipped = {
            let header = reader.fill_buf()?;
            header.starts_with(&[0x1f, 0x8b])
        };

        let source: Box<dyn BufRead + Send + 'a> = if is_gzipped {
            Box::new(BufReader::new(GzDecoder::new(reader)))
        } else {
            Box::new(reader)
        };

        Ok(source)
    }

    /// Builds the `IpAsnMap`, consuming the builder.
    ///
    /// This method reads from the source, parses each line, interns strings,
    /// converts IP ranges to CIDRs, and inserts them into the final lookup table.
    pub fn build(self) -> Result<IpAsnMap, Error> {
        let source = self.source.ok_or_else(|| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No data source provided",
            ))
        })?;

        let mut interner = StringInterner::new();
        let mut table = IpNetworkTable::new();

        for (i, line_result) in source.lines().enumerate() {
            let line_number = i + 1;
            let line = line_result?;
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let parsed: ParsedLine = match parse_line(&line) {
                Ok(p) => p,
                Err(kind) => {
                    if self.strict {
                        return Err(Error::Parse {
                            line_number,
                            line_content: line,
                            kind,
                        });
                    } else if let Some(callback) = &self.on_warning {
                        let warning = if kind == ParseErrorKind::IpFamilyMismatch {
                            Warning::IpFamilyMismatch {
                                line_number,
                                line_content: line,
                            }
                        } else {
                            Warning::Parse {
                                line_number,
                                line_content: line,
                                message: format!("{kind:?}"),
                            }
                        };
                        callback(warning);
                    }
                    continue;
                }
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
#[non_exhaustive]
pub struct AsnInfoView<'a> {
    /// The matching IP network block for the looked-up address.
    pub network: IpNetwork,
    /// The Autonomous System Number (ASN).
    pub asn: u32,
    /// The two-letter ISO 3166-1 alpha-2 country code.
    pub country_code: &'a str,
    /// The common name of the organization that owns the IP range.
    pub organization: &'a str,
}
