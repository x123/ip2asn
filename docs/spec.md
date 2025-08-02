# **Technical Specification: The `ip2asn` Crate**

This document outlines the complete technical specification for the `ip2asn`
Rust crate. It is intended to be a developer-ready guide for implementation.

## **1. Vision & Core Concepts**

  * **Vision**: To provide the Rust ecosystem with a high-performance,
	memory-efficient, and ergonomic library for mapping IP addresses to their
	corresponding Autonomous System (AS) information.
  * **Core Problem**: Application developers need to efficiently enrich IP
	addresses with ASN data from large text files. The library must perform lookups
	in under a microsecond without excessive memory overhead.
  * **Release Versioning**: The initial release will target version `0.1.0`,
	following standard Cargo conventions.

-----

## **2. Public API & Data Structures**

The public API will be designed to be ergonomic, robust, and compliant with the
[Rust API
Guidelines](https://rust-lang.github.io/api-guidelines/checklist.html). All
public items MUST be documented.

### **2.1. Main Structs & Enums**

```rust
// In lib.rs

/// A read-optimized, in-memory map for IP address to ASN lookups.
/// Construction is handled by the `Builder`.
pub struct IpAsnMap { /* private fields */ }

/// A builder for configuring and loading an `IpAsnMap`.
pub struct Builder { /* private fields */ }

/// A lightweight, read-only view into the ASN information for an IP address.
/// This struct is returned by the `lookup` method.
#[derive(Debug, PartialEq, Eq)]
pub struct AsnInfoView<'a> {
    pub network: IpNetwork,
    pub asn: u32,
    pub country_code: &'a str,
    pub organization: &'a str,
}

/// An owned, lifetime-free struct containing ASN information.
/// This struct is returned by the `lookup_owned` method and is useful
/// for async or multi-threaded applications.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AsnInfo {
    pub network: IpNetwork,
    pub asn: u32,
    pub country_code: String,
    pub organization: String,
}

/// The primary error type for the crate.
#[derive(Debug)]
pub enum Error { /* see section 5.2 */ }

/// A non-fatal warning for a skipped line during parsing.
#[derive(Debug)]
pub enum Warning { /* see section 5.3 */ }
```

### **2.2. Core Functionality**

```rust
// In impl IpAsnMap
impl IpAsnMap {
    /// Creates a new, empty `IpAsnMap`.
    /// This is useful for applications that may load data later or start empty.
    pub fn new() -> Self {
        Self::default()
    }

    /// Performs a longest-prefix match for the given IP address.
    ///
    * // Returns `Some(AsnInfoView)` if a matching network is found, otherwise `None`.
    * // The lookup is extremely fast, suitable for high-throughput pipelines.
    pub fn lookup(&self, ip: std::net::IpAddr) -> Option<AsnInfoView> {
        // ...
    }

    /// Performs a lookup and returns an owned `AsnInfo` struct.
    /// This is ideal for async contexts or when the result needs to be stored,
    /// as it avoids the lifetime constraints of `AsnInfoView`.
    pub fn lookup_owned(&self, ip: std::net::IpAddr) -> Option<AsnInfo> {
        self.lookup(ip).map(AsnInfo::from)
    }
}

impl Default for IpAsnMap {
    /// Creates a new, empty `IpAsnMap`.
    fn default() -> Self {
        // ...
    }
}
```

### **2.3. Builder API**

```rust
// In impl Builder
impl Builder {
    /// Creates a new builder with default, resilient settings.
    pub fn new() -> Self { /* ... */ }

    /// Loads data from a file path.
    /// Automatically handles gzip decompression by inspecting the file's magic bytes.
    pub fn with_file(self, path: &str) -> Result<Self, Error> { /* ... */ }

    /// Loads data from a URL. Requires the `fetch` feature.
    /// Automatically handles gzip decompression. Uses a blocking HTTP client.
    #[cfg(feature = "fetch")]
    pub fn with_url(self, url: &str) -> Result<Self, Error> { /* ... */ }

    /// Loads data from any source that implements `std::io::Read`.
    /// This method expects a plain, uncompressed text stream.
    pub fn with_source<R: std::io::Read>(self, reader: R) -> Self { /* ... */ }

    /// Enables strict parsing mode.
    /// If called, `build()` will return an `Err` on the first parse failure.
    pub fn strict(mut self) -> Self { /* ... */ }

    /// Sets a callback function to be invoked for each skipped line in resilient mode.
    pub fn on_warning<F: Fn(Warning)>(mut self, callback: F) -> Self { /* ... */ }

    /// Builds the `IpAsnMap`, consuming the builder.
    /// This is a potentially expensive operation.
    pub fn build(self) -> Result<IpAsnMap, Error> { /* ... */ }
}
```

-----

## **3. Data Handling & Internal Architecture**

### **3.1. Source Data Format**

  * The initial implementation will exclusively parse the tab-separated format provided by `iptoasn.com`.
  * **Format**: `range_start\trange_end\tAS_number\tcountry_code\tAS_description`

### **3.2. Gzip Decompression**

  * The `with_file()` and `with_url()` methods MUST transparently decompress
	gzipped content.
  * **Detection**: Decompression will be triggered by detecting the gzip magic
	number (`[0x1f, 0x8b]`) at the beginning of the stream, not by file extension.
  * **Dependency**: The `flate2` crate is recommended.

### **3.3. Core Lookup Engine**

  * The internal storage engine will be an `ip_network_table::IpNetworkTable`
	(or a similar PATRICIA trie implementation) optimized for longest-prefix
	matching of IP network blocks.
  * A private `range_to_cidrs(start: IpAddr, end: IpAddr) -> Vec<IpNetwork>`
	utility will be implemented to convert start/end ranges into the minimal set of
	covering CIDR prefixes.

### **3.4. Memory & Performance Optimizations**

To meet performance goals, the data stored in the trie will be highly optimized.

  * **Country Code**: The 2-character country code will be stored as a `[u8;
	2]`. During parsing, non-standard values (`None`, `Unknown`, etc.) will be
	normalized to the user-assigned ISO code `ZZ`.
  * **Organization**: The organization description strings will be
	**interned**. A central `Vec<String>` will store each unique organization name
	once. The record in the trie will store a `u32` index pointing to this vector,
	avoiding massive string duplication.

-----

## **4. Error & Warning Handling**

### **4.1. Resilient vs. Strict Mode**

  * **Default (Resilient)**: By default, the `build()` process will skip any
	malformed lines. If an `on_warning` callback is configured, it will be called
	for each skipped line with a `Warning` payload.
  * **Strict Mode**: If `builder.strict()` is called, the `build()` process
	will fail fast, returning an `Error` on the first malformed line encountered.

### **4.2. `Error` Enum**

```rust
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
```

### **4.3. `Warning` Enum**

```rust
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
    // ... other non-fatal warnings as needed ...
}
```

-----

## **5. Cargo Features**

  * `fetch`:
    * Enables the `builder.with_url()` method.
    * Adds a dependency on `reqwest`, using its `blocking` client to ensure
	  the crate remains runtime-agnostic.
  * `serde`:
    * Enables serialization and deserialization for the `AsnInfo` struct.
    * Adds a dependency on `serde` and enables the `serde` feature in the `ip_network` crate.

-----

## **6. Async Compatibility**

  * **Runtime-Agnostic Design**: The core library API is **100% synchronous**.
	It has no dependency on `tokio`, `smol`, or any other async runtime.
  * **Primary Solution**: The `lookup_owned()` method is the primary solution for
    using the map in async contexts. It returns an owned `AsnInfo` struct that is
    `Send + Sync` and has no lifetime constraints, making it safe to store, move
    between threads, or use in async functions without `unsafe` code.
  * **Documentation**: The crate-level documentation MUST provide clear examples
    of using `lookup_owned()` in an async context. It should still mention the
    `spawn_blocking` pattern for the initial, potentially long-running `build()`
    call.

-----

## **7. Performance & Benchmarking**

  * **Goals**:
      * Lookup Speed: `< 500` nanoseconds per lookup.
	  * Memory Usage: A file with \~700,000 ranges should result in an
		in-memory map of approximately 150-200 MB.
  * **Tool**: Benchmarks MUST be implemented using the `criterion` crate.
  * **Benchmark Suite**: The suite MUST include benchmarks for:
    1.  The `build()` time for a large, real-world dataset.
	2.  `lookup()` performance for a random selection of IPv4 and IPv6
		addresses known to be in the dataset.
	3.  `lookup()` performance for IPs known to be unallocated (the "not found"
		case).
	4.  `lookup()` performance for a curated list of edge-case IPs (e.g., the
		first and last address of several network blocks).

-----

## **8. Future Development (Post-v1.0)**

### **8.1. Hot-Reloading**

A future version should include a mechanism for hot-reloading the dataset in a
long-running service.

  * **Change Detection**: Use HTTP `HEAD` requests to check `ETag` or
	`Last-Modified` headers to avoid downloading the full dataset unnecessarily.
  * **Update Mechanism**: Use a "blue-green" strategy. When an update is
	detected, build the new map on a background thread. Once complete, atomically
	swap the new map into service using an `Arc<IpAsnMap>`. This ensures zero
	downtime for lookups.
  * **API**: This could be exposed via a new wrapper struct, e.g.,
	`UpdatingIpAsnMap`.
