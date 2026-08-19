import type { Accessor, Setter } from 'solid-js';
import type { Game, Emulator } from './game.types';
import type { IEmuBoxBackend } from './backend.types';

export type MaintenanceActionId = 'restart-app' | 'repair-dirs' | 'check-updates' | 'reboot' | 'poweroff';

export interface MaintenanceAction {
  id: MaintenanceActionId;
  tag: string;
  title: string;
  description: string;
  variant?: 'primary' | 'danger' | 'warning' | 'default';
  action: () => Promise<void> | void;
}

export interface UseMaintenanceOptions {
  backend: IEmuBoxBackend;
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
  onClose: () => void;
  backend: IEmuBoxBackend;
  focusedIndex?: number;
  onSelectIndex?: (idx: number) => void;
}

export interface EmulatorCrudFormData {
  id?: string;
  name?: string;
  supportedPlatforms?: string[];
  coreType?: 'libretro' | 'standalone';
  executable?: string;
  arguments?: string[];
  version?: string;
  status?: 'active' | 'inactive';
}

export interface UseEmulatorCrudOptions {
  isOpen: () => boolean;
  initialData: () => Emulator | null | undefined;
  onClose: () => void;
  onSave: (emulator: Emulator) => void;
  onDelete: (emulatorId: string) => void;
}

export interface UseEmulatorCrudReturn {
  formData: Accessor<Partial<Emulator>>;
  setFormData: Setter<Partial<Emulator>>;
  modalFocusIdx: Accessor<number>;
  isTyping: Accessor<boolean>;
  setIsTyping: Setter<boolean>;
  focusModalElement: (idx: number) => void;
  handleSave: () => void;
  handleDelete: () => void;
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
  isOpen: boolean;
  initialData?: Emulator | null;
  onClose: () => void;
  onSave: (emulator: Emulator) => void;
  onDelete: (emulatorId: string) => void;
}

export interface EmulatorSelectorModalProps {
  game: Game | null;
  emulators: Emulator[];
  isOpen: boolean;
  onClose: () => void;
  onConfirmLaunch: (game: Game, emulator: Emulator) => void;
}

export interface ConfirmLaunchModalProps {
  game: Game | null;
  isOpen: boolean;
  onClose: () => void;
  onConfirmLaunch: (game: Game) => void;
}

export interface GameDetailsModalProps {
  game: Game | null;
  isOpen: boolean;
  onClose: () => void;
  onLaunch: (game: Game) => void;
  onToggleFavorite: (id: string) => void;
}
