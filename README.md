# Zavigator

Zavigator is a private-by-design Zcash explorer: the backend serves only public chain data, while the browser downloads encrypted transactions, decrypts them locally with a Unified Full Viewing Key (UFVK), and renders a personal finance–like dashboard. The viewing key never leaves the user’s device.

## Quick start

```bash
pnpm install
pnpm dev
```

Visit http://localhost:3000 to confirm the WASM runtime spins up and the UI shell loads.

## WASM workflow

1. Core logic lives in `rust/crates/indexer/indexer-core`.
2. `rust/crates/web/wasm` wraps it with `wasm-bindgen` and exports `decrypt_history`.
3. Run `just wasm-build target=web` whenever Rust changes; the generated package is imported via the `@wasm/rust_wasm` alias.
4. `src/components/wasm-provider.tsx` instantiates the module once per session and exposes `decryptHistory`, readiness state, and `reload` through `useWasm()`.

## Repository map

- `src/` – Next.js app router, providers, and upcoming shielded UX.
- `rust/crates/indexer` – shared parsing/indexer logic.
- `rust/crates/web/wasm` – browser-facing bindings compiled with `wasm-pack`.

Future work focuses on wiring UFVK input, encrypted data retrieval, and decrypted history visualizations that stay entirely client-side.
