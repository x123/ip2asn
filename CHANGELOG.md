# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a
Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.2] - 2025-08-03

### Changed

- The `fmt::Debug` implementation for `Builder` was refactored to be more robust for testing.

## [0.1.1] - 2025-08-02

### Added

- `IpAsnMap::new()` for ergonomic creation of an empty map.
- `IpAsnMap::lookup_owned()` which returns a new owned `AsnInfo` struct, useful
  for async contexts.
- A new optional `serde` feature to allow `AsnInfo` to be serialized and
  deserialized.

### Changed

- `IpAsnMap` now implements the `Default` trait.
- `AsnInfo` now implements `PartialOrd`, `Ord`, and `Hash`. The sort order is
  `asn`, `network`, `country_code`, then `organization`.
