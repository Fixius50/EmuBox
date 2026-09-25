import type { SystemSettings, Emulator } from "./game.types";
import type { SystemStore } from "@stores/system.store";
import type { SoundFxService } from "@services/audio/sound-fx.service";
import type { InputAction } from "./input.types";
import type { IEmuBoxBackend } from "./backend.types";
import type { DisplayInfo } from "./system.types";

export type SettingsTabId = "system" | "audio" | "gamepad" | "services" | "stores";

export interface TabItem {
  id: SettingsTabId;
  name: string;
  tag: string;
  desc: string;
}

export const SETTINGS_TABS: readonly TabItem[] = [
  {
    id: "system",
    name: "Sistema y pantalla",
    tag: "OS",
    desc: "Preferencias e informacion",
  },
  {
    id: "audio",
    name: "Audio general",
    tag: "SND",
    desc: "Entrada, salida y volumen",
  },
  {
    id: "gamepad",
    name: "Mando & Controles",
    tag: "PAD",
    desc: "Dispositivos Conectados",
  },
  {
    id: "services",
    name: "Servicios",
    tag: "NET",
    desc: "Descargas y conexiones locales",
  },
  {
    id: "stores",
    name: "Tiendas",
    tag: "STORE",
    desc: "Bibliotecas y cuentas externas",
  },
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
  activeSettingsTab: () => string;
  settingsRowIndex: () => number;
}

export interface UseSettingsNavigationOptions {
  soundFx: SoundFxService;
  emulatorCount: () => number;
  storeProviderCount?: () => number;
  activeSettingsTab: () => string;
  settingsFocusArea: () => "sidebar" | "content";
  settingsRowIndex: () => number;
  onSettingsTabChange: (tab: SettingsTabId) => void;
  onSettingsFocusAreaChange: (area: "sidebar" | "content") => void;
  onSettingsRowIndexChange: (index: number) => void;
  onToggleCurrentSetting: () => void;
  onAdjustCurrentSlider: (delta: number) => void;
  onBack: () => void;
}

export interface UseSettingsControllerReturn {
  handleSaveEmulator: (emulator: Emulator) => Promise<void>;
  handleDeleteEmulator: (emulatorId: string) => Promise<void>;
  handleToggleCurrentSetting: () => void;
  handleAdjustCurrentSlider: (delta: number) => void;
  triggerVibrationTest: (padIndex: number) => void;
}

export interface SettingsSidebarProps {
  activeTab: string;
  focusArea?: "sidebar" | "content";
  onTabChange: (tabId: SettingsTabId) => void;
}

export interface SystemTabProps {
  settings?: SystemSettings;
  emulators?: Emulator[];
  displayInfo?: DisplayInfo | null;
  isRowFocused: (row: number) => boolean;
  onSelectContentArea?: () => void;
  onUpdateSettings: (updater: (s: SystemSettings) => void) => void;
}

export interface ServicesTabProps {
  settings?: SystemSettings;
  backend?: IEmuBoxBackend;
  libraryStore?: import('@stores/library.store').LibraryStore;
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
  backend?: IEmuBoxBackend;
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

export interface SettingsViewProps {
  onControllerReady?: (handler: ((action: InputAction) => void) | null) => void;
  onNavigate?: (action: InputAction) => void;
  settings?: SystemSettings;
  emulators?: Emulator[];
  storeBackend?: IEmuBoxBackend;
  libraryStore?: import('@stores/library.store').LibraryStore;
  activeTab?: string;
  focusArea?: "sidebar" | "content";
  focusedRowIndex?: number;
  onTabChange?: (tab: string) => void;
  onUpdateSettings?: (newSettings: SystemSettings) => void;
  onSelectContentArea?: () => void;
  onBack?: () => void;
}
