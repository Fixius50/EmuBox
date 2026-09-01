import { Component, createMemo, onMount, onCleanup, createSignal, Show } from 'solid-js';

// Types
import type { Game } from '@contracts/game.types';

// Services
import { MockBackendService } from '@services/backend/mock-backend.service';
import { TauriBackendService } from '@services/backend/tauri-backend.service';
import { SoundFxService } from '@services/audio/sound-fx.service';
import { GraphicsDetectorService } from '@services/graphics/graphics-detector.service';
import { ViewportService } from '@services/system/viewport.service';

// Stores
import { createLibraryStore } from '@stores/library.store';
import { createSystemStore } from '@stores/system.store';
import { createNavigationStore } from '@stores/navigation.store';
import { createModalStore } from '@stores/modal.store';

// Hooks & Controllers
import { useConsoleInput } from '@hooks/useConsoleInput';
import { useConsoleNavigation } from '@hooks/useConsoleNavigation';
import { useGameLauncher } from '@hooks/useGameLauncher';
import { useSettingsController } from '@hooks/useSettingsController';

// Components
import { Shell } from '@components/layout/Shell';
import { Header } from '@components/layout/Header';
import { PlatformWheel, PlatformWheelHandle } from '@components/platforms/PlatformWheel';
import { PlatformGamesView } from '@components/library/PlatformGamesView';
import { EmulatorSelectorModal } from '@components/modals/EmulatorSelectorModal';
import { SettingsView } from '@components/settings/SettingsView';
import { MaintenanceModal } from '@components/modals/MaintenanceModal';

// Initial Mock Dataset
import gamesDataset from '@data/games-10000.json';

export const App: Component = () => {
  // 1. Singletons & Stores Initialization
  const viewport = ViewportService.getInstance();
  onCleanup(() => viewport.destroy());

  const graphicsDetector = new GraphicsDetectorService();
  graphicsDetector.detect();

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

  // Global hotkey: F12 (o Ctrl+Q) cierra la interfaz gráfica y sale a la consola Linux (TTY1)
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (
        e.key === 'F12' ||
        (e.ctrlKey && e.key.toLowerCase() === 'q') ||
        (e.ctrlKey && e.altKey && e.key.toLowerCase() === 't')
      ) {
        e.preventDefault();
        backend.exitToLinuxShell();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    onCleanup(() => window.removeEventListener('keydown', handleKeyDown));
  });

  let platformWheelHandle: PlatformWheelHandle | undefined;

  // 2. Settings Controller (Business Logic & OTA Lifecycle)
  const {
    updateInfo,
    setUpdateInfo,
    handleCheckUpdates,
    handleApplyUpdate,
    handleSaveEmulator,
    handleDeleteEmulator,
    handleToggleCurrentSetting,
    handleAdjustCurrentSlider,
    triggerVibrationTest
  } = useSettingsController({
    systemStore,
    soundFx,
    backend,
    activeSettingsTab,
    settingsRowIndex
  });

  // 3. Computed View Memos
  const activePlatform = createMemo(() => {
    const list = systemStore.platforms();
    return list.length === 0 ? null : (list[navigationStore.wheelPlatformIndex()] || list[0] || null);
  });

  const platformGames = createMemo(() => {
    const all = libraryStore.games();
    const plat = activePlatform();
    return plat ? all.filter((g) => g.platform === plat.id) : all;
  });

  const focusedGame = createMemo(() => {
    const list = platformGames();
    const idx = navigationStore.focusedGameIndex();
    return list.length === 0 ? null : (list[idx] || list[list.length - 1] || null);
  });

  const ambientBackdrop = createMemo(() => {
    if (navigationStore.libraryViewMode() === 'games') {
      const g = focusedGame();
      if (g) return g.backdropImage || g.coverImage;
    }
    return '';
  });

  // 4. Composable Logic Hooks
  const { launchWithEmulator, launchGameDirect } = useGameLauncher({ backend, systemStore, modalStore, soundFx });

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
    },
    onSelectGame: (g) => {
      launchGameDirect(g);
    }
  });

  const { inputStatus } = useConsoleInput({
    onAction: handleAction
  });

  // 5. Initial Dataset Bootstrap
  onMount(async () => {
    await systemStore.loadSystemData();
    try {
      if (backend.isTauriEnvironment) {
        // Escanear y cargar juegos reales del sistema Linux (/var/lib/emubox/games)
        await backend.scanGames();
        await libraryStore.loadGames();
      } else {
        // Entorno de desarrollo en navegador web sin runtime Tauri
        const realGames = await backend.getGames();
        if (realGames && realGames.length > 0) {
          await libraryStore.loadGames(realGames);
        } else {
          const allGames = gamesDataset as unknown as Game[];
          mockBackend.setGames(allGames);
          await libraryStore.loadGames(allGames);
        }
      }
    } catch {
      await libraryStore.loadGames();
    }

    if (systemStore.settings()) {
      soundFx.setEnabled(systemStore.settings()!.audio.uiSoundEffects);
    }

    try {
      setUpdateInfo(await backend.getUpdateInfo());
    } catch {
      // Ignored if backend not ready
    }
  });

  return (
    <Shell ambientBackdropUrl={ambientBackdrop()} crtShaderEnabled={false}>
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
            getGamesCountForPlatform={(id) => libraryStore.games().filter((g) => g.platform === id).length}
            getPreviewGamesForPlatform={(id) => libraryStore.games().filter((g) => g.platform === id)}
          />
        </Show>

        <Show when={navigationStore.libraryViewMode() === 'games' && activePlatform()}>
          <PlatformGamesView
            platform={activePlatform()!}
            games={platformGames()}
            focusedIndex={navigationStore.focusedGameIndex()}
            onFocusIndex={(idx) => navigationStore.setFocusedGameIndex(idx)}
            onSelectGame={(g) => {
              launchGameDirect(g);
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
          updateInfo={updateInfo()}
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
          onCheckUpdates={handleCheckUpdates}
          onApplyUpdate={handleApplyUpdate}
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

      <MaintenanceModal
        isOpen={modalStore.isMaintenanceOpen()}
        onClose={() => {
          soundFx.playBack();
          modalStore.closeMaintenance();
        }}
        backend={backend}
        focusedIndex={modalStore.maintenanceIndex()}
        onSelectIndex={(idx) => modalStore.setMaintenanceIndex(idx)}
      />
    </Shell>
  );
};

export default App;
