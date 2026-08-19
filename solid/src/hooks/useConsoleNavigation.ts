import type { InputAction } from '@contracts/input.types';
import type { Game, SystemSettings } from '@contracts/game.types';
import type { NavigationStore } from '@stores/navigation.store';
import type { LibraryStore } from '@stores/library.store';
import type { SystemStore } from '@stores/system.store';
import type { ModalStore } from '@stores/modal.store';
import type { SoundFxService } from '@services/audio/sound-fx.service';
import { SETTINGS_TABS } from '../components/settings/SettingsView';

const ITEMS_PER_ROW = 6;

interface UseConsoleNavigationOptions {
  navigationStore: NavigationStore;
  libraryStore: LibraryStore;
  systemStore: SystemStore;
  modalStore: ModalStore;
  soundFx: SoundFxService;
  platformGames: () => Game[];
  focusedGame: () => Game | null;
  activeSettingsTab?: () => string;
  onSettingsTabChange?: (tab: string) => void;
  settingsFocusArea?: () => 'sidebar' | 'content';
  onSettingsFocusAreaChange?: (area: 'sidebar' | 'content') => void;
  settingsRowIndex?: () => number;
  onSettingsRowIndexChange?: (idx: number) => void;
  onToggleCurrentSetting?: () => void;
  onAdjustCurrentSlider?: (delta: number) => void;
  onEnterPlatform?: () => void;
  onExitPlatform?: () => void;
}

