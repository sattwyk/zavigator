import { NuqsAdapter } from 'nuqs/adapters/next/app';

import { WasmProvider } from '@/components/wasm-provider';

export function AppProvider({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <NuqsAdapter>
      <WasmProvider>{children}</WasmProvider>
    </NuqsAdapter>
  );
}
