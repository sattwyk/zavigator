set shell := ["bash", "-lc"]

default:
    @just --list

# Build the Rust crate into WebAssembly artifacts under rust-wasm/pkg.
wasm-build target="web":
    cd rust-wasm && wasm-pack build --target {{target}}

# Run the Rust unit tests for the wasm crate.
wasm-test:
    cd rust-wasm && cargo test

# Remove previously generated wasm artifacts.
wasm-clean:
    rm -rf rust-wasm/pkg
    rm -rf rust-wasm/target

# Install or upgrade wasm-pack so the build recipe can run.
install-wasm-pack:
    cargo install wasm-pack --force
