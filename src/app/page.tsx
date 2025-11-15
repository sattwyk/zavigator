'use client';

import { useMemo } from 'react';

import { useWasm } from '@/components/wasm-provider';

export default function Home() {
  const { ready, error, add, reload } = useWasm();
  const result = useMemo(() => (ready && add ? add(2, 10) : null), [add, ready]);

  return (
    <div>
      <main>
        <h1>Welcome to My Next.js App</h1>
        {error && (
          <p className="text-red-500">
            Failed to load WebAssembly module. <button onClick={reload}>Retry</button>
          </p>
        )}
        {!error && (ready ? <p>2 + 3 = {result}</p> : <p>Loading WASM...</p>)}
      </main>
    </div>
  );
}
