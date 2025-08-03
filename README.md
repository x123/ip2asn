# ip2asn

[![crates.io](https://img.shields.io/crates/v/ip2asn.svg?style=flat-square)](https://crates.io/crates/ip2asn)
[![docs.rs](https://img.shields.io/docsrs/ip2asn?style=flat-square)](https://docs.rs/ip2asn)

A high-performance, memory-efficient Rust crate for mapping IP addresses to
Autonomous System (AS) information with sub-microsecond lookups.

-----

## Features

* **High Performance**: Sub-microsecond longest-prefix match lookups using a
  PATRICIA trie (`ip_network_table`).
* **Memory Efficient**: Uses string interning and optimized data structures to
  minimize memory footprint.
* **Ergonomic API**: A simple, chainable Builder pattern for easy configuration
  and map creation.
* **Flexible Data Sources**: Load data from any `std::io::Read` source (files,
  in-memory buffers, etc.).
* **Gzip Support**: Transparently decompresses `.gz` data sources out of the
  box.
* **Remote Fetching**: An optional `fetch` feature allows building the map
  directly from a URL.
* **Async-Friendly Lookups**: An owned `AsnInfo` struct is available via
  `lookup_owned()` for safe, lifetime-free use in async or threaded contexts.
* **Serde Support**: An optional `serde` feature allows `AsnInfo` to be
  serialized and deserialized.
* **Robust Error Handling**: Supports a `strict` mode to fail on any parsing
  error and a flexible `on_warning` callback for custom logging.
* **Runtime Agnostic**: A fully synchronous core library that works with any
  async runtime (`tokio`, `smol`, etc.) or in non-async applications.

-----

## Quick Start

Add `ip2asn` to your `Cargo.toml`:

```toml
[dependencies]
ip2asn = "0.1.2"
```

Then, use the `Builder` to load your data and perform lookups.

```rust
use ip2asn::{Builder, IpAsnMap};
use std::net::IpAddr;

fn main() -> Result<(), ip2asn::Error> {
    // A small, in-memory TSV data source for the example.
    let data = "31.13.64.0\t31.13.127.255\t32934\tUS\tFACEBOOK-AS";

    // Build the map from a source that implements `io::Read`.
    let map = Builder::new()
        .with_source(data.as_bytes())?
        .build()?;

    // Perform a lookup.
    let ip: IpAddr = "31.13.100.100".parse().unwrap();
    if let Some(info) = map.lookup(ip) {
        assert_eq!(info.network, "31.13.64.0/18".parse().unwrap());
        assert_eq!(info.asn, 32934);
        assert_eq!(info.country_code, "US");
        assert_eq!(info.organization, "FACEBOOK-AS");
        println!(
            "{} -> AS{} {} ({}) in {}",
            ip, info.asn, info.organization, info.country_code, info.network
        );
    }

    Ok(())
}
```

### Creating an Empty Map

For applications that start with an empty map and load data later, you can use
`IpAsnMap::new()` or `Default::default()`.

```rust
use ip2asn::IpAsnMap;

let empty_map = IpAsnMap::new();
assert!(empty_map.lookup("8.8.8.8".parse().unwrap()).is_none());
```

### Async-Friendly Lookups

The standard `lookup` method returns a view with a lifetime tied to the map.
For async code or situations where you need to store the result, use
`lookup_owned`.

```rust
use ip2asn::{Builder, AsnInfo}; // Note: AsnInfo is the owned struct
use std::net::IpAddr;
# fn main() -> Result<(), ip2asn::Error> {
# let data = "1.0.0.0\t1.0.0.255\t13335\tAU\tCLOUDFLARENET";
# let map = Builder::new().with_source(data.as_bytes())?.build()?;
let ip: IpAddr = "1.0.0.1".parse().unwrap();

// `lookup_owned` returns an `Option<AsnInfo>` with no lifetime.
if let Some(info) = map.lookup_owned(ip) {
    // This `info` object can be sent across threads or stored.
    assert_eq!(info.asn, 13335);
}
# Ok(())
# }
```

### Fetching from a URL

With the `fetch` feature enabled, you can build the map directly from a remote
data source.

```toml
[dependencies]
ip2asn = { version = "0.1.1", features = ["fetch"] }
```

```rust
use ip2asn::{Builder, IpAsnMap};
use std::net::IpAddr;

fn main() -> Result<(), ip2asn::Error> {
    // This example requires a running web server providing the data.
    // You can find a real-world data source from services like `iptoasn.com`.
    let url = "https://iptoasn.com/data/ip2asn-combined.tsv.gz";
    let map = Builder::new().from_url(url)?.build()?;
    
    let ip: IpAddr = "1.0.0.1".parse().unwrap();
    if let Some(info) = map.lookup(ip) {
        assert_eq!(info.asn, 13335); // CLOUDFLARENET
    }
    Ok(())
}
```

### Serialization with Serde

Enable the `serde` feature to serialize and deserialize the `AsnInfo` struct.

```toml
[dependencies]
ip2asn = { version = "0.1.1", features = ["serde"] }
serde_json = "1.0"
```

```rust
# #[cfg(feature = "serde")]
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# use ip2asn::{Builder, AsnInfo};
# let data = "1.0.0.0\t1.0.0.255\t13335\tAU\tCLOUDFLARENET";
# let map = Builder::new().with_source(data.as_bytes())?.build()?;
# let info = map.lookup_owned("1.0.0.1".parse()?).unwrap();
// Serialize the owned `AsnInfo` struct.
let serialized = serde_json::to_string(&info)?;
println!("{}", serialized);

// Deserialize it back.
let deserialized: AsnInfo = serde_json::from_str(&serialized)?;
assert_eq!(info, deserialized);
# Ok(())
# }
# #[cfg(not(feature = "serde"))]
# fn main() {}
```

### Error Handling

By default, the builder skips lines that cannot be parsed. You can enable
`strict` mode to fail fast or use the `on_warning` callback for custom logging.

```rust
use ip2asn::{Builder, Warning};
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

fn main() -> Result<(), ip2asn::Error> {
    let malformed_data = "1.0.0.0\t1.0.0.255\t13335\tAU\tCLOUDFLARENET\nthis is not a valid line";

    // Strict mode will return an error.
    let result = Builder::new().with_source(malformed_data.as_bytes())?.strict().build();
    assert!(result.is_err());

    // The warning callback allows you to inspect and log issues.
    let warning_count = Arc::new(AtomicUsize::new(0));
    let count_clone = warning_count.clone();
    let map = Builder::new()
        .with_source(malformed_data.as_bytes())?
        .on_warning(Box::new(move |warning: Warning| {
            eprintln!("Builder warning: {warning:?}");
            count_clone.fetch_add(1, Ordering::SeqCst);
        }))
        .build()?;

    assert_eq!(warning_count.load(Ordering::SeqCst), 1);
    Ok(())
}
```

-----

## `ip2asn-cli`

This workspace also includes `ip2asn-cli`, a command-line tool that uses the `ip2asn` library to perform lookups from your terminal.

### Installation

You can install the CLI directly from this repository using `cargo`:

```sh
cargo install --path ip2asn-cli
```

### Usage

The primary command is `lookup`, which takes one or more IP addresses as arguments. If no IPs are provided, it reads them from `stdin`.

```sh
# Look up a single IP
$ ip2asn-cli lookup 8.8.8.8
15169 | 8.8.8.8 | 8.8.8.0/24 | GOOGLE | US

# Look up multiple IPs and get JSON output
$ ip2asn-cli lookup --json 1.1.1.1 9.9.9.9
{"ip":"1.1.1.1","found":true,"info":{"network":"1.1.1.0/24","asn":13335,"country_code":"US","organization":"CLOUDFLARENET"}}
{"ip":"9.9.9.9","found":true,"info":{"network":"9.9.9.0/24","asn":19281,"country_code":"CH","organization":"QUAD9-AS-CH"}}

# Read from stdin
$ echo "208.67.222.222" | ip2asn-cli lookup
22822 | 208.67.222.222 | 208.67.222.0/24 | OPENDNS | US
```

### Automatic Dataset Updates

On the first run, and subsequently whenever the cached data is more than 24 hours old, `ip2asn-cli` will automatically check for and download the latest IP-to-ASN dataset from [iptoasn.com](https://iptoasn.com).

*   **Data is cached at:** `~/.cache/ip2asn/data.tsv.gz`
*   Progress messages are printed to `stderr`, so they won't interfere with `stdout` parsing.
*   You can force a download at any time by running `ip2asn-cli update`.

### Configuration

You can configure the tool's behavior by creating a file at `~/.config/ip2asn/config.toml`.

**`config.toml`**
```toml
# Set to false to disable the automatic 24-hour update check.
# The `update` subcommand will still work.
# Defaults to false.
auto_update = true
```

-----

## License

This project is licensed under the **MIT License**.
