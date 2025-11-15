set shell := ["bash", "-lc"]

default:
    @just --list

# Build the WebAssembly crate that feeds the Next.js frontend.
wasm-build target="web":
    cd rust/crates/web/wasm && wasm-pack build --target {{target}}

# Run the Rust unit tests for the wasm crate.
wasm-test:
    cd rust && cargo test -p rust-wasm

# Remove previously generated wasm artifacts.
wasm-clean:
    rm -rf rust/crates/web/wasm/pkg
    rm -rf rust/target

# Install or upgrade wasm-pack so the build recipe can run.
install-wasm-pack:
    cargo install wasm-pack --force

# Run all indexer crate tests to validate backend logic.
indexer-test:
    cd rust && cargo test -p indexer-core

# Boot the indexer service binary.
indexer-run:
    cd rust && cargo run -p indexer-service
