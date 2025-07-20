# ip2asn

[![crates.io](https://img.shields.io/crates/v/ip2asn.svg?style=flat-square)](https://crates.io/crates/ip2asn)
[![docs.rs](https://img.shields.io/docsrs/ip2asn?style=flat-square)](https://docs.rs/ip2asn)

A high-performance, memory-efficient Rust crate for mapping IP addresses to
Autonomous System (AS) information with sub-microsecond lookups.

-----

## Features

  * **High Performance**: Sub-microsecond longest-prefix match lookups using a PATRICIA trie (`ip_network_table`).
  * **Memory Efficient**: Uses string interning and optimized data structures to minimize memory footprint.
  * **Ergonomic API**: A simple, chainable Builder pattern for easy configuration and map creation.
  * **Flexible Data Sources**: Load data from any `std::io::Read` source (files, in-memory buffers, etc.).
  * **Gzip Support**: Transparently decompresses `.gz` data sources out of the box.
  * **Remote Fetching**: An optional `fetch` feature allows building the map directly from a URL.
  * **Robust Error Handling**: Supports a `strict` mode to fail on any parsing error and a flexible `on_warning` callback for custom logging.
  * **Runtime Agnostic**: A fully synchronous core library that works with any async runtime (`tokio`, `smol`, etc.) or in non-async applications.

-----

## Quick Start

Add `ip2asn` to your `Cargo.toml`:

```toml
[dependencies]
ip2asn = "0.1.0"
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

### Fetching from a URL

With the `fetch` feature enabled, you can build the map directly from a remote
data source.

```toml
[dependencies]
ip2asn = { version = "0.1.0", features = ["fetch"] }
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

## License

This project is licensed under the **MIT License**.
