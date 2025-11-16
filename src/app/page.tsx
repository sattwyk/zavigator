'use client';

import { useWasm } from '@/components/wasm-provider';

export default function Home() {
  const { ready, error, reload } = useWasm();

  const statusMessage = error
    ? 'We could not initialize the local decryptor. Please retry.'
    : ready
      ? 'Local shielded data decryptor is ready on your device.'
      : 'Preparing the local decryptor so viewing keys never leave the browser...';

  return (
    <div className="mx-auto flex min-h-screen max-w-4xl flex-col gap-8 px-6 py-12">
      <section className="space-y-6">
        <div className="space-y-2">
          <p className="text-sm tracking-wide text-slate-500 uppercase">Zavigator</p>
          <h1 className="text-3xl font-semibold">
            Private-by-default shielded history, decrypted entirely in your browser.
          </h1>
          <p className="text-slate-600">
            Paste a Unified Full Viewing Key, fetch encrypted transactions from the indexer, and let the WASM runtime
            turn them into human-friendly finance insights without exposing your secrets.
          </p>
        </div>
        <ul className="grid gap-4 sm:grid-cols-2">
          <li className="rounded-lg border border-slate-200 bg-white/50 p-4 shadow-sm">
            <p className="font-medium">Viewing key never leaves the tab</p>
            <p className="text-sm text-slate-600">
              Decryption happens via `decrypt_history` inside the WASM module compiled from `indexer-core`.
            </p>
          </li>
          <li className="rounded-lg border border-slate-200 bg-white/50 p-4 shadow-sm">
            <p className="font-medium">Composable building blocks</p>
            <p className="text-sm text-slate-600">
              The frontend will combine decrypted notes with filters, analytics, and memos to build a personal
              finance-like dashboard.
            </p>
          </li>
        </ul>
      </section>

      <section className="rounded-2xl border border-slate-200 bg-white/70 p-6 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <div>
            <h2 className="text-lg font-semibold">Local decrypt runtime</h2>
            <p className="text-sm text-slate-600">{statusMessage}</p>
          </div>
          <StatusBadge ready={ready} error={Boolean(error)} />
        </div>
        {error && (
          <button
            type="button"
            onClick={reload}
            className="mt-4 inline-flex items-center rounded-md bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-800"
          >
            Retry load
          </button>
        )}
      </section>
    </div>
  );
}

function StatusBadge({ ready, error }: { ready: boolean; error: boolean }) {
  if (error) {
    return (
      <span className="inline-flex items-center rounded-full bg-red-100 px-3 py-1 text-sm font-medium text-red-700">
        Error
      </span>
    );
  }

  if (ready) {
    return (
      <span className="inline-flex items-center rounded-full bg-emerald-100 px-3 py-1 text-sm font-medium text-emerald-700">
        Ready
      </span>
    );
  }

  return (
    <span className="inline-flex items-center rounded-full bg-slate-100 px-3 py-1 text-sm font-medium text-slate-700">
      Loading...
    </span>
  );
}
