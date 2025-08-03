### Project Blueprint & Iterative Breakdown

The development plan is structured in five phases, starting from the innermost
algorithmic core and progressively building outwards to I/O and features. Each
chunk within a phase represents a small, testable unit of work.

---

### **Phase 9: CLI Foundation & Core Lookup** 🏗️

**Goal:** Establish the `ip2asn-cli` crate within a Cargo workspace and implement the foundational lookup logic, reading IPs from command-line arguments and loading the dataset from an explicit file path.

*   **[x] Chunk 9.1: Workspace & Crate Setup**
    *   **Task:** Modify the root `Cargo.toml` to define a workspace.
    *   **Task:** Create the `ip2asn-cli` directory with a default `main.rs` and `Cargo.toml`.
    *   **Task:** In `ip2asn-cli/Cargo.toml`, add the required dependencies (`ip2asn` with a path, `clap`, etc.) as specified in `docs/spec-ip2asn-cli.md`.
    *   **Task:** Ensure `cargo check --workspace` passes.

*   **[x] Chunk 9.2: Basic Argument Parsing & Lookup**
    *   **Task:** In `ip2asn-cli/src/main.rs`, use `clap` to define the CLI structure. It should accept one or more positional IP address arguments and a `--data <PATH>` option.
    *   **Task:** Implement the core loop: iterate through the IP address arguments, parse them, and perform a lookup using the `ip2asn` library.
    *   **Task:** The `IpAsnMap` should be built from the file provided by the `--data` option.
    *   **Task:** Implement the human-readable, pipe-separated output format and print results to `stdout`.
    *   **TDD: Write a simple integration test** that runs the compiled CLI binary with a test dataset and asserts the correct output for a known IP.

### **Phase 10: Enhanced I/O & Formatting**

**Goal:** Expand the CLI's input/output capabilities to support reading from `stdin` and formatting output as JSON.

*   **[x] Chunk 10.1: Stdin Input Handling**
    *   **Task:** Modify the argument parsing logic. If no positional IP address arguments are provided, the application should switch to reading IPs line-by-line from `stdin`.
    *   **Task:** Refactor the core lookup loop to handle either the list of arguments or the lines from `stdin` as its input.
    *   **TDD: Update the integration test** to cover piping input to the CLI and asserting the correct output.

*   **[x] Chunk 10.2: JSON Output**
    *   **Task:** Add the `--json` / `-j` flag to the `clap` CLI definition.
    *   **Task:** Create a new output module or function that formats the `AsnInfo` struct into the specified JSON structure. The `serde` and `serde_json` crates will be required.
    *   **Task:** In the main loop, check if the `--json` flag is present and call the appropriate formatter.
    *   **TDD: Add a new integration test** that runs the CLI with the `--json` flag and validates the output is correct JSON.

### **Phase 11: Data Management & Caching**

**Goal:** Implement the data download and caching mechanism to make the tool self-sufficient.

*   **[x] Chunk 11.1: The `update` Subcommand**
    *   **Task:** Add an `update` subcommand to the `clap` definition.
    *   **Task:** Implement the handler for the `update` command. It should use the `home` crate to find the user's cache directory (`~/.cache/ip2asn`).
    *   **Task:** Use `reqwest` to download the dataset from the URL specified in the spec.
    *   **Task:** Use `indicatif` to display a progress bar during the download.
    *   **Task:** Save the downloaded file to `{USER_CACHE_DIR}/ip2asn/data.tsv.gz`.

*   **[x] Chunk 11.2: Default Cache Loading**
    *   **Task:** Modify the main lookup logic. If the `--data` flag is *not* provided, the tool should now attempt to load the dataset from the default cache location.
    *   **Task:** Implement error handling: if the cache file does not exist, print a helpful error message instructing the user to run `ip2asn-cli update`.

### **Phase 12: Automation & Configuration**

**Goal:** Add the final layer of automation, allowing the tool to check for updates automatically based on a user configuration file.

*   **[x] Chunk 12.1: Configuration File Support**
    *   **Task:** Implement logic to find and parse the `config.toml` file from the user's config directory (`~/.config/ip2asn/config.toml`) using the `home` crate.
    *   **Task:** Define a simple config struct and use a TOML parser (like the `toml` crate) to deserialize the file. If the file doesn't exist, use default values (i.e., `auto_update = false`).

*   **[x] Chunk 12.2: Automatic Update Logic**
    *   **Task:** In the main lookup function, before loading the map from the cache, check if `auto_update` is enabled in the config.
    *   **Task:** If enabled, check the `mtime` (last modified time) of the cached data file. If it's recent (e.g., less than 24 hours old), do nothing.
    *   **Task:** If the file is old, perform an HTTP `HEAD` request to the data URL. Compare `ETag` or `Last-Modified` headers with stored values to see if a new file is available.
    *   **Task:** If the remote file is newer, trigger the same download logic from the `update` subcommand to refresh the cache before proceeding with the lookup.

### **Phase 13: Test-Driven Refactoring & Hardening**

**Goal:** Enhance the robustness, maintainability, and testability of the `ip2asn-cli` tool by implementing a comprehensive test suite first, followed by targeted code refactoring and tooling improvements.

