# ip2asn

A high-performance, memory-efficient Rust crate for mapping IP addresses to
Autonomous System (AS) information with sub-microsecond lookups.

-----

## Features

  * **High Performance**: Sub-microsecond longest-prefix match lookups using a
	PATRICIA trie.
  * **Memory Efficient**: Uses string interning and optimized data structures
	to minimize memory footprint.
  * **Ergonomic API**: A simple Builder pattern for easy configuration and map
	creation.
  * **Gzip Support**: Transparently decompresses `.gz` data sources out of the
	box.
  * **Runtime Agnostic**: A fully synchronous core library that works with any
	async runtime (`tokio`, `smol`, etc.) or in non-async applications.
  * **Optional Serialization**: A `serde` feature allows for pre-compiling the
	map to a binary file for near-instant startup.

-----

## Quick Start

Add `ip2asn` to your `Cargo.toml`:

```toml
[dependencies]
ip2asn = "0.1.0"
```

Then, use the `Builder` to load your data and perform lookups.

```rust
use ip2asn::IpAsnMap;
use std::net::IpAddr;

fn main() -> Result<(), ip2asn::Error> {
    // A small, in-memory TSV data source for the example.
    let data = "31.13.64.0\t31.13.127.255\t32934\tUS\tFACEBOOK-AS";

    // Build the map from a source that implements `io::Read`.
    let map = IpAsnMap::builder()
        .with_source(data.as_bytes())
        .build()?;

    // Perform a lookup.
    let ip: IpAddr = "31.13.100.100".parse().unwrap();
    if let Some(info) = map.lookup(ip) {
        assert_eq!(info.asn, 32934);
        assert_eq!(info.country_code, "US");
        assert_eq!(info.organization, "FACEBOOK-AS");
        println!("{ip} -> AS{}: {} ({})", info.asn, info.organization, info.country_code);
    }

    Ok(())
}
```

-----

## License

This project is licensed under the **MIT License**.
