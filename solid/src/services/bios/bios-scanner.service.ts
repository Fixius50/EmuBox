import type { BiosStatus, BiosRequirement } from '@contracts/bios.types';

export class BiosScannerService {
  private defaultRequirements: Record<string, BiosRequirement> = {
    ps1: {
      platformId: 'ps1',
      platformName: 'Sony PlayStation',
      emulatorId: 'duckstation',
      allRequiredPresent: true,
      biosFiles: [
        {
          filename: 'scph1001.bin',
          description: 'US PlayStation BIOS v4.1',
          state: 'found_valid'
        }
      ]
    },
    ps2: {
      platformId: 'ps2',
      platformName: 'Sony PlayStation 2',
      emulatorId: 'pcsx2',
      allRequiredPresent: true,
      biosFiles: [
        {
          filename: 'SCPH-70012.bin',
          description: 'PlayStation 2 v12 BIOS',
          state: 'found_valid'
        }
      ]
    }
  };

  public async scanBios(): Promise<BiosStatus> {
    return {
      totalRequired: 2,
      totalFound: 2,
      missingRequiredCount: 0,
      platforms: this.defaultRequirements
    };
  }
}
