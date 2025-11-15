This is a [Next.js](https://nextjs.org) project bootstrapped with [`create-next-app`](https://nextjs.org/docs/app/api-reference/cli/create-next-app).

## Getting Started

First, run the development server:

```bash
npm run dev
# or
yarn dev
# or
pnpm dev
# or
bun dev
```

Open [http://localhost:3000](http://localhost:3000) with your browser to see the result.

You can start editing the page by modifying `app/page.tsx`. The page auto-updates as you edit the file.

This project uses [`next/font`](https://nextjs.org/docs/app/building-your-application/optimizing/fonts) to automatically optimize and load [Geist](https://vercel.com/font), a new font family for Vercel.

## WebAssembly workflow

The frontend consumes the Rust crate in `rust/crates/web/wasm` via the generated `@wasm/rust_wasm` package. To rebuild the bindings after touching the Rust sources, use the provided Just recipes:

```bash
just wasm-build target=web
# optional helpers
just wasm-test
just wasm-clean
```

The JS glue emitted by `wasm-pack` exposes an async `init` function plus the Rust exports (for example, `add`). These bindings are consumed exclusively inside the `WasmProvider` React context so the module is instantiated once per browser session and shared across all client components.

### Runtime provider pattern

- `src/components/wasm-provider.tsx` is a client component that initializes the WASM module, exposes its readiness/error state, and offers a `reload` helper.
- `src/components/app-provider.tsx` wraps the entire tree with `<WasmProvider>`, so every client component can call `useWasm()` without re-running initialization.
- Components like `src/app/page.tsx` can call `useWasm()` to read `ready`, `error`, and the exported functions (e.g., `add`).

### Bundler considerations

- Next.js already respects the `@wasm/*` path alias declared in `tsconfig.json`, so imports like `import init from '@wasm/rust_wasm'` resolve to the generated `pkg` folder.
- If you need to teach Turbopack how to treat additional file types, add a rule inside the [`turbopack` config](https://nextjs.org/docs/app/api-reference/config/next-config-js/turbopack) in `next.config.ts` (for example, to run a loader over custom binaries).
- When falling back to webpack (production builds or explicit opt-in), you can enable extra features—such as `asyncWebAssembly`—by extending the [`webpack` hook](https://nextjs.org/docs/app/api-reference/config/next-config-js/webpack) inside `next.config.ts`.

In practice, the bindings produced by `wasm-pack` already expose a JS façade that loads the `.wasm` asset via `new URL(..., import.meta.url)`, so no extra loaders are required unless you deviate from that default output.

## Learn More

To learn more about Next.js, take a look at the following resources:

- [Next.js Documentation](https://nextjs.org/docs) - learn about Next.js features and API.
- [Learn Next.js](https://nextjs.org/learn) - an interactive Next.js tutorial.

You can check out [the Next.js GitHub repository](https://github.com/vercel/next.js) - your feedback and contributions are welcome!

## Deploy on Vercel

The easiest way to deploy your Next.js app is to use the [Vercel Platform](https://vercel.com/new?utm_medium=default-template&filter=next.js&utm_source=create-next-app&utm_campaign=create-next-app-readme) from the creators of Next.js.

Check out our [Next.js deployment documentation](https://nextjs.org/docs/app/building-your-application/deploying) for more details.