*   **[x] Chunk 13.1: Expand Test Coverage (Write Failing Tests)**
    *   **TDD: Write a failing integration test for the `update` subcommand.** This test will directly invoke `ip2asn-cli update`, use `mockito` to mock the server response, and assert that the dataset is downloaded to the correct cache location.
    *   **TDD: Write failing unit tests for `config.rs`.** These tests will cover loading a valid config, handling a malformed config file, and correctly applying default values when the file is missing.
    *   **TDD: Write failing integration tests for specific error conditions.** Use `mockito` to simulate network failures (e.g., 500 errors) and assert that the CLI returns the appropriate error messages. Add a test for invalid `stdin` to ensure graceful error handling.

*   **[x] Chunk 13.2: Code Refactoring & Error Handling Improvements**
    *   **Task:** Refactor the `CliError::Update` enum variant in `error.rs` to wrap `reqwest::Error` directly, preserving error context. Update the `From` and `Display` implementations accordingly.
    *   **Task:** Refactor `run_lookup` and `run_update` to use the `home` crate for locating the cache and config directories, removing any redundant path construction logic.
    *   **Task:** Simplify the JSON serialization logic in the `perform_lookup` function to remove redundant code.
    *   **Verification:** All tests from Chunk 13.1 should now pass.

*   **[x] Chunk 13.3: Test Suite & Tooling Enhancements**
    *   **Task:** Refactor the test setup in `tests/cli.rs` to use `rstest` fixtures, abstracting away the boilerplate `ENV_MUTEX` and `TestEnv` instantiation.
    *   **Task:** Add a new `coverage` recipe to the `justfile` that uses `cargo-llvm-cov` to generate an LCOV report.
    *   **Task:** Add a `coverage-html` recipe to the `justfile` to generate a user-friendly HTML report from the LCOV data.

### **Phase 14: Comprehensive Test Coverage**

**Goal:** Systematically improve test coverage for both the `ip2asn-cli` and `ip2asn` crates by implementing the test cases outlined in `docs/spec-test-coverage.md`. This will be done file-by-file to ensure methodical progress and maintain a working state at each step.

*   **[x] Chunk 14.1: `ip2asn-cli` Error Handling Tests**
    *   **File:** `ip2asn-cli/src/error.rs`
    *   **TDD:** Write unit tests for the `fmt::Display` implementation of `CliError`, covering each variant.
    *   **TDD:** Write unit tests for the `std::error::Error` implementation of `CliError`, verifying that `source()` returns the correct underlying error for wrapped variants.
    *   **TDD:** Write unit tests for each `From` trait implementation, ensuring correct conversion from source errors to `CliError` variants.

*   **[x] Chunk 14.2: `ip2asn-cli` Configuration Tests**
    *   **File:** `ip2asn-cli/src/config.rs`
    *   **TDD:** Write a test to verify that `Config::load` correctly uses the `IP2ASN_CONFIG_PATH` environment variable.
    *   **TDD:** Write a test to ensure `Config::load` returns a default `Config` when no file or environment variable is present.
    *   **TDD:** Write a test that asserts `Config::load` returns a `CliError::Config` when attempting to load a non-existent configuration file.

*   **[ ] Chunk 14.3: `ip2asn-cli` Main Logic Tests**
    *   **File:** `ip2asn-cli/src/main.rs`
    *   **TDD:** Write an integration test for `run_lookup` where the home directory cannot be found, asserting a `CliError::NotFound` error.
    *   **TDD:** Write an integration test for `run_lookup` where the dataset file is missing, asserting a `CliError::NotFound` error.
    *   **TDD:** Write integration tests for `check_for_updates` covering:
        *   Forcing an update when the cache file is missing.
        *   Skipping a remote check when the cache is recent.
        *   Handling an invalid `Last-Modified` header from a mocked server.
        *   Gracefully handling a missing `Last-Modified` header.
    *   **TDD:** Write integration tests for `run_update` covering:
        *   Asserting `CliError::NotFound` when the home directory cannot be found.
        *   Simulating a failure to get `content-length` from a mocked server.
    *   **TDD:** Write integration tests for `perform_lookup` covering:
        *   An empty string as input, asserting `Ok(())`.
        *   An invalid IP address, asserting correctly formatted JSON error output.

*   **[ ] Chunk 14.4: Core Library Error & Warning Tests**
    *   **File:** `src/lib.rs`
    *   **TDD:** Write unit tests for `Error::source` for each variant that wraps another error.
    *   **TDD:** Write unit tests for the `fmt::Display` implementations of `Error`, `Warning`, and `ParseErrorKind`, covering all variants.

*   **[ ] Chunk 14.5: Core Library Data Structure Tests**
    *   **File:** `src/lib.rs`
    *   **TDD:** Write a unit test for `IpAsnMap::builder()` to ensure it returns a new `Builder`.
    *   **TDD:** Write unit tests for `AsnInfo` to verify `Ord` and `Hash` implementations.
    *   **TDD:** Write a unit test for the `fmt::Display` implementation of `AsnInfo`.

*   **[ ] Chunk 14.6: Core Library Builder Logic Tests**
    *   **File:** `src/lib.rs`
    *   **TDD:** Write a unit test for the `fmt::Debug` implementation of the `Builder`.
    *   **TDD:** Write a test calling `build()` on a `Builder` with no data source, asserting an `Error::Io` is returned.
    *   **TDD:** Write tests for `build()` warning handling:
        *   Triggering `IpFamilyMismatch` with a warning handler.
        *   Triggering a parse error in non-strict mode without a warning handler.