export function useConsoleNavigation(options: UseConsoleNavigationOptions) {
  const {
    navigationStore,
    libraryStore,
    systemStore,
    modalStore,
    soundFx,
    platformGames,
    focusedGame,
    activeSettingsTab,
    onSettingsTabChange,
    settingsFocusArea,
    onSettingsFocusAreaChange,
    settingsRowIndex,
    onSettingsRowIndexChange,
    onToggleCurrentSetting,
    onAdjustCurrentSlider,
    onEnterPlatform,
    onExitPlatform
  } = options;

  let transitionCooldownUntil = 0;

  const handleAction = (action: InputAction) => {
    const now = performance.now();

    // Global Rescue / Maintenance Shortcut
    if (action === 'MAINTENANCE_MENU') {
      soundFx.playSelect();
      modalStore.openMaintenance();
      return;
    }

    if (modalStore.isMaintenanceOpen()) {
      switch (action) {
        case 'NAV_DOWN':
          modalStore.setMaintenanceIndex(Math.min(4, modalStore.maintenanceIndex() + 1));
          soundFx.playMove();
          return;
        case 'NAV_UP':
          modalStore.setMaintenanceIndex(Math.max(0, modalStore.maintenanceIndex() - 1));
          soundFx.playMove();
          return;
        case 'BUTTON_A':
          soundFx.playSelect();
          (window as any).__EMUBOX_TRIGGER_MAINTENANCE__?.();
          return;
        case 'BUTTON_B':
          soundFx.playBack();
          modalStore.closeMaintenance();
          return;
        default:
          return;
      }
    }

    const isModalOpen = modalStore.isEmulatorSelectorOpen();

    // 1. Navigation when in Settings View
    if (!isModalOpen && navigationStore.currentSection() === 'settings') {
      if ((window as any).__EMUBOX_IS_EMULATOR_MODAL_OPEN__?.()) {
        switch (action) {
          case 'NAV_DOWN':
            (window as any).__EMUBOX_EMU_MODAL_NAV__?.('DOWN');
            soundFx.playMove();
            return;
          case 'NAV_UP':
            (window as any).__EMUBOX_EMU_MODAL_NAV__?.('UP');
            soundFx.playMove();
            return;
          case 'NAV_LEFT':
            (window as any).__EMUBOX_EMU_MODAL_NAV__?.('LEFT');
            soundFx.playMove();
            return;
          case 'NAV_RIGHT':
            (window as any).__EMUBOX_EMU_MODAL_NAV__?.('RIGHT');
            soundFx.playMove();
            return;
          case 'BUTTON_A':
            soundFx.playSelect();
            (window as any).__EMUBOX_EMU_MODAL_NAV__?.('SELECT');
            return;
          case 'BUTTON_B':
            soundFx.playBack();
            (window as any).__EMUBOX_CLOSE_EMULATOR_MODAL__?.();
            return;
          default:
            return;
        }
      }
      const area = settingsFocusArea ? settingsFocusArea() : 'sidebar';
      const rowIdx = settingsRowIndex ? settingsRowIndex() : 0;
      const currentTab = activeSettingsTab ? activeSettingsTab() : 'system';
      const currentTabIndex = SETTINGS_TABS.indexOf(currentTab as any);

      // A. When focused on the Sidebar Tabs
      if (area === 'sidebar') {
        if (action === 'BUTTON_B') {
          soundFx.playBack();
          navigationStore.setCurrentSection('library');
          navigationStore.setLibraryViewMode('wheel');
          return;
        }

        switch (action) {
          case 'NAV_DOWN':
          case 'BUTTON_RB':
            if (currentTabIndex < SETTINGS_TABS.length - 1) {
              onSettingsTabChange?.(SETTINGS_TABS[currentTabIndex + 1]);
              soundFx.playMove();
            } else {
              onSettingsTabChange?.(SETTINGS_TABS[0]);
              soundFx.playMove();
            }
            return;

          case 'NAV_UP':
          case 'BUTTON_LB':
            if (currentTabIndex > 0) {
              onSettingsTabChange?.(SETTINGS_TABS[currentTabIndex - 1]);
              soundFx.playMove();
            } else {
              onSettingsTabChange?.(SETTINGS_TABS[SETTINGS_TABS.length - 1]);
              soundFx.playMove();
            }
            return;

          case 'NAV_RIGHT':
          case 'BUTTON_A':
            soundFx.playSelect();
            onSettingsFocusAreaChange?.('content');
            onSettingsRowIndexChange?.(0);
            return;

          default:
            return;
        }
      }

      // B. When focused inside the Right Settings Content Pane
      if (area === 'content') {
        const isSliderRow = currentTab === 'audio' && rowIdx === 1;

        if (action === 'BUTTON_B') {
          soundFx.playBack();
          onSettingsFocusAreaChange?.('sidebar');
          return;
        }

        if (action === 'NAV_LEFT' && !isSliderRow) {
          soundFx.playBack();
          onSettingsFocusAreaChange?.('sidebar');
          return;
        }

        // Calculate maximum rows for current tab
        let maxRows = 2;
        if (currentTab === 'system') {
          maxRows = 4;
        } else if (currentTab === 'emulators') {
          maxRows = systemStore.emulators().length || 1;
        } else if (currentTab === 'audio') {
          maxRows = 2;
        } else if (currentTab === 'gamepad') {
          maxRows = 2 + 4; // rumble + primary + up to 4 pads
        } else if (currentTab === 'update') {
          maxRows = 2; // Auto-Update Switch (0), Action Button (1)
        }

        switch (action) {
          case 'NAV_DOWN':
            if (rowIdx < maxRows - 1) {
              onSettingsRowIndexChange?.(rowIdx + 1);
              soundFx.playMove();
            }
            return;

          case 'NAV_UP':
            if (rowIdx > 0) {
              onSettingsRowIndexChange?.(rowIdx - 1);
              soundFx.playMove();
            }
            return;

          case 'BUTTON_A':
            soundFx.playSelect();
            onToggleCurrentSetting?.();
            return;

          case 'NAV_LEFT':
            if (isSliderRow) {
              onAdjustCurrentSlider?.(-5);
            }
            return;

          case 'NAV_RIGHT':
            if (isSliderRow) {
              onAdjustCurrentSlider?.(5);
            }
            return;

          default:
            return;
        }
      }
    }

    // 2. Navigation when in LEVEL 1 (Platform & System Hub Wheel)
    if (!isModalOpen && navigationStore.currentSection() === 'library' && navigationStore.libraryViewMode() === 'wheel') {
      const totalItems = systemStore.platforms().length + 1; // Consoles + Ajustes
      if (totalItems === 0) return;

      switch (action) {
        case 'NAV_RIGHT':
        case 'BUTTON_RB':
          navigationStore.setWheelPlatformIndex((navigationStore.wheelPlatformIndex() + 1) % totalItems);
          soundFx.playMove();
          return;

        case 'NAV_LEFT':
        case 'BUTTON_LB':
          navigationStore.setWheelPlatformIndex((navigationStore.wheelPlatformIndex() - 1 + totalItems) % totalItems);
          soundFx.playMove();
          return;

        case 'BUTTON_A':
          soundFx.playSelect();
          if (typeof document !== 'undefined' && document.activeElement instanceof HTMLElement) {
            document.activeElement.blur();
          }
          transitionCooldownUntil = performance.now() + 220;

          if (onEnterPlatform) {
            onEnterPlatform();
          } else {
            navigationStore.setFocusedGameIndex(0);
            navigationStore.setLibraryViewMode('games');
          }
          return;

        default:
          return;
      }
    }

    // 3. Navigation when in LEVEL 2 (Platform Games Shelf)
    if (!isModalOpen && navigationStore.currentSection() === 'library' && navigationStore.libraryViewMode() === 'games') {
      const totalGames = platformGames().length;
      if (totalGames === 0) {
        if (action === 'BUTTON_B') {
          soundFx.playBack();
          if (onExitPlatform) onExitPlatform();
          else navigationStore.setLibraryViewMode('wheel');
        }
        return;
      }

      const currentIndex = navigationStore.focusedGameIndex();

      switch (action) {
        case 'NAV_RIGHT':
          if (currentIndex < totalGames - 1) {
            navigationStore.setFocusedGameIndex(currentIndex + 1);
            soundFx.playMove();
          }
          return;

        case 'NAV_LEFT':
          if (currentIndex > 0) {
            navigationStore.setFocusedGameIndex(currentIndex - 1);
            soundFx.playMove();
          }
          return;

        case 'NAV_DOWN':
          if (currentIndex + ITEMS_PER_ROW < totalGames) {
            navigationStore.setFocusedGameIndex(currentIndex + ITEMS_PER_ROW);
            soundFx.playMove();
          } else if (currentIndex < totalGames - 1) {
            navigationStore.setFocusedGameIndex(totalGames - 1);
            soundFx.playMove();
          }
          return;

        case 'NAV_UP':
          if (currentIndex - ITEMS_PER_ROW >= 0) {
            navigationStore.setFocusedGameIndex(currentIndex - ITEMS_PER_ROW);
            soundFx.playMove();
          } else if (currentIndex > 0) {
            navigationStore.setFocusedGameIndex(0);
            soundFx.playMove();
          }
          return;

        case 'BUTTON_A':
          if (now < transitionCooldownUntil) return;
          soundFx.playSelect();
          const targetGame = focusedGame();
          if (targetGame) {
            modalStore.openEmulatorSelector(targetGame);
          }
          return;

        case 'BUTTON_B':
          soundFx.playBack();
          if (onExitPlatform) onExitPlatform();
          else navigationStore.setLibraryViewMode('wheel');
          return;

        case 'BUTTON_X':
          const favGame = focusedGame();
          if (favGame) {
            soundFx.playFavorite();
            libraryStore.toggleFavorite(favGame.id);
          }
          return;

        default:
          return;
      }
    }
  };

  return {
    handleAction
  };
}
