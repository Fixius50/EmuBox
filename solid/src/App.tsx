import { Component, createMemo, onMount, createSignal, Show } from 'solid-js';

// Types
import type { Game, Emulator } from '@contracts/game.types';

// Services
import { MockBackendService } from '@services/backend/mock-backend.service';
import { TauriBackendService } from '@services/backend/tauri-backend.service';
import { SoundFxService } from '@services/audio/sound-fx.service';

// Stores
import { createLibraryStore } from '@stores/library.store';
import { createSystemStore } from '@stores/system.store';
import { createNavigationStore } from '@stores/navigation.store';
import { createModalStore } from '@stores/modal.store';

// Hooks
import { useConsoleInput } from '@hooks/useConsoleInput';
import { useConsoleNavigation } from '@hooks/useConsoleNavigation';
import { useGameLauncher } from '@hooks/useGameLauncher';

// Components
import { Shell } from '@components/layout/Shell';
import { Header } from '@components/layout/Header';
import { PlatformWheel, PlatformWheelHandle } from '@components/platforms/PlatformWheel';
import { PlatformGamesView } from '@components/library/PlatformGamesView';
import { EmulatorSelectorModal } from '@components/modals/EmulatorSelectorModal';
import { SettingsView } from '@components/settings/SettingsView';

// Data
import gamesDataset from '@data/games-10000.json';

const PERFORMANCE_MODES_LIST = ['high-performance', 'balanced', 'power-saver', 'ultra-boost'] as const;

