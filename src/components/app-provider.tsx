import { NuqsAdapter } from 'nuqs/adapters/next/app';

export function AppProvider({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return <NuqsAdapter>{children}</NuqsAdapter>;
}
