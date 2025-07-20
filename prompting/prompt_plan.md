### Project Blueprint & Iterative Breakdown

The development plan is structured in five phases, starting from the innermost
algorithmic core and progressively building outwards to I/O and features. Each
chunk within a phase represents a small, testable unit of work.

---

### **Phase 1: The Core Algorithm (`range_to_cidrs`)** 🧠

**Goal:** Create a rock-solid, standalone utility for converting IP ranges to CIDR prefixes. This is the most complex algorithmic piece and has no other project dependencies, making it the ideal starting point.

* **[x] Chunk 1.1: Project Scaffolding**
    * **Task:** Initialize the Rust library project (`cargo new --lib ip2asn`).
    * **Task:** Add initial dependencies to `Cargo.toml`: `ip_network` for IP address manipulation.
    * **Task:** Set up the basic module structure in `src/lib.rs` and create a private `range.rs` module for the new logic.

* **[x] Chunk 1.2: Foundational Cases (IPv4)**
    * **TDD: Write a failing test** for the simplest case: a single IP address range (e.g., `10.0.0.5` to `10.0.0.5`), asserting it returns a single `/32` CIDR.
    * **Task:** Write the minimal code in `src/range.rs` to make the single-address test pass.
    * **Task:** Add a failing test for a perfectly aligned CIDR block (e.g., `192.168.0.0` to `192.168.0.255`), asserting it returns a single `/24`.
    * **Task:** Implement the core conversion algorithm to handle aligned blocks.

* **[x] Chunk 1.3: Complex & Generic Cases**
    * **TDD: Write a failing test** for a non-aligned range that requires multiple CIDRs (e.g., `10.0.0.1` to `10.0.0.10`).
    * **Task:** Refine the algorithm to correctly handle arbitrary start and end boundaries.
    * **Task:** Add failing tests for equivalent IPv6 ranges (single address, aligned, non-aligned).
    * **Task:** Make the `range_to_cidrs` function generic over `IpAddr` to work for both IPv4 and IPv6.
    * **Task:** Add failing tests for edge cases, like a range spanning the entire address space or invalid input where `start > end`.
    * **Task:** Finalize the algorithm and its error handling.

---

### **Phase 2: Data Structures & Parsing Logic** 📝

**Goal:** Define all data structures and create a parser that can transform a single line of text into a structured internal record.

* **[x] Chunk 2.1: Define All Data Types**
    * **Task:** In `src/lib.rs`, define the public API skeletons for `IpAsnMap`, `Builder`, `AsnInfoView`, `Error`, and `Warning` (with its `ParseErrorKind`).
    * **Task:** Create a new `src/types.rs` module and declare it in `src/lib.rs`.
    * **Task:** In `src/types.rs`, define the *internal*, optimized `AsnRecord` struct that will be stored in the lookup table. This struct will hold the ASN, a `[u8; 2]` for the country code, and a `u32` for the interned organization string index.

* **[x] Chunk 2.2: Line Parser**
    * **Task:** Create a new `src/parser.rs` module and declare it in `src/lib.rs`.
    * **Task:** Create a new test file `tests/parser.rs`.
    * **TDD: Write a failing test** in `tests/parser.rs` that passes a single, valid line of TSV data to a new `parser::parse_line` function and asserts a correctly parsed `AsnRecord`.
    * **Task:** Implement the minimal happy-path logic in `parser::parse_line` to make the test pass.
    * **TDD: Write failing tests** for malformed lines (wrong column count, invalid IP/ASN, mismatched IP families), asserting the correct `ParseErrorKind`.
    * **Task:** Implement the error-checking and validation logic in the parser to make the error tests pass.
    * **TDD: Write a failing test** for country code normalization (e.g., a line with `"None"` should result in an `AsnRecord` with `['Z', 'Z']`).
    * **Task:** Implement the country code normalization logic to make the final test pass.

---

Excellent. Here is the detailed plan for Phase 3, formatted for `prompt_plan.md`, based on the "Sequential Assembly Line" approach.

### **Phase 3: Building & Querying the Map** 🗺️

