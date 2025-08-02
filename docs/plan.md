### Project Blueprint & Iterative Breakdown

The development plan is structured in five phases, starting from the innermost
algorithmic core and progressively building outwards to I/O and features. Each
chunk within a phase represents a small, testable unit of work.

---

### **Phase 9: CLI Foundation & Core Lookup** 🏗️

**Goal:** Establish the `ip2asn-cli` crate within a Cargo workspace and implement the foundational lookup logic, reading IPs from command-line arguments and loading the dataset from an explicit file path.

*   **[ ] Chunk 9.1: Workspace & Crate Setup**
    *   **Task:** Modify the root `Cargo.toml` to define a workspace.
    *   **Task:** Create the `ip2asn-cli` directory with a default `main.rs` and `Cargo.toml`.
    *   **Task:** In `ip2asn-cli/Cargo.toml`, add the required dependencies (`ip2asn` with a path, `clap`, etc.) as specified in `docs/spec-ip2asn-cli.md`.
    *   **Task:** Ensure `cargo check --workspace` passes.

*   **[ ] Chunk 9.2: Basic Argument Parsing & Lookup**
    *   **Task:** In `ip2asn-cli/src/main.rs`, use `clap` to define the CLI structure. It should accept one or more positional IP address arguments and a `--data <PATH>` option.
    *   **Task:** Implement the core loop: iterate through the IP address arguments, parse them, and perform a lookup using the `ip2asn` library.
    *   **Task:** The `IpAsnMap` should be built from the file provided by the `--data` option.
    *   **Task:** Implement the human-readable, pipe-separated output format and print results to `stdout`.
    *   **TDD: Write a simple integration test** that runs the compiled CLI binary with a test dataset and asserts the correct output for a known IP.

### **Phase 10: Enhanced I/O & Formatting** 🔄

**Goal:** Expand the CLI's input/output capabilities to support reading from `stdin` and formatting output as JSON.

*   **[ ] Chunk 10.1: Stdin Input Handling**
    *   **Task:** Modify the argument parsing logic. If no positional IP address arguments are provided, the application should switch to reading IPs line-by-line from `stdin`.
    *   **Task:** Refactor the core lookup loop to handle either the list of arguments or the lines from `stdin` as its input.
    *   **TDD: Update the integration test** to cover piping input to the CLI and asserting the correct output.

*   **[ ] Chunk 10.2: JSON Output**
    *   **Task:** Add the `--json` / `-j` flag to the `clap` CLI definition.
    *   **Task:** Create a new output module or function that formats the `AsnInfo` struct into the specified JSON structure. The `serde` and `serde_json` crates will be required.
    *   **Task:** In the main loop, check if the `--json` flag is present and call the appropriate formatter.
    *   **TDD: Add a new integration test** that runs the CLI with the `--json` flag and validates the output is correct JSON.

### **Phase 11: Data Management & Caching** 💾

**Goal:** Implement the data download and caching mechanism to make the tool self-sufficient.

*   **[ ] Chunk 11.1: The `update` Subcommand**
    *   **Task:** Add an `update` subcommand to the `clap` definition.
    *   **Task:** Implement the handler for the `update` command. It should use the `directories` crate to find the user's cache directory and create an `ip2asn` subdirectory.
    *   **Task:** Use `reqwest` to download the dataset from the URL specified in the spec.
    *   **Task:** Use `indicatif` to display a progress bar during the download.
    *   **Task:** Save the downloaded file to `{USER_CACHE_DIR}/ip2asn/data.tsv.gz`.

*   **[ ] Chunk 11.2: Default Cache Loading**
    *   **Task:** Modify the main lookup logic. If the `--data` flag is *not* provided, the tool should now attempt to load the dataset from the default cache location.
    *   **Task:** Implement error handling: if the cache file does not exist, print a helpful error message instructing the user to run `ip2asn-cli update`.

### **Phase 12: Automation & Configuration** ⚙️

**Goal:** Add the final layer of automation, allowing the tool to check for updates automatically based on a user configuration file.

*   **[ ] Chunk 12.1: Configuration File Support**
    *   **Task:** Implement logic to find and parse the `config.toml` file from the user's config directory (using the `directories` crate).
    *   **Task:** Define a simple config struct and use a TOML parser (like the `toml` crate) to deserialize the file. If the file doesn't exist, use default values (i.e., `auto_update = false`).

*   **[ ] Chunk 12.2: Automatic Update Logic**
    *   **Task:** In the main lookup function, before loading the map from the cache, check if `auto_update` is enabled in the config.
    *   **Task:** If enabled, check the `mtime` (last modified time) of the cached data file. If it's recent (e.g., less than 24 hours old), do nothing.
    *   **Task:** If the file is old, perform an HTTP `HEAD` request to the data URL. Compare `ETag` or `Last-Modified` headers with stored values to see if a new file is available.
    *   **Task:** If the remote file is newer, trigger the same download logic from the `update` subcommand to refresh the cache before proceeding with the lookup.
