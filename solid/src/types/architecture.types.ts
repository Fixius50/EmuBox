export type Architecture = 'x86_64' | 'aarch64' | 'unsupported';

export function normalizeArchitecture(value: string): Architecture {
  if (value === 'x86_64') return 'x86_64';
  if (value === 'aarch64' || value === 'arm64') return 'aarch64';
  return 'unsupported';
}