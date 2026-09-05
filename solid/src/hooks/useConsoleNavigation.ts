import type { InputAction } from '@contracts/input.types';
import type { Game } from '@contracts/game.types';
import type { NavigationStore } from '@stores/navigation.store';
import type { LibraryStore } from '@stores/library.store';
import type { SystemStore } from '@stores/system.store';
import type { ModalStore } from '@stores/modal.store';
import type { SoundFxService } from '@services/audio/sound-fx.service';
import { SETTINGS_TABS } from '@contracts/settings.types';

import { shelfColumns } from '@services/library/grid-layout';
import { ViewportService } from '@services/system/viewport.service';

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
  onSelectGame?: (game: Game) => void;
}

export function useConsoleNavigation(options: UseConsoleNavigationOptions) {
  const viewport = ViewportService.getInstance();
  const itemsPerRow = () => shelfColumns(viewport.width());
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
    onExitPlatform,
    onSelectGame
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

    // Modal: Maintenance Menu
    if (modalStore.isMaintenanceOpen()) {
      switch (action) {
        case 'NAV_DOWN':
          modalStore.setMaintenanceIndex(Math.min(4, modalStore.maintenanceIndex() + 1));
          soundFx.playMove();
          break;
        case 'NAV_UP':
          modalStore.setMaintenanceIndex(Math.max(0, modalStore.maintenanceIndex() - 1));
          soundFx.playMove();
          break;
        case 'BUTTON_A':
          soundFx.playSelect();
          (window as any).__EMUBOX_TRIGGER_MAINTENANCE__?.();
          break;
        case 'BUTTON_B':
          soundFx.playBack();
          modalStore.closeMaintenance();
          break;
        default:
          break;
      }
      return;
    }

    const isModalOpen = modalStore.isEmulatorSelectorOpen();
    const currentSection = navigationStore.currentSection();

    switch (currentSection) {
      case 'settings': {
        if (isModalOpen) return;

        if ((window as any).__EMUBOX_IS_EMULATOR_MODAL_OPEN__?.()) {
          switch (action) {
            case 'NAV_DOWN':
              (window as any).__EMUBOX_EMU_MODAL_NAV__?.('DOWN');
              soundFx.playMove();
              break;
            case 'NAV_UP':
              (window as any).__EMUBOX_EMU_MODAL_NAV__?.('UP');
              soundFx.playMove();
              break;
            case 'NAV_LEFT':
              (window as any).__EMUBOX_EMU_MODAL_NAV__?.('LEFT');
              soundFx.playMove();
              break;
            case 'NAV_RIGHT':
              (window as any).__EMUBOX_EMU_MODAL_NAV__?.('RIGHT');
              soundFx.playMove();
              break;
            case 'BUTTON_A':
              soundFx.playSelect();
              (window as any).__EMUBOX_EMU_MODAL_NAV__?.('SELECT');
              break;
            case 'BUTTON_B':
              soundFx.playBack();
              (window as any).__EMUBOX_CLOSE_EMULATOR_MODAL__?.();
              break;
            default:
              break;
          }
          return;
        }

        const area = settingsFocusArea ? settingsFocusArea() : 'sidebar';
        const rowIdx = settingsRowIndex ? settingsRowIndex() : 0;
        const currentTab = activeSettingsTab ? activeSettingsTab() : 'system';
        const currentTabIndex = SETTINGS_TABS.findIndex((t) => t.id === currentTab);

        switch (area) {
          case 'sidebar': {
            if (action === 'BUTTON_B') {
              soundFx.playBack();
              navigationStore.setCurrentSection('library');
              navigationStore.setLibraryViewMode('wheel');
              return;
            }

            switch (action) {
              case 'NAV_DOWN':
              case 'BUTTON_RB': {
                const nextIdx = currentTabIndex < SETTINGS_TABS.length - 1 ? currentTabIndex + 1 : 0;
                onSettingsTabChange?.(SETTINGS_TABS[nextIdx].id);
                soundFx.playMove();
                break;
              }
              case 'NAV_UP':
              case 'BUTTON_LB': {
                const prevIdx = currentTabIndex > 0 ? currentTabIndex - 1 : SETTINGS_TABS.length - 1;
                onSettingsTabChange?.(SETTINGS_TABS[prevIdx].id);
                soundFx.playMove();
                break;
              }
              case 'NAV_RIGHT':
              case 'BUTTON_A':
                soundFx.playSelect();
                onSettingsFocusAreaChange?.('content');
                onSettingsRowIndexChange?.(0);
                break;
              default:
                break;
            }
            break;
          }

          case 'content': {
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

            let maxRows = 2;
            switch (currentTab) {
              case 'system':
                maxRows = 4;
                break;
              case 'emulators':
                maxRows = systemStore.emulators().length || 1;
                break;
              case 'audio':
                maxRows = 2;
                break;
              case 'gamepad':
                maxRows = 2 + 4;
                break;
              case 'update':
                maxRows = 2;
                break;
              default:
                maxRows = 2;
                break;
            }

            switch (action) {
              case 'NAV_DOWN':
                if (rowIdx < maxRows - 1) {
                  onSettingsRowIndexChange?.(rowIdx + 1);
                  soundFx.playMove();
                }
                break;
              case 'NAV_UP':
                if (rowIdx > 0) {
                  onSettingsRowIndexChange?.(rowIdx - 1);
                  soundFx.playMove();
                }
                break;
              case 'BUTTON_A':
                soundFx.playSelect();
                onToggleCurrentSetting?.();
                break;
              case 'NAV_LEFT':
                if (isSliderRow) {
                  onAdjustCurrentSlider?.(-5);
                }
                break;
              case 'NAV_RIGHT':
                if (isSliderRow) {
                  onAdjustCurrentSlider?.(5);
                }
                break;
              default:
                break;
            }
            break;
          }
          default:
            break;
        }
        break;
      }

      case 'library': {
        if (isModalOpen) return;

        const viewMode = navigationStore.libraryViewMode();

        switch (viewMode) {
          case 'wheel': {
            const totalItems = systemStore.platforms().length + 1;
            if (totalItems === 0) return;

            switch (action) {
              case 'NAV_RIGHT':
              case 'BUTTON_RB':
                navigationStore.setWheelPlatformIndex((navigationStore.wheelPlatformIndex() + 1) % totalItems);
                soundFx.playMove();
                break;
              case 'NAV_LEFT':
              case 'BUTTON_LB':
                navigationStore.setWheelPlatformIndex((navigationStore.wheelPlatformIndex() - 1 + totalItems) % totalItems);
                soundFx.playMove();
                break;
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
                break;
              default:
                break;
            }
            break;
          }

          case 'games': {
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
                break;
              case 'NAV_LEFT':
                if (currentIndex > 0) {
                  navigationStore.setFocusedGameIndex(currentIndex - 1);
                  soundFx.playMove();
                }
                break;
              case 'NAV_DOWN':
                if (currentIndex + itemsPerRow() < totalGames) {
                  navigationStore.setFocusedGameIndex(currentIndex + itemsPerRow());
                  soundFx.playMove();
                } else if (currentIndex < totalGames - 1) {
                  navigationStore.setFocusedGameIndex(totalGames - 1);
                  soundFx.playMove();
                }
                break;
              case 'NAV_UP':
                if (currentIndex - itemsPerRow() >= 0) {
                  navigationStore.setFocusedGameIndex(currentIndex - itemsPerRow());
                  soundFx.playMove();
                } else if (currentIndex > 0) {
                  navigationStore.setFocusedGameIndex(0);
                  soundFx.playMove();
                }
                break;
              case 'BUTTON_A':
                if (now >= transitionCooldownUntil) {
                  soundFx.playSelect();
                  const targetGame = focusedGame();
                  if (targetGame) {
                    if (onSelectGame) {
                      onSelectGame(targetGame);
                    } else {
                      modalStore.openEmulatorSelector(targetGame);
                    }
                  }
                }
                break;
              case 'BUTTON_B':
                soundFx.playBack();
                if (onExitPlatform) onExitPlatform();
                else navigationStore.setLibraryViewMode('wheel');
                break;
              case 'BUTTON_X': {
                const favGame = focusedGame();
                if (favGame) {
                  soundFx.playFavorite();
                  libraryStore.toggleFavorite(favGame.id);
                }
                break;
              }
              default:
                break;
            }
            break;
          }
          default:
            break;
        }
        break;
      }
      default:
        break;
    }
  };

  return {
    handleAction
  };
}
