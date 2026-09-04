import type { Accessor, Setter } from 'solid-js';
import type { SystemSettings, Emulator } from './game.types';
import type { UpdateInfo, UpdateCheckResult, UpdateProgress } from './update.types';
import type { SystemStore } from '@stores/system.store';
import type { SoundFxService } from '@services/audio/sound-fx.service';
import type { IEmuBoxBackend } from './backend.types';

export type SettingsTabId = 'system' | 'emulators' | 'audio' | 'gamepad';

export interface TabItem {
  id: SettingsTabId;
  name: string;
  tag: string;
  desc: string;
}

export const SETTINGS_TABS: readonly TabItem[] = [
  { id: 'system', name: 'Sistema & Pantalla', tag: 'OS', desc: 'Hardware, GPU, Auto-Update, VSync' },
  { id: 'emulators', name: 'Núcleos & Emuladores', tag: 'EMU', desc: 'Cores Libretro, Binarios' },
  { id: 'audio', name: 'Audio & Sintetizador', tag: 'SND', desc: 'Volumen, Efectos UI' },
  { id: 'gamepad', name: 'Mando & Controles', tag: 'PAD', desc: 'Dispositivos Conectados' }
] as const;

export interface GamepadDeviceInfo {
  index: number;
  id: string;
  buttonsCount: number;
  axesCount: number;
  hasVibration: boolean;
}

export interface UseSettingsControllerOptions {
  systemStore: SystemStore;
  soundFx: SoundFxService;
  backend: IEmuBoxBackend;
  activeSettingsTab: () => string;
  settingsRowIndex: () => number;
}

export interface UseSettingsControllerReturn {
  updateInfo: Accessor<UpdateInfo | undefined>;
  setUpdateInfo: Setter<UpdateInfo | undefined>;
  handleCheckUpdates: () => Promise<UpdateCheckResult | undefined>;
  handleApplyUpdate: (ver?: string) => Promise<UpdateProgress | undefined>;
  handleSaveEmulator: (emulator: Emulator) => void;
  handleDeleteEmulator: (emulatorId: string) => void;
  handleToggleCurrentSetting: () => void;
  handleAdjustCurrentSlider: (delta: number) => void;
  triggerVibrationTest: (padIndex: number) => void;
}

export interface SettingsSidebarProps {
  activeTab: string;
  focusArea?: 'sidebar' | 'content';
  onTabChange: (tabId: SettingsTabId) => void;
}

export interface SystemTabProps {
  settings?: SystemSettings;
  isRowFocused: (row: number) => boolean;
  onSelectContentArea?: () => void;
  onUpdateSettings: (updater: (s: SystemSettings) => void) => void;
}

export interface EmulatorsTabProps {
  emulators?: Emulator[];
  isRowFocused: (row: number) => boolean;
  onSelectContentArea?: () => void;
  onOpenEditModal: (emu?: Emulator) => void;
}

export interface AudioTabProps {
  settings?: SystemSettings;
  isRowFocused: (row: number) => boolean;
  onSelectContentArea?: () => void;
  onUpdateSettings: (updater: (s: SystemSettings) => void) => void;
}

export interface GamepadTabProps {
  settings?: SystemSettings;
  isRowFocused: (row: number) => boolean;
  onSelectContentArea?: () => void;
  onUpdateSettings: (updater: (s: SystemSettings) => void) => void;
  onTriggerGamepadTest?: (padIndex: number) => void;
}

export interface UpdateTabProps {
  settings?: SystemSettings;
  updateInfo?: UpdateInfo;
  isRowFocused: (row: number) => boolean;
  onSelectContentArea?: () => void;
  onUpdateSettings: (updater: (s: SystemSettings) => void) => void;
  onCheckUpdates?: () => Promise<any>;
  onApplyUpdate?: (ver: string) => Promise<any>;
}

export interface SettingsViewProps {
  settings?: SystemSettings;
  emulators?: Emulator[];
  updateInfo?: UpdateInfo;
  activeTab?: string;
  focusArea?: 'sidebar' | 'content';
  focusedRowIndex?: number;
  onTabChange?: (tab: string) => void;
  onUpdateSettings?: (newSettings: SystemSettings) => void;
  onSelectContentArea?: () => void;
  onBack?: () => void;
  onSaveEmulator?: (emulator: Emulator) => void;
  onDeleteEmulator?: (emulatorId: string) => void;
  onCheckUpdates?: () => Promise<any>;
  onApplyUpdate?: (ver: string) => Promise<any>;
}
