# ip2asn-cli

[![crates.io](https://img.shields.io/crates/v/ip2asn-cli.svg?style=flat-square)](https://crates.io/crates/ip2asn-cli)
[![docs.rs](https://img.shields.io/docsrs/ip2asn-cli?style=flat-square)](https://docs.rs/ip2asn-cli)

A high-performance, command-line tool for quickly looking up IP address to
Autonomous System (AS) information.

-----

## Quick Start

### Basic Lookup

The primary use is to look up an IP address directly.

```sh
# Look up a single IP
$ ip2asn 8.8.8.8
15169 | 8.8.8.8 | 8.8.8.0/24 | GOOGLE | US
```

### Advanced Usage

You can look up multiple IPs or pipe them from standard input.

```sh
# Look up multiple IPs from arguments
$ ip2asn 1.1.1.1 9.9.9.9
13335 | 1.1.1.1 | 1.1.1.0/24 | CLOUDFLARENET | US
19281 | 9.9.9.9 | 9.9.9.0/24 | QUAD9-AS-CH | CH

# Read IPs from stdin
$ echo "208.67.222.222" | ip2asn
22822 | 208.67.222.222 | 208.67.222.0/24 | OPENDNS | US
```

For scripting, JSON output is available.

```sh
$ ip2asn --json 1.1.1.1
{"ip":"1.1.1.1","found":true,"info":{"network":"1.1.1.0/24","asn":13335,"country_code":"US","organization":"CLOUDFLARENET"}}
```

-----

## Installation

Install using `cargo`:

```sh
cargo install ip2asn-cli
```

-----

## Configuration

You can control the tool's behavior by creating a configuration file at `~/.config/ip2asn/config.toml`.

**`config.toml`**
```toml
# Set to false to disable the automatic 24-hour update check.
# The `update` subcommand will still work.
# Defaults to false.
auto_update = true
```

-----

## Dataset Management

`ip2asn-cli` handles the IP-to-ASN dataset for you.

*   **Automatic Download**: On the first run, the tool downloads the latest dataset from [iptoasn.com](https://iptoasn.com).
*   **Caching**: The dataset is cached at `~/.cache/ip2asn/data.tsv.gz`.
*   **Auto-Updates**: If the cached data is over 24 hours old, the tool checks for a newer version and downloads it. This can be disabled in the configuration.
*   **Manual Updates**: You can force an update at any time with the `update` command.
    ```sh
    $ ip2asn update
    ```

-----

## License

This project is licensed under the **MIT License**.
