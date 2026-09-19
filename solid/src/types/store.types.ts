export interface StoreProviderInfo {
  id: 'steam' | 'epic' | 'gog';
  name: string;
  authenticated: boolean;
}

export interface StoreAccount {
  id: string;
  provider: StoreProviderInfo['id'];
  externalAccountId: string;
  displayName: string;
  avatarUrl?: string | null;
  status: string;
  lastLoginAt?: number | null;
  lastSyncAt?: number | null;
}

export interface StoreEntitlement {
  accountId: string;
  provider: StoreProviderInfo['id'];
  externalGameId: string;
  title: string;
  owned: boolean;
  installed: boolean;
  canonicalGameId?: string | null;
  lastSeenAt: number;
}