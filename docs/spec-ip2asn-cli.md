# Technical Specification: `ip2asn-cli`

This document outlines the complete technical specification for the `ip2asn-cli` Rust command-line tool. It is intended to be a developer-ready guide for implementation.

## 1. Vision & Core Concepts

The `ip2asn-cli` tool will provide a fast and ergonomic command-line interface for enriching IP addresses with Autonomous System (AS) information. It will serve as a companion to the `ip2asn` library crate, targeting security researchers, network engineers, and developers who need to perform quick lookups or enrich large datasets of IPs.

## 2. Project & Crate Structure

The project will be structured as a **Cargo Workspace** to keep the core library and the CLI tool decoupled.

*   **`ip2asn` (Library Crate):**
    *   The existing core library. It will have **no CLI-specific dependencies**.
*   **`ip2asn-cli` (Binary Crate):**
    *   A new crate to be created within the workspace (e.g., in an `ip2asn-cli/` directory).
    *   It will contain all CLI logic and its own dependencies.

The root `Cargo.toml` will define the workspace members:
```toml
[workspace]
members = [
    ".",          # Or "ip2asn" if the lib is in a subdirectory
    "ip2asn-cli",
]
```

### 2.1. Workspace Dependency Management

To facilitate seamless development, the `ip2asn-cli` crate will depend on the `ip2asn` library via a **path dependency**.

**In `ip2asn-cli/Cargo.toml`:**
```toml
[dependencies]
ip2asn = { path = "..", version = "0.1.1" } # Adjust path and version as needed
```

**Rationale:**
*   **For Development:** This allows changes in the `ip2asn` library to be immediately available to `ip2asn-cli` without needing to publish an intermediate version.
*   **For Publishing:** When `cargo publish` is run, Cargo automatically rewrites the `path` dependency into a standard versioned dependency on `crates.io`. This is the standard, idiomatic approach for workspace development and publishing.

## 3. CLI Functionality & Usage

The CLI will be invoked as `ip2asn-cli`. It will have two primary modes of operation: direct lookup and a dedicated `update` subcommand.

### 3.1. Lookup Operation (Default)

This is the primary function of the tool.

#### **Input Handling**

The tool must support two methods for receiving IP addresses:

1.  **Command-Line Arguments:** The user can provide one or more IP addresses as positional arguments.
    ```sh
    ip2asn-cli 8.8.8.8 1.1.1.1
    ```
2.  **Standard Input (stdin):** If no IP address arguments are provided, the tool will read IP addresses from `stdin`, one per line.
    ```sh
    cat list_of_ips.txt | ip2asn-cli
    ```

#### **Output Formatting**

1.  **Human-Readable (Default):**
    *   A pipe-delimited (`|`) format.
    *   **Format:** `ASN | IP | Network | Owner | Country`
    *   **Example:** `15169 | 8.8.8.8 | 8.8.8.0/24 | GOOGLE | US`

2.  **JSON (Optional):**
    *   Enabled with the `--json` or `-j` flag.
    *   Each line of output will be a self-contained JSON object.
    *   **Example:** `{"ip": "8.8.8.8", "asn": 15169, "country_code": "US", "owner": "GOOGLE", "network": "8.8.8.0/24"}`

### 3.2. `update` Subcommand

A dedicated subcommand to manage the ASN dataset.

*   **Usage:** `ip2asn-cli update`
*   **Action:** Downloads the latest dataset from the source URL to the cache location, overwriting any existing file. It should display progress.

## 4. Data Management

### 4.1. Data Source

*   **URL:** `https://iptoasn.com/data/ip2asn-combined.tsv.gz`

### 4.2. Data Loading & Caching

*   **Cache Location:** Use the `directories` crate to find the standard user cache directory.
    *   **Path:** `{USER_CACHE_DIR}/ip2asn/data.tsv.gz`
*   **Data Loading:** The tool will load the dataset from this cache location into the `ip2asn::IpAsnMap`.
*   **Override:** A `--data <PATH>` flag must be available to bypass the cache.

### 4.3. Data Freshness & Updates

*   **Manual (Default):** The user is responsible for updating via `ip2asn-cli update`.
*   **Automatic Updates (Opt-in):**
    *   Enabled via a configuration file.
    *   When enabled, the tool checks for a new dataset on every run.
    *   The check must be efficient: check local file age first, then use an HTTP `HEAD` request to compare `ETag` or `Last-Modified` headers before downloading.

## 5. Configuration

*   **Location:** `{USER_CONFIG_DIR}/ip2asn/config.toml` (using the `directories` crate).
*   **Format:** TOML.
*   **Example `config.toml`:**
    ```toml
    # Set to true to enable automatic checks for dataset updates.
    auto_update = true
    ```

## 6. Error Handling

*   **File Not Found:** If no cached data exists, instruct the user to run `ip2asn-cli update`.
*   **Network Errors:** Report network errors clearly.
*   **Invalid Input:** Report invalid IPs on `stderr` and continue processing other inputs.
*   **Lookup Failure:** Produce no output for that IP.

## 7. Dependencies

The `ip2asn-cli` crate will use the following dependencies.

```toml
[dependencies]
# Core library
ip2asn = { path = "..", version = "0.1.1" } # Version should match the lib crate

# CLI argument parsing
clap = { version = "4.5.4", features = ["derive"] }

# HTTP client for downloading data
reqwest = { version = "0.12.2", features = ["blocking"] }

# JSON serialization for output
serde = "1.0.219"
serde_json = "1.0.142"

# Cross-platform directory handling
directories = "6.0.0"

# Progress bar for downloads
indicatif = "0.18.0"
```
