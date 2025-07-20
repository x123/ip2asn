//! This module contains the primary data structures for the `ip2asn` crate,
//! with a focus on memory efficiency and query performance.

/// Represents a single, optimized record for the ASN lookup table.
///
/// This struct is designed to be as small as possible to minimize the memory
/// footprint of the final `IpAsnMap`.
///
/// - `asn`: The Autonomous System Number.
/// - `country_code`: A 2-byte array representing the ISO 3166-1 alpha-2 country code.
/// - `organization_idx`: An index into a string interning table, pointing to the
///   full organization name. This avoids storing duplicate strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsnRecord {
    /// The Autonomous System Number.
    pub asn: u32,
    /// A 2-byte array representing the ISO 3166-1 alpha-2 country code.
    pub country_code: [u8; 2],
    /// An index into a string interning table for the organization name.
    pub organization_idx: u32,
}
