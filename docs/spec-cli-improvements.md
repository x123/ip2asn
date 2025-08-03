# Technical Specification: `ip2asn-cli` Refactoring & Improvement

This document outlines the technical specification for refactoring and improving the `ip2asn-cli` crate. It is based on a thorough code review and is intended to be a developer-ready guide for implementation.

## 1. Vision & Goals

The goal of this initiative is to enhance the robustness, maintainability, and testability of the `ip2asn-cli` tool. This will be achieved by addressing identified technical debt, improving code structure, and increasing test coverage.

## 2. Architecture & Code Quality Refinements

### 2.1. Centralize Cache Path Logic (Anti-DRY)

*   **Problem:** The logic for determining the cache directory path is duplicated in [`run_lookup`](ip2asn-cli/src/main.rs:173) and [`run_update`](ip2asn-cli/src/main.rs:276).
*   **Requirement:**
    1.  Create a new private function, `get_cache_dir() -> Result<PathBuf, CliError>`, within [`main.rs`](ip2asn-cli/src/main.rs).
    2.  This function will use the `home` crate to locate the user's home directory and construct the path to the cache directory (`~/.cache/ip2asn`).
    3.  Both `run_lookup` and `run_update` must be refactored to use this new function.

### 2.2. Simplify JSON Output Logic

*   **Problem:** The `perform_lookup` function in [`main.rs`](ip2asn-cli/src/main.rs:315) has redundant code for serializing JSON output for valid and invalid IPs.
*   **Requirement:**
    1.  Refactor [`perform_lookup`](ip2asn-cli/src/main.rs:315) to construct the [`JsonOutput`](ip2asn-cli/src/main.rs:130) struct based on the lookup result.
    2.  The `serde_json::to_string` call should only happen once at the end of the function, regardless of whether the IP was found or invalid.

## 3. Error Handling Enhancements

*   **Problem:** The `From<reqwest::Error>` implementation in [`error.rs`](ip2asn-cli/src/error.rs:48) converts the rich `reqwest::Error` into a simple `String`, losing valuable context.
*   **Requirement:**
    1.  Modify the [`CliError::Update`](ip2asn-cli/src/error.rs:8) enum variant to wrap the underlying error type directly: `Update(reqwest::Error)`.
    2.  Update the corresponding `From<reqwest::Error>` implementation to use this new variant.
    3.  Adjust the `fmt::Display` implementation for `CliError` to properly format the wrapped error.

## 4. Testing Strategy & Coverage Plan

### 4.1. Test Suite Refactoring

*   **Problem:** The test setup logic (`ENV_MUTEX` lock and `TestEnv` instantiation) is duplicated across most tests in [`tests/cli.rs`](ip2asn-cli/tests/cli.rs).
*   **Requirement:**
    1.  Abstract the repetitive setup into a helper function or a test macro to reduce boilerplate. For example, a function `setup_test_env(auto_update: bool) -> TestEnv` could handle the mutex lock and environment creation.

### 4.2. Test Coverage Expansion

The following gaps in test coverage must be addressed:

1.  **`update` Subcommand:**
    *   **Plan:** Add a new integration test that directly invokes `ip2asn-cli update`. This test should use `mockito` to mock the server response and assert that the data file is successfully downloaded and created in the temporary cache directory.

2.  **Configuration Loading (`config.rs`):**
    *   **Plan:** Create a new unit test module within [`config.rs`](ip2asn-cli/src/config.rs).
    *   Add tests to verify:
        *   Correct loading of a valid `config.toml`.
        *   Graceful handling of a malformed/invalid `config.toml` file (should return a `CliError::Config`).
        *   Correctly falling back to `Config::default()` when the config file does not exist.

3.  **Error Condition Tests:**
    *   **Plan:** Add new integration tests to simulate and verify the following failure scenarios:
        *   **Cache Directory Creation Failure:** Mock a scenario where creating the cache directory fails due to permissions issues (this may require more advanced test setup).
        *   **Network Errors:** Use `mockito` to simulate network failures (e.g., 500 server error, connection timeout) for both `HEAD` and `GET` requests and assert that the correct `CliError::Update` is returned.
        *   **Invalid `stdin`:** Add a test where non-IP strings are piped to `stdin` and assert that they are gracefully handled (reported on `stderr`) without crashing the program.

## 5. `justfile` Enhancements

*   **Problem:** The project lacks an automated way to measure test coverage.
*   **Requirement:**
    1.  Add `cargo-llvm-cov` as a development tool.
    2.  Add a new recipe to the [`justfile`](justfile) for generating coverage reports.

    ```makefile
    # Generate test coverage report
    coverage:
        @cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info

    # Generate and open HTML coverage report
    coverage-html:
        @just coverage
        @genhtml lcov.info --output-directory coverage/
        @echo "HTML report generated in coverage/ directory."
    ```
