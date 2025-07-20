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

* **[ ] Chunk 2.2: Line Parser**
    * **Task:** Create a new `src/parser.rs` module and declare it in `src/lib.rs`.
    * **Task:** Create a new test file `tests/parser.rs`.
    * **TDD: Write a failing test** in `tests/parser.rs` that passes a single, valid line of TSV data to a new `parser::parse_line` function and asserts a correctly parsed `AsnRecord`.
    * **Task:** Implement the minimal happy-path logic in `parser::parse_line` to make the test pass.
    * **TDD: Write failing tests** for malformed lines (wrong column count, invalid IP/ASN, mismatched IP families), asserting the correct `ParseErrorKind`.
    * **Task:** Implement the error-checking and validation logic in the parser to make the error tests pass.
    * **TDD: Write a failing test** for country code normalization (e.g., a line with `"None"` should result in an `AsnRecord` with `['Z', 'Z']`).
    * **Task:** Implement the country code normalization logic to make the final test pass.

---

### **Phase 3: Building & Querying the Map** 🗺️

**Goal:** Integrate the parser, `range_to_cidrs` utility, and the lookup table to create a fully functional, in-memory map.

* **[ ] Chunk 3.1: String Interner**
    * **TDD: Write a failing test** for a new, private string interning utility, asserting that it correctly assigns unique and reused IDs for a mix of strings.
    * **Task:** Implement the string interner using a `HashMap` and a `Vec`.

* **[ ] Chunk 3.2: Builder & Lookup Integration**
    * **TDD: Write a failing test** in `tests/integration.rs` that builds a map from an in-memory source and asserts that a lookup for a known IP fails or panics.
    * **Task:** Implement the `build` method, wiring together the line reader, parser, `range_to_cidrs`, interner, and `IpNetworkTable` insertion.
    * **Task:** Implement the `lookup` method to query the table and resolve the internal `AsnRecord` into a public `AsnInfoView`.
    * **Task:** Add assertions to the integration test for IPs that are *not* in the map (expecting `None`) and for edge-of-range IPs to ensure all tests pass.

---

### **Phase 4: I/O, Features, and Ergonomics** 🔌

**Goal:** Add support for file/network I/O and the optional `serde` and `fetch` features.

* **[ ] Chunk 4.1: Gzip Decompression**
    * **TDD: Write a failing test** using two files (one plain, one gzipped). Assert that building from plain text works but building from the gzipped file fails.
    * **Task:** Implement `Builder::with_file`, adding magic byte detection and the `flate2` dependency to handle decompression transparently. Ensure both tests pass.

* **[ ] Chunk 4.2: `fetch` Feature**
    * **TDD: Write a failing test** (gated by `#[cfg(feature = "fetch")]`) that uses a mock HTTP server (`wiremock`) to serve test data and asserts that `Builder::with_url` fails.
    * **Task:** Add the `reqwest` dependency (with `blocking` feature) and implement `with_url`. Ensure the test passes.

* **[ ] Chunk 4.3: `serde` Feature**
    * **TDD: Write a failing test** (gated by `#[cfg(feature = "serde")]`) that builds a map, serializes it, deserializes it, and asserts that a lookup on the new map returns the correct data.
    * **Task:** Add `serde` and `postcard` dependencies, derive traits, and implement the public `serialize` and `deserialize` methods. Ensure the test passes.

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
