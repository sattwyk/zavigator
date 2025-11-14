'use client';

import { useEffect, useState } from 'react';

import init, { add } from '@wasm/rust_wasm';

export default function Home() {
  const [wasmReady, setWasmReady] = useState(false);
  const [result, setResult] = useState<number | null>(null);

  useEffect(() => {
    // Initialize the WASM module
    init().then(() => {
      setWasmReady(true);
      setResult(add(2, 3));
    });
  }, []);

  return (
    <div>
      <main>
        <h1>Welcome to My Next.js App</h1>
        {wasmReady ? <p>2 + 3 = {result}</p> : <p>Loading WASM...</p>}
      </main>
    </div>
  );
}