**Goal:** Integrate the parser, `range_to_cidrs` utility, and a new string
interner to construct a fully functional, queryable `IpAsnMap` from a data
source.

*   **[x] Chunk 3.1: String Interner Utility**
    *   **Task:** Create a new `src/interner.rs` module and declare it in `src/lib.rs`.
    *   **Task:** In `src/interner.rs`, define a `StringInterner` struct containing a `HashMap<String, u32>` for lookups and a `Vec<String>` for storage.
    *   **TDD: Write a failing unit test** in `interner.rs`'s `tests` module. The test should:
        *   Create an interner.
        *   Intern the string "Apple Inc." and assert it gets ID 0.
        *   Intern the string "Google LLC" and assert it gets ID 1.
        *   Intern "Apple Inc." again and assert it gets ID 0.
        *   Assert that retrieving ID 1 returns `"Google LLC"`.
    *   **Task:** Implement the `get_or_intern(&mut self, s: &str) -> u32` and `get(&self, id: u32) -> &str` methods on `StringInterner` to make the tests pass.

*   **[x] Chunk 3.2: Builder & Lookup Integration**
    *   **Task:** Add the `ip_network_table` crate as a dependency in `Cargo.toml`.
    *   **Task:** Create a new `tests/integration.rs` file.
    *   **TDD: Write a failing integration test** in `tests/integration.rs`. This test will be the primary driver for the integration work. It should:
        1.  Define a multi-line string of test TSV data.
        2.  Pass the data to `Builder::with_source()`.
        3.  Call `.build()` and assert the `Result` is `Ok`.
        4.  Call `.lookup()` on the resulting `IpAsnMap` with an IP known to be in the first range and assert it fails (as `lookup` is not yet implemented).
    *   **Task:** Implement the `IpAsnMap::lookup` method. It should query the internal `IpNetworkTable` and, upon finding a record, use the `organization_idx` to retrieve the organization name from the interner's storage and construct the `AsnInfoView`.
    *   **Task:** Implement the `Builder::build` method. This is the core assembly logic:
        1.  Initialize a `StringInterner` and an `IpNetworkTable`.
        2.  Create a `BufReader` for the source.
        3.  Loop through each line of the source data.
        4.  For each valid line, call `parser::parse_line`.
        5.  Use the interner to get an ID for the organization string.
        6.  Create an `AsnRecord` with the interned ID.
        7.  Call `range_to_cidrs` with the start and end IPs.
        8.  Iterate through the resulting CIDRs and insert each one into the `IpNetworkTable` with the `AsnRecord`.
        9.  Store the final `IpNetworkTable` and the interner's `Vec<String>` in the `IpAsnMap`.
    *   **Task:** Refine the integration test. Add assertions for:
        *   An IP address that falls in the middle of a range.
        *   An IP address at the exact start/end of a range.
        *   An IP that is *not* in any range, asserting the lookup returns `None`.
        *   An IP that maps to an organization that appeared multiple times in the source data, ensuring the `AsnInfoView` is correct.
    *   **Task:** Ensure all tests pass.

---
Excellent. Here is the detailed plan for Phase 4, formatted for `prompt_plan.md`, based on our discussion of Approach 1.

---

### **Phase 4: I/O, Features, and Ergonomics** 🔌

**Goal:** Add support for file/network I/O and the optional `serde` and `fetch`
features, making the library robust and easy to use in production environments.
This phase implements the I/O methods on the `Builder` and serialization
methods on the `IpAsnMap` as defined in the specification.

*   **[x] Chunk 4.1: File I/O & Gzip Decompression**
    *   **Task:** Add `flate2` as a dependency.
    *   **Task:** Create two test data files in `testdata/`: one plain text (`small.tsv`) and an identical one that is gzipped (`small.tsv.gz`).
    *   **TDD: Write a failing integration test.** The test should first build a map from the plain text file using a new `Builder::from_path()` method and assert it succeeds. Then, add a second assertion that building from the gzipped file fails.
    *   **Task:** Implement `Builder::from_path(path: P) where P: AsRef<Path>`. This method will:
        1.  Open the file.
        2.  Read the first two bytes to check for the gzip magic number (`[0x1f, 0x8b]`).
        3.  If the magic number is present, wrap the file reader in a `flate2::read::GzDecoder`.
        4.  Wrap the resulting reader in a `BufReader` and store it in the builder's `source` field.
    *   **Task:** Update the `Error` enum with an `Io(std::io::Error)` variant and propagate any file-related errors.
    *   **Task:** Ensure both tests (plain and gzipped) pass, demonstrating transparent decompression.

