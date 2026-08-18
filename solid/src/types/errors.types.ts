export type EmuBoxErrorCode =
  | 'NotFound'
  | 'PermissionDenied'
  | 'InvalidConfiguration'
  | 'EmulatorNotInstalled'
  | 'EmulatorNotConfigured'
  | 'BiosMissing'
  | 'ExecutableMissing'
  | 'ProcessFailed'
  | 'GameLaunchFailed'
  | 'StorageUnavailable'
  | 'HardwareUnavailable'
  | 'IpcError'
  | 'Unknown';

export interface EmuBoxErrorPayload {
  code: EmuBoxErrorCode;
  message: string;
  details?: Record<string, unknown> | string;
  timestamp: number;
}

export class EmuBoxError extends Error {
  public readonly code: EmuBoxErrorCode;
  public readonly details?: Record<string, unknown> | string;
  public readonly timestamp: number;

  constructor(code: EmuBoxErrorCode, message: string, details?: Record<string, unknown> | string) {
    super(message);
    this.name = 'EmuBoxError';
    this.code = code;
    this.details = details;
    this.timestamp = Date.now();
  }

  public toJSON(): EmuBoxErrorPayload {
    return {
      code: this.code,
      message: this.message,
      details: this.details,
      timestamp: this.timestamp
    };
  }

  public static fromPayload(payload: EmuBoxErrorPayload): EmuBoxError {
    return new EmuBoxError(payload.code, payload.message, payload.details);
  }
}
