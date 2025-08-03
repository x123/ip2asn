# Journal

## 2025-08-03

### Test Concurrency Failures and Solutions

While implementing comprehensive test coverage, a significant regression occurred where the test suite would pass when run with `--test-threads=1` but fail when run in parallel. This indicated state leakage between tests.

**Challenges & Solutions:**

*   **Problem 1: Integration Test State Leakage (`tests/cli.rs`)**
    *   **Symptom:** `auto_update_tests::test_auto_update_skips_check_for_recent_cache` failed because the auto-update check was being triggered when it shouldn't have been.
    *   **Root Cause:** Tests were using `std::env::set_var` to configure behavior (e.g., `IP2ASN_TESTING`, `IP2ASN_DATA_URL`). This modifies the environment for the *entire* test runner process, causing race conditions where one test's environment bleeds into another's.
    *   **Solution:** Refactored all integration tests to stop using `std::env::set_var`. Instead, environment variables are now passed directly to the child process via `assert_cmd::Command::env()`. This properly isolates the environment for each test run, making them parallel-safe.

*   **Problem 2: Unit Test State Leakage (`src/config.rs`)**
    *   **Symptom:** `config::tests::test_load_no_config_path_env_var` failed its assertion, indicating it was loading a config with `auto_update = true` instead of the default `false`.
    *   **Root Cause:** Similar to the integration tests, the unit tests for `Config::load` were using `std::env::set_var` to manipulate `HOME` and `IP2ASN_CONFIG_PATH`. Since unit tests run in the *same* process, this state leakage was even more direct.
    *   **Solution:** Added the `serial_test` crate as a dev-dependency. The conflicting tests in `config.rs` were marked with the `#[serial]` attribute. This forces `cargo test` to run them sequentially, preventing them from interfering with each other's environment variables while still allowing the rest of the test suite to run in parallel.

These fixes have restored the stability of the test suite and led to the creation of new, explicit guidelines for writing concurrent-safe tests in the project's `.clinerules` and `spec.md`.

## 2025-08-02

### CLI Auto-Update and Configuration

Completed the implementation of the automatic dataset update feature and configuration file support for the `ip2asn-cli` tool.

**Key Developments:**

*   **Configuration:** Added `config.rs` to manage loading a `config.toml` file from `~/.config/ip2asn/config.toml`. This file controls the `auto_update` feature.
*   **Directory Structure:** Switched from `directories::ProjectDirs` to `directories::BaseDirs` to use standard Unix-like paths (`~/.config/ip2asn` for config, `~/.cache/ip2asn` for data).
*   **Auto-Update Logic:** Implemented `check_for_updates()` in `main.rs`.
    *   The check is triggered on `lookup` if no explicit `--data` path is given.
    *   It only runs if `auto_update = true` in the config.
    *   It checks if the cached data is older than 24 hours.
    *   If it is, it performs a `HEAD` request to get the `Last-Modified` header from the remote data URL.
    *   If the remote file is newer, it triggers `run_update()` to download the new dataset.
*   **Output Handling:** Redirected all progress and status messages (`Checking for updates...`, `Downloading...`) to `stderr` to keep `stdout` clean for parsable results.

**Challenges & Solutions:**

*   **Test Flakiness:** The integration tests for the auto-update feature were initially very flaky when run in parallel.
    *   **Problem 1: Shared State:** Tests modified shared environment variables (`IP2ASN_CONFIG_PATH`, `IP2ASN_DATA_URL`), causing race conditions.
        *   **Solution:** Introduced a `lazy_static` `std::sync::Mutex` to serialize the execution of tests that modify the environment, ensuring they run one at a time while allowing other tests to remain parallel.
    *   **Problem 2: Empty `stderr` in `assert_cmd`:** In the `#[tokio::test]` environment, `assert_cmd` failed to capture `stderr` from the child process, even when output was being generated. The process appeared to exit before the `stderr` buffer was flushed.
        *   **Solution 1:** Added explicit `io::stderr().flush()?` calls after every `eprintln!` in the update logic. This fixed the issue for one test but not another.
        *   **Solution 2:** The final fix was discovering an interaction with the `tracing_subscriber::fmt().with_test_writer()`. Removing the subscriber initialization from the failing test allowed `assert_cmd` to correctly capture `stderr`.

This phase was a valuable lesson in the subtleties of testing CLI applications, especially when dealing with I/O, environment variables, and asynchronous test harnesses.