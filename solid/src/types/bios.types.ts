export type BiosState = 'required_missing' | 'optional_missing' | 'found_valid' | 'found_invalid_checksum';

export interface BiosFile {
  filename: string;
  expectedMd5?: string;
  expectedSha1?: string;
  description: string;
  foundPath?: string;
  state: BiosState;
  fileSizeBytes?: number;
}

export interface BiosRequirement {
  platformId: string;
  platformName: string;
  emulatorId: string;
  biosFiles: BiosFile[];
  allRequiredPresent: boolean;
}

export interface BiosStatus {
  totalRequired: number;
  totalFound: number;
  missingRequiredCount: number;
  platforms: Record<string, BiosRequirement>;
}
