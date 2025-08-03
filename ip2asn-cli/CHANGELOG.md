# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a
Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2025-08-03

- ip2asn-cli specific README.md with examples

## [0.1.0] - 2025-08-03

### Added

- A new `ip2asn-cli` crate for command-line interaction.
- `lookup` subcommand to find ASN information for IP addresses.
- `update` subcommand to download the latest IP-to-ASN dataset.
- Support for reading IP addresses from standard input.
- `--json` flag for machine-readable output.
- Configuration file support at `~/.config/ip2asn/config.toml`.
- Automatic daily checks for dataset updates, which can be disabled via config.
- `--config` and `--cache-dir` flags to override default paths for improved
  testability.
- Comprehensive integration test suite using `assert_cmd` and `mockito`.
- Unit tests for configuration loading and error handling logic.

### Changed

- The binary is named `ip2asn` for a more ergonomic user experience.
- The `lookup` command is now the default, allowing for usage like `ip2asn
  8.8.8.8`.
- All informational output (e.g., update checks, progress bars) is sent to
  `stderr` to keep `stdout` clean.
- The "AS" prefix was removed from the ASN in the default human-readable
  output.
- Replaced `wiremock` with `mockito` for HTTP mocking in tests.
- Migrated tests to use `rstest` for a simpler, fixture-based structure.

### Fixed

- Made the entire test suite parallel-safe by isolating environment variables
  and serializing tests that modify the process environment.
- Improved error handling to be more specific and robust, replacing `Box<dyn
  Error>` with a dedicated `CliError` type.
