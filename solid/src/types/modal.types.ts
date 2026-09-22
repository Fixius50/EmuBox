import type { Accessor, Setter } from "solid-js";
import type { Game, Emulator } from "./game.types";
import type { IEmuBoxBackend } from "./backend.types";
import type { InputAction } from "./input.types";

export type MaintenanceActionId =
  "restart-app" | "repair-dirs" | "check-updates" | "reboot" | "poweroff";

export interface MaintenanceAction {
  id: MaintenanceActionId;
  tag: string;
  title: string;
  description: string;
  variant?: "primary" | "danger" | "warning" | "default";
  action: () => Promise<void> | void;
}

export interface UseMaintenanceOptions {
  backend: IEmuBoxBackend;
  isOpen: () => boolean;
  onControllerReady?: (handler: ((action: InputAction) => void) | null) => void;
  onClose: () => void;
  focusedIndex?: () => number;
  onSelectIndex?: (idx: number) => void;
}

export interface UseMaintenanceReturn {
  actions: MaintenanceAction[];
  feedbackMsg: Accessor<string>;
  isLoading: Accessor<boolean>;
  handleExecute: (idx: number) => Promise<void>;
  activeIndex: Accessor<number>;
}

export interface MaintenanceModalProps {
  isOpen: boolean;
  onControllerReady?: (handler: ((action: InputAction) => void) | null) => void;
  onClose: () => void;
  backend: IEmuBoxBackend;
  focusedIndex?: number;
  onSelectIndex?: (idx: number) => void;
}

export interface EmulatorCrudFormData {
  id?: string;
  name?: string;
  supportedPlatforms?: string[];
  coreType?: "libretro" | "standalone";
  executable?: string;
  arguments?: string[];
  version?: string;
  status?: "active" | "inactive";
}

export interface UseEmulatorCrudOptions {
  onControllerReady?: (handler: ((action: InputAction) => void) | null) => void;
  isOpen: () => boolean;
  initialData: () => Emulator | null | undefined;
  onClose: () => void;
  onSave: (emulator: Emulator) => Promise<void>;
  onDelete: (emulatorId: string) => Promise<void>;
}

export interface UseEmulatorCrudReturn {
  formData: Accessor<Partial<Emulator>>;
  setFormData: Setter<Partial<Emulator>>;
  modalFocusIdx: Accessor<number>;
  isTyping: Accessor<boolean>;
  setIsTyping: Setter<boolean>;
  isPending: Accessor<boolean>;
  error: Accessor<string>;
  handleClose: () => void;
  focusModalElement: (idx: number) => void;
  handleSave: () => Promise<void>;
  handleDelete: () => Promise<void>;
  setElementRefs: (refs: {
    modalBoxRef?: HTMLDivElement;
    nameInputRef?: HTMLInputElement;
    execInputRef?: HTMLInputElement;
    typeInputRef?: HTMLInputElement;
    deleteBtnRef?: HTMLButtonElement;
    cancelBtnRef?: HTMLButtonElement;
    saveBtnRef?: HTMLButtonElement;
  }) => void;
}

export interface EmulatorCrudModalProps {
  onControllerReady?: (handler: ((action: InputAction) => void) | null) => void;
  isOpen: boolean;
  initialData?: Emulator | null;
  onClose: () => void;
  onSave: (emulator: Emulator) => Promise<void>;
  onDelete: (emulatorId: string) => Promise<void>;
}

export interface EmulatorSelectorModalProps {
  onControllerReady?: (handler: ((action: InputAction) => void) | null) => void;
  game: Game | null;
  emulators: Emulator[];
  isOpen: boolean;
  onClose: () => void;
  onConfirmLaunch: (game: Game, emulator: Emulator) => void;
  getPreferredEmulator: (gameId: string) => Promise<string | undefined>;
  onSavePreference: (game: Game, emulator: Emulator) => Promise<void>;
}
