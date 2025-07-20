# Default task
default:
    @just --list

# Format the code
fmt:
    @cargo fmt

# Lint the code
clippy:
    @cargo clippy -- -D warnings

# cargo deny check
cargo-deny:
    @cargo deny check

# Run all lints
lint:
    @just cargo-deny
    @just clippy
    @just fmt

# run all tests
test *extra_args:
    @just lint
    @cargo nextest run --all-features {{extra_args}}

# run all tests in release mode
test-release *extra_args:
    @cargo nextest run --release --all-features {{extra_args}}

# run all tests with logs enabled
test-debug *extra_args:
    @cargo nextest run --all-features --nocapture {{extra_args}}

# cargo-audit
audit:
    @cargo audit

# install cargo-audit
install-audit:
    @cargo install cargo-audit

# install cargo-textest
install-nextest:
    @cargo install cargo-nextest --locked