export const App: Component = () => {
  // 1. Singletons & Stores
  const mockBackend = new MockBackendService();
  const backend = new TauriBackendService(mockBackend);
  const soundFx = new SoundFxService();

  const libraryStore = createLibraryStore(backend);
  const systemStore = createSystemStore(backend);
  const navigationStore = createNavigationStore();
  const modalStore = createModalStore();

  const [activeSettingsTab, setActiveSettingsTab] = createSignal<string>('system');
  const [settingsFocusArea, setSettingsFocusArea] = createSignal<'sidebar' | 'content'>('sidebar');
  const [settingsRowIndex, setSettingsRowIndex] = createSignal<number>(0);

  let platformWheelHandle: PlatformWheelHandle | undefined;

  // 2. Computed View Memos
  const activePlatform = createMemo(() => {
    const list = systemStore.platforms();
    if (list.length === 0) return null;
    return list[navigationStore.wheelPlatformIndex()] || list[0] || null;
  });

  const platformGames = createMemo(() => {
    const all = libraryStore.games();
    const plat = activePlatform();
    return plat ? all.filter(g => g.platform === plat.id) : all;
  });

  const focusedGame = createMemo(() => {
    const list = platformGames();
    const idx = navigationStore.focusedGameIndex();
    if (list.length === 0) return null;
    return list[idx] || list[list.length - 1] || null;
  });

  const ambientBackdrop = createMemo(() => {
    if (navigationStore.libraryViewMode() === 'games') {
      const g = focusedGame();
      if (g) return g.backdropImage || g.coverImage;
    }
    return '';
  });

  // Toggle or adjust setting action handlers
  const handleToggleCurrentSetting = () => {
    const settings = systemStore.settings();
    if (!settings) return;
    const tab = activeSettingsTab();
    const row = settingsRowIndex();
    const clone = JSON.parse(JSON.stringify(settings));

    if (tab === 'system' && row === 0) {
      // Rotate Performance Mode
      const current = clone.system?.performanceMode || 'high-performance';
      const curIdx = PERFORMANCE_MODES_LIST.indexOf(current as any);
      const nextMode = PERFORMANCE_MODES_LIST[(curIdx + 1) % PERFORMANCE_MODES_LIST.length];
      if (!clone.system) clone.system = {};
      clone.system.performanceMode = nextMode;
      systemStore.updateSettings(clone);
    } else if (tab === 'system' && row === 3) {
      clone.display.vsync = !clone.display.vsync;
      systemStore.updateSettings(clone);
    } else if (tab === 'audio' && row === 0) {
      clone.audio.uiSoundEffects = !clone.audio.uiSoundEffects;
      soundFx.setEnabled(clone.audio.uiSoundEffects);
      systemStore.updateSettings(clone);
    } else if (tab === 'gamepad' && row === 0) {
      clone.gamepad.vibration = !clone.gamepad.vibration;
      systemStore.updateSettings(clone);
    } else if (tab === 'gamepad' && row >= 2) {
      // Test vibration on physical gamepad
      const padIdx = row - 2;
      try {
        const pads = navigator.getGamepads ? navigator.getGamepads() : [];
        const pad = pads[padIdx];
        if (pad && (pad as any).vibrationActuator) {
          (pad as any).vibrationActuator.playEffect('dual-rumble', {
            startDelay: 0,
            duration: 300,
            weakMagnitude: 0.8,
            strongMagnitude: 0.8
          });
        }
      } catch {
        // ignore
      }
    }
  };

  const handleAdjustCurrentSlider = (delta: number) => {
    const settings = systemStore.settings();
    if (!settings) return;
    const tab = activeSettingsTab();
    const row = settingsRowIndex();
    const clone = JSON.parse(JSON.stringify(settings));

    if (tab === 'audio' && row === 1) {
      clone.audio.masterVolume = Math.min(100, Math.max(0, clone.audio.masterVolume + delta));
      systemStore.updateSettings(clone);
      soundFx.playMove();
    }
  };

  // Emulator CRUD Handlers
  const handleSaveEmulator = (emulator: Emulator) => {
    const emus = [...systemStore.emulators()];
    const existingIdx = emus.findIndex(e => e.id === emulator.id);
    if (existingIdx >= 0) {
      emus[existingIdx] = emulator;
    } else {
      emus.push(emulator);
    }
    systemStore.setEmulators(emus);
    soundFx.playFavorite();
  };

  const handleDeleteEmulator = (emulatorId: string) => {
    const emus = systemStore.emulators().filter(e => e.id !== emulatorId);
    systemStore.setEmulators(emus);
    soundFx.playBack();
  };

  // 3. Composable Logic Hooks
  const { launchWithEmulator } = useGameLauncher({ backend, modalStore, soundFx });

  const { handleAction } = useConsoleNavigation({
    navigationStore,
    libraryStore,
    systemStore,
    modalStore,
    soundFx,
    platformGames,
    focusedGame,
    activeSettingsTab,
    onSettingsTabChange: (tab) => {
      setActiveSettingsTab(tab);
      setSettingsRowIndex(0);
    },
    settingsFocusArea,
    onSettingsFocusAreaChange: (area) => setSettingsFocusArea(area),
    settingsRowIndex,
    onSettingsRowIndexChange: (idx) => setSettingsRowIndex(idx),
    onToggleCurrentSetting: handleToggleCurrentSetting,
    onAdjustCurrentSlider: handleAdjustCurrentSlider,
    onEnterPlatform: () => {
      if (platformWheelHandle) {
        platformWheelHandle.triggerEnter();
      } else {
        navigationStore.setFocusedGameIndex(0);
        navigationStore.setLibraryViewMode('games');
      }
    },
    onExitPlatform: () => {
      navigationStore.setLibraryViewMode('wheel');
    }
  });

  const { inputStatus } = useConsoleInput({
    onAction: handleAction
  });

  // 4. Initial Dataset Bootstrap
  onMount(async () => {
    await systemStore.loadSystemData();
    try {
      const allGames = gamesDataset as unknown as Game[];
      mockBackend.setGames(allGames);
      await libraryStore.loadGames(allGames);
    } catch {
      await libraryStore.loadGames();
    }
    if (systemStore.settings()) {
      soundFx.setEnabled(systemStore.settings()!.audio.uiSoundEffects);
    }
  });

  return (
    <Shell
      ambientBackdropUrl={ambientBackdrop()}
      crtShaderEnabled={false}
    >
      <Header
        inputStatus={inputStatus()}
        totalGamesCount={libraryStore.games().length}
      />

      <Show when={navigationStore.currentSection() === 'library'}>
        <Show when={navigationStore.libraryViewMode() === 'wheel'}>
          <PlatformWheel
            ref={(handle) => { platformWheelHandle = handle; }}
            platforms={systemStore.platforms()}
            selectedIndex={navigationStore.wheelPlatformIndex()}
            onSelectPlatform={() => {
              soundFx.playSelect();
              navigationStore.setFocusedGameIndex(0);
              navigationStore.setLibraryViewMode('games');
            }}
            onSelectSection={(section) => {
              soundFx.playSelect();
              setSettingsFocusArea('sidebar');
              navigationStore.setCurrentSection(section);
            }}
            onNavigateIndex={(idx) => {
              soundFx.playMove();
              navigationStore.setWheelPlatformIndex(idx);
            }}
            getGamesCountForPlatform={(id) => libraryStore.games().filter(g => g.platform === id).length}
            getPreviewGamesForPlatform={(id) => libraryStore.games().filter(g => g.platform === id)}
          />
        </Show>

        <Show when={navigationStore.libraryViewMode() === 'games' && activePlatform()}>
          <PlatformGamesView
            platform={activePlatform()!}
            games={platformGames()}
            focusedIndex={navigationStore.focusedGameIndex()}
            onFocusIndex={(idx) => navigationStore.setFocusedGameIndex(idx)}
            onSelectGame={(g) => {
              soundFx.playSelect();
              modalStore.openEmulatorSelector(g);
            }}
            onBackToPlatforms={() => {
              soundFx.playBack();
              navigationStore.setLibraryViewMode('wheel');
            }}
            onToggleFavorite={(id) => {
              soundFx.playFavorite();
              libraryStore.toggleFavorite(id);
            }}
          />
        </Show>
      </Show>

      <Show when={navigationStore.currentSection() === 'settings'}>
        <SettingsView
          settings={systemStore.settings()}
          emulators={systemStore.emulators()}
          activeTab={activeSettingsTab()}
          focusArea={settingsFocusArea()}
          focusedRowIndex={settingsRowIndex()}
          onTabChange={(tab) => {
            setActiveSettingsTab(tab);
            setSettingsRowIndex(0);
          }}
          onUpdateSettings={(s) => {
            soundFx.setEnabled(s.audio.uiSoundEffects);
            systemStore.updateSettings(s);
          }}
          onSaveEmulator={handleSaveEmulator}
          onDeleteEmulator={handleDeleteEmulator}
          onBack={() => {
            soundFx.playBack();
            navigationStore.setCurrentSection('library');
            navigationStore.setLibraryViewMode('wheel');
          }}
        />
      </Show>

      <EmulatorSelectorModal
        game={modalStore.selectedGame()}
        emulators={systemStore.emulators()}
        isOpen={modalStore.isEmulatorSelectorOpen()}
        onClose={() => {
          soundFx.playBack();
          modalStore.closeEmulatorSelector();
        }}
        onConfirmLaunch={launchWithEmulator}
      />
    </Shell>
  );
};

export default App;