*   **[ ] Chunk 4.2: `fetch` Feature & Network I/O**
    *   **Task:** Add `reqwest` (with `blocking` feature) and `wiremock` (as a `dev-dependency`) to `Cargo.toml`.
    *   **TDD: Write a failing integration test, gated by `#[cfg(feature = "fetch")]`.** The test should:
        1.  Start a `wiremock::MockServer`.
        2.  Set up a mock to respond to a GET request with the contents of the gzipped test data file (`small.tsv.gz`) and a `Content-Type: application/gzip` header.
        3.  Call a new `Builder::from_url(mock_server.uri())` method and assert that the build fails.
    *   **Task:** Implement `Builder::from_url(url: &str)`, gated by the `fetch` feature. This method will:
        1.  Use `reqwest::blocking::get()` to fetch the URL.
        2.  Handle potential HTTP errors, wrapping them in a new `Error::Http` variant.
        3.  The response body from `reqwest` is a reader. Buffer it, then apply the same gzip magic-byte detection logic from Chunk 4.1 to transparently decompress the stream.
    *   **Task:** Ensure the `wiremock` test passes.

*   **[ ] Chunk 4.3: `serde` Feature & Serialization**
    *   **Task:** Add `postcard` and `serde` (with the `derive` feature) to `Cargo.toml`.
    *   **Task:** Add `#[derive(Serialize, Deserialize)]` to `IpAsnMap` and its internal components (`AsnRecord`, etc.). Note that `IpNetworkTable` already supports `serde`.
    *   **TDD: Write a failing integration test, gated by `#[cfg(feature = "serde")]`.** The test should:
        1.  Build an `IpAsnMap` from a test file.
        2.  Call a new `map.to_bytes()` method and assert it succeeds.
        3.  Call a new `IpAsnMap::from_bytes()` with the resulting byte slice and assert it fails.
    *   **Task:** Implement `IpAsnMap::to_bytes(&self) -> Result<Vec<u8>, Error>` using `postcard::to_allocvec`.
    *   **Task:** Implement `IpAsnMap::from_bytes(bytes: &[u8]) -> Result<Self, Error>` using `postcard::from_bytes`.
    *   **Task:** Update the `Error` enum with a `Serialization(String)` variant to wrap errors from `postcard`.
    *   **Task:** Refine the test to assert that a lookup on the deserialized map returns the exact same `AsnInfoView` as a lookup on the original map.

---

### **Phase 5: Final Polish & Release Prep** ✨

**Goal:** Finalize documentation, add benchmarks, and ensure the crate is ready for publishing.

* **[ ] Chunk 5.1: Warning & Strict Mode**
    * **TDD: Write failing tests** for `builder.strict()` and `builder.on_warning()`. Assert that `strict()` mode returns an `Err` on bad data and that resilient mode calls the warning callback.
    * **Task:** Implement the final logic for strict mode and the warning callback hook.

* **[ ] Chunk 5.2: Benchmarking**
    * **Task:** Add `criterion` as a dev-dependency.
    * **Task:** Create a `benches` directory and implement the benchmark suite covering build time and all specified lookup scenarios.

* **[ ] Chunk 5.3: Documentation**
    * **Task:** Write comprehensive crate-level documentation in `src/lib.rs`.
    * **Task:** Add the detailed usage examples for `tokio` and `smol`.
    * **Task:** Run `cargo doc` and ensure all public APIs are documented with clear explanations and examples.

* **[ ] Chunk 5.4: Final Review & Publish**
    * **Task:** Review the entire API against the Rust API Guidelines checklist.
    * **Task:** Do a final `cargo check`, `cargo test`, and `cargo clippy -- -D warnings` run.
    * **Task:** Publish version `0.1.0` to `crates.io`.
