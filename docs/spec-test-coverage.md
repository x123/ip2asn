# Test Coverage Improvement Specification for `ip2asn`

## 1. Introduction

This document outlines a plan for improving the test coverage of the `ip2asn` crate. The following sections detail the areas of the codebase that currently lack sufficient test coverage, based on the analysis of the `llvm-cov` reports. For each area, specific test cases are proposed to address the gaps and improve the overall quality and robustness of the crate.

## 2. `ip2asn-cli` Crate

### 2.1. `ip2asn-cli/src/error.rs`

**File Summary:** This file defines the `CliError` enum and its implementations for `fmt::Display`, `std::error::Error`, and various `From` trait implementations.

**Uncovered Areas and Proposed Tests:**

*   **`fmt::Display` for `CliError`** ([`ip2asn-cli/src/error.rs:20-33`](ip2asn-cli/src/error.rs:20))
    *   **Proposed Tests:**
        *   Create a test for each `CliError` variant to ensure that the `fmt::Display` implementation produces the expected output string.

*   **`std::error::Error` for `CliError`** ([`ip2asn-cli/src/error.rs:35-45`](ip2asn-cli/src/error.rs:35))
    *   **Proposed Tests:**
        *   For each `CliError` variant that wraps another error (e.g., `CliError::Io(e)`), create a test to verify that the `source()` method returns the correct underlying error.

*   **`From` Trait Implementations for `CliError`** ([`ip2asn-cli/src/error.rs:47-68`](ip2asn-cli/src/error.rs:47))
    *   **Proposed Tests:**
        *   For each `From` implementation, create a test that converts the source error type into a `CliError` and asserts that the resulting `CliError` variant is correct.

### 2.2. `ip2asn-cli/src/config.rs`

**File Summary:** This file defines the `Config` struct and its `load` method for configuration management.

**Uncovered Areas and Proposed Tests:**

*   **`Config::load` - Environment Variable Handling** ([`ip2asn-cli/src/config.rs:22`](ip2asn-cli/src/config.rs:22))
    *   **Proposed Test:**
        *   Create a test where the `IP2ASN_CONFIG_PATH` environment variable is set to a valid path, and assert that `Config::load` correctly uses that path to load the configuration.

*   **`Config::load` - Default Configuration** ([`ip2asn-cli/src/config.rs:26`](ip2asn-cli/src/config.rs:26))
    *   **Proposed Test:**
        *   Create a test where no configuration file is present and the `IP2ASN_CONFIG_PATH` environment variable is not set, and assert that `Config::load` returns the default `Config` struct.

*   **`Config::load` - File Read Error** ([`ip2asn-cli/src/config.rs:36-41`](ip2asn-cli/src/config.rs:36))
    *   **Proposed Test:**
        *   Create a test that attempts to load a non-existent configuration file and asserts that `Config::load` returns a `CliError::Config` with the expected error message.

### 2.3. `ip2asn-cli/src/main.rs`

**File Summary:** This file contains the main CLI logic, including argument parsing, lookup, and update functionalities.

**Uncovered Areas and Proposed Tests:**

*   **`run_lookup` Function** ([`ip2asn-cli/src/main.rs:187-204`](ip2asn-cli/src/main.rs:187))
    *   **Proposed Tests:**
        *   Create a test where the home directory cannot be determined and assert that `run_lookup` returns a `CliError::NotFound` error.
        *   Create a test where the dataset file is not found and assert that `run_lookup` returns a `CliError::NotFound` error with the appropriate message.

*   **`check_for_updates` Function** ([`ip2asn-cli/src/main.rs:246-290`](ip2asn-cli/src/main.rs:246))
    *   **Proposed Tests:**
        *   Create a test where the cache file does not exist and assert that `check_for_updates` forces an update.
        *   Create a test where the cache file is recent and not in test mode, and assert that the remote check is skipped.
        *   Create a test that simulates an invalid `Last-Modified` header from the server and asserts that the correct `CliError::Io` error is returned.
        *   Create a test where the `Last-Modified` header is missing from the server response and assert that the function handles this case gracefully.

