'use client';

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useEffectEvent,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from 'react';

import init, * as wasmModule from '@wasm/rust_wasm';

type WasmInitOutput = Awaited<ReturnType<typeof init>>;

export type ShieldedProtocol = 'Sapling' | 'Orchard';
export type TransferType = 'Incoming' | 'Outgoing' | 'Internal';
export type NetworkName = 'mainnet' | 'testnet';

export type DecryptedNote = {
  txid: string;
  index: number;
  value: number;
  memo: Uint8Array;
  protocol: ShieldedProtocol;
  transferType: TransferType;
  height: number;
};

export type DecryptHistoryInput = {
  viewingKey: string;
  network: NetworkName;
  transactions: Array<{
    rawTx: string;
    height: number;
  }>;
};

type RawDecryptedNote = {
  txid: string;
  index: number;
  value: number;
  memo: number[];
  protocol: ShieldedProtocol;
  transfer_type: TransferType;
  height: number;
};

type WasmContextValue = {
  ready: boolean;
  error: Error | null;
  exports: WasmInitOutput | null;
  reload: () => Promise<void>;
  decryptHistory: (input: DecryptHistoryInput) => Promise<DecryptedNote[]>;
};

const WasmContext = createContext<WasmContextValue | undefined>(undefined);

export function WasmProvider({ children }: { children: ReactNode }) {
  const [error, setError] = useState<Error | null>(null);
  const [exports, setExports] = useState<WasmInitOutput | null>(null);
  const inFlightRef = useRef<Promise<WasmInitOutput> | null>(null);

  const handleSuccess = useCallback((result: WasmInitOutput) => {
    setError(null);
    setExports(result);
  }, []);

  const handleFailure = useCallback((err: unknown) => {
    setError(err instanceof Error ? err : new Error('Failed to initialize WASM module'));
    setExports(null);
  }, []);

  const loadWasm = useCallback(async () => {
    if (!inFlightRef.current) {
      inFlightRef.current = init();
    }
    return inFlightRef.current;
  }, []);

  const applyResult = useEffectEvent((promise: Promise<WasmInitOutput>, cancelledRef?: { current: boolean }) => {
    promise
      .then((result) => {
        if (cancelledRef?.current) {
          return;
        }
        handleSuccess(result);
      })
      .catch((err) => {
        if (cancelledRef?.current) {
          return;
        }
        handleFailure(err);
      });
  });

  useEffect(() => {
    const cancelled = { current: false };
    const promise = loadWasm();
    applyResult(promise, cancelled);

    return () => {
      cancelled.current = true;
    };
  }, [loadWasm]);

  const reload = useCallback(async () => {
    setError(null);
    setExports(null);
    inFlightRef.current = null;
    const promise = loadWasm();
    promise.then(handleSuccess).catch(handleFailure);
    await promise;
  }, [handleFailure, handleSuccess, loadWasm]);

  const ready = exports !== null && error === null;

  const decryptHistory = useCallback(
    async (input: DecryptHistoryInput): Promise<DecryptedNote[]> => {
      if (!ready) {
        throw new Error('WASM module is not ready yet');
      }

      if (input.transactions.length === 0) {
        return [];
      }

      const payload = JSON.stringify(
        input.transactions.map((tx) => ({
          raw_tx: tx.rawTx,
          height: tx.height,
        })),
      );

      try {
        const notesJson = wasmModule.decrypt_history(input.viewingKey, payload, input.network);
        return parseDecryptedNotes(notesJson);
      } catch (cause) {
        throw normalizeWasmError(cause);
      }
    },
    [ready],
  );

  const value = useMemo<WasmContextValue>(
    () => ({
      ready,
      error,
      exports,
      reload,
      decryptHistory,
    }),
    [decryptHistory, error, exports, ready, reload],
  );

  return <WasmContext.Provider value={value}>{children}</WasmContext.Provider>;
}

function parseDecryptedNotes(payload: string): DecryptedNote[] {
  const rawNotes = JSON.parse(payload) as RawDecryptedNote[];
  return rawNotes.map((note) => ({
    txid: note.txid,
    index: note.index,
    value: note.value,
    memo: new Uint8Array(note.memo),
    protocol: note.protocol,
    transferType: note.transfer_type,
    height: note.height,
  }));
}

function normalizeWasmError(value: unknown): Error {
  if (value instanceof Error) {
    return value;
  }
  if (typeof value === 'string') {
    return new Error(value);
  }
  return new Error('WASM execution failed');
}

export function useWasm() {
  const context = useContext(WasmContext);
  if (!context) {
    throw new Error('useWasm must be used within a WasmProvider');
  }
  return context;
}
