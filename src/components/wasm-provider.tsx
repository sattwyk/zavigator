'use client';

import { createContext, useCallback, useContext, useEffect, useMemo, useRef, useState, type ReactNode } from 'react';

import init, * as wasmModule from '@wasm/rust_wasm';

type WasmInitOutput = Awaited<ReturnType<typeof init>>;

type WasmContextValue = {
  ready: boolean;
  error: Error | null;
  exports: WasmInitOutput | null;
  add: typeof wasmModule.add | null;
  reload: () => Promise<void>;
};

const WasmContext = createContext<WasmContextValue | undefined>(undefined);

export function WasmProvider({ children }: { children: ReactNode }) {
  const [ready, setReady] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [exports, setExports] = useState<WasmInitOutput | null>(null);
  const inFlightRef = useRef<Promise<WasmInitOutput> | null>(null);

  const loadWasm = useCallback(async () => {
    if (!inFlightRef.current) {
      inFlightRef.current = init();
    }
    return inFlightRef.current;
  }, []);

  const applyResult = useCallback((promise: Promise<WasmInitOutput>, cancelledRef?: { current: boolean }) => {
    promise
      .then((result) => {
        if (cancelledRef?.current) {
          return;
        }
        setReady(true);
        setError(null);
        setExports(result);
      })
      .catch((err) => {
        if (cancelledRef?.current) {
          return;
        }
        setError(err instanceof Error ? err : new Error('Failed to initialize WASM module'));
        setReady(false);
        setExports(null);
      });
  }, []);

  useEffect(() => {
    const cancelled = { current: false };
    const promise = loadWasm();
    applyResult(promise, cancelled);

    return () => {
      cancelled.current = true;
    };
  }, [applyResult, loadWasm]);

  const reload = useCallback(async () => {
    setReady(false);
    setError(null);
    setExports(null);
    inFlightRef.current = null;
    const promise = loadWasm();
    applyResult(promise);
    await promise;
  }, [applyResult, loadWasm]);

  const value = useMemo<WasmContextValue>(
    () => ({
      ready,
      error,
      exports,
      add: ready ? wasmModule.add : null,
      reload,
    }),
    [error, exports, ready, reload],
  );

  return <WasmContext.Provider value={value}>{children}</WasmContext.Provider>;
}

export function useWasm() {
  const context = useContext(WasmContext);
  if (!context) {
    throw new Error('useWasm must be used within a WasmProvider');
  }
  return context;
}