*   **`run_update` Function** ([`ip2asn-cli/src/main.rs:303-341`](ip2asn-cli/src/main.rs:303))
    *   **Proposed Tests:**
        *   Create a test where the home directory cannot be determined and assert that `run_update` returns a `CliError::NotFound` error.
        *   Create a test that simulates a failure to get the content length from the server and asserts that the correct `CliError::Io` error is returned.
        *   Create a test that runs `run_update` without the `IP2ASN_TESTING` environment variable set to ensure the progress bar logic is exercised.

*   **`perform_lookup` Function** ([`ip2asn-cli/src/main.rs:354-360`](ip2asn-cli/src/main.rs:354))
    *   **Proposed Tests:**
        *   Create a test where the input IP address is an empty string and assert that the function returns `Ok(())`.
        *   Create a test with an invalid IP address and assert that the JSON output for an invalid IP is correctly formatted.

## 3. Core Library (`ip2asn`)

### 3.1. `src/lib.rs`

**File Summary:** This is the main library file, defining the core data structures, error types, and the `Builder` for constructing the `IpAsnMap`.

**Uncovered Areas and Proposed Tests:**

*   **Error and Warning Handling**
    *   **`Error::source`** ([`src/lib.rs:95-102`](src/lib.rs:95)):
        *   **Proposed Test:** For each `Error` variant that wraps another error (e.g., `Error::Io`), create a test to verify that the `source()` method returns the correct underlying error.
    *   **`fmt::Display` for `Error`, `Warning`, and `ParseErrorKind`** ([`src/lib.rs:105-121`](src/lib.rs:105), [`src/lib.rs:158-178`](src/lib.rs:158), [`src/lib.rs:219-242`](src/lib.rs:219)):
        *   **Proposed Tests:** Create tests for each variant of these enums to ensure the `fmt::Display` implementation produces the expected string representation.

*   **`IpAsnMap` and `AsnInfo`**
    *   **`IpAsnMap::builder`** ([`src/lib.rs:289-291`](src/lib.rs:289)):
        *   **Proposed Test:** Create a test that calls `IpAsnMap::builder()` and asserts that it returns a new `Builder` instance.
    *   **`PartialOrd`, `Ord`, and `Hash` for `AsnInfo`** ([`src/lib.rs:391-414`](src/lib.rs:391)):
        *   **Proposed Tests:**
            *   Create a test to verify the comparison logic in the `Ord` implementation for `AsnInfo`.
            *   Create a test to verify that the `Hash` implementation for `AsnInfo` produces the expected hash.
    *   **`fmt::Display` for `AsnInfo`** ([`src/lib.rs:436-443`](src/lib.rs:436)):
        *   **Proposed Test:** Create a test to verify that the `fmt::Display` implementation for `AsnInfo` produces the expected string format.

*   **`Builder` Logic**
    *   **`fmt::Debug` for `Builder`** ([`src/lib.rs:464-472`](src/lib.rs:464)):
        *   **Proposed Test:** Create a test to verify that the `fmt::Debug` implementation for the `Builder` produces the expected output.
    *   **`build()` - No Data Source** ([`src/lib.rs:554-559`](src/lib.rs:554)):
        *   **Proposed Test:** Create a test that calls `build()` on a `Builder` without a data source and asserts that it returns the expected `Error::Io` variant.
    *   **`build()` - Warning Handling** ([`src/lib.rs:582-594`](src/lib.rs:582)):
        *   **Proposed Tests:**
            *   Create a test with data that causes an `IpFamilyMismatch` and a warning handler, and assert that the `Warning::IpFamilyMismatch` is correctly generated and passed to the handler.
            *   Create a test that triggers a parse error in non-strict mode without a warning handler to cover the `else` branch.
