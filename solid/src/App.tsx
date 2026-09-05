import { Component, createMemo, onMount, onCleanup, createSignal, Show } from 'solid-js';
import { listen } from '@tauri-apps/api/event';

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
import { GameLibraryView } from '@components/library/GameLibraryView';
import { EmulatorSelectorModal } from '@components/modals/EmulatorSelectorModal';
import { SettingsView } from '@components/settings/SettingsView';
import { MaintenanceModal } from '@components/modals/MaintenanceModal';
import { DownloadSourceModal } from '@components/modals/DownloadSourceModal';

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

  const handleGameActivate = (game: Game) => {
    if (game.installed) {
      soundFx.playSelect();
      modalStore.openEmulatorSelector(game);
    } else {
      soundFx.playSelect();
      void libraryStore.openSources(game);
    }
  };

  const [activeSettingsTab, setActiveSettingsTab] = createSignal<string>('system');
  const confirmSource = () => {
    const game = libraryStore.sourceGame();
    const source = libraryStore.sourceOptions()[libraryStore.sourceIndex()];
    if (!game || !source?.downloadable) return;
    libraryStore.closeSources();
    void libraryStore.downloadGame(game.id, source.id);
  };
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

  const [selectedPlatform, setSelectedPlatform] = createSignal<string>('all');

  // 2. Settings Controller (Business Logic & OTA Lifecycle)
  const {
    updateInfo,
    setUpdateInfo,
    handleCheckUpdates,
    handleApplyUpdate,
    handleSaveEmulator,
    handleDeleteEmulator,
    handleToggleCurrentSetting,
    handleAdjustCurrentSlider
  } = useSettingsController({
    systemStore,
    soundFx,
    backend,
    activeSettingsTab,
    settingsRowIndex
  });

  // 3. Computed View Memos
  const platformGames = createMemo(() => {
    const all = libraryStore.games();
    const platform = selectedPlatform();
    return platform === 'all' ? all : all.filter((game) => game.platform === platform);
  });

  const focusedGame = createMemo(() => {
    const list = platformGames();
    const idx = navigationStore.focusedGameIndex();
    return list.length === 0 ? null : (list[idx] || list[list.length - 1] || null);
  });

  const ambientBackdrop = createMemo(() => {
    const game = focusedGame();
    if (game) return game.backdropImage || game.coverImage;
    return '';
  });

  // 4. Composable Logic Hooks
  const { launchWithEmulator } = useGameLauncher({ backend, systemStore, modalStore, soundFx });

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
    onEnterPlatform: () => navigationStore.setLibraryViewMode('games'),
    onExitPlatform: () => {
      navigationStore.setLibraryViewMode('games');
    },
    onSelectGame: (g) => {
      handleGameActivate(g);
    }
  });

  const { inputStatus } = useConsoleInput({
    onAction: action => {
      if (libraryStore.sourceGame()) {
        if (action === 'BUTTON_B') libraryStore.closeSources();
        else if (action === 'BUTTON_A') confirmSource();
        else if (action === 'NAV_DOWN' || action === 'NAV_UP') {
          libraryStore.setSourceIndex(Math.max(0, Math.min(libraryStore.sourceOptions().length - 1,
            libraryStore.sourceIndex() + (action === 'NAV_DOWN' ? 1 : -1))));
        }
        return;
      }
      handleAction(action);
    }
  });

  // 5. Initial Dataset Bootstrap
  let unlistenLibraryUpdated: (() => void) | undefined;
  let disposed = false;
  onCleanup(() => {
    disposed = true;
    unlistenLibraryUpdated?.();
  });
  onMount(async () => {
    // Sonda de diagnóstico: confirma si el puente IPC de Tauri existe en este webview.
    try {
      const internals = (window as any).__TAURI_INTERNALS__;
      const globalTauri = (window as any).__TAURI__;
      const probeInfo = JSON.stringify({
        hasInternals: !!internals,
        hasGlobalTauri: !!globalTauri,
        invokeType: typeof internals?.invoke,
        isTauriEnvironment: backend.isTauriEnvironment
      });
      if (internals?.invoke) {
        await internals.invoke('frontend_probe', { message: probeInfo });
      } else {
        console.error('[EmuBox] Puente Tauri no detectado:', probeInfo);
      }
    } catch (probeError) {
      console.error('[EmuBox] Sonda de diagnóstico falló:', probeError);
    }

    if (backend.isTauriEnvironment) {
      try {
        const unlisten = await listen('library-updated', () => {
          void libraryStore.loadGames().catch(error => console.error('[Library]', error));
        });
        if (disposed) unlisten();
        else unlistenLibraryUpdated = unlisten;
      } catch (error) {
        console.error('[Library] No se pudo suscribir a cambios', error);
      }
    }

    await systemStore.loadSystemData().catch(error => console.error('[System]', error));
    try {
      if (backend.isTauriEnvironment) {
        // Escanear y cargar juegos reales del sistema Linux (/var/lib/emubox/games)
        await backend.scanGames();
        await libraryStore.loadGames();
      } else {
        // Entorno de desarrollo en navegador web sin runtime Tauri
        await libraryStore.loadGames();
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
        currentSection={navigationStore.currentSection() === 'settings' ? 'settings' : 'library'}
        onNavigate={(section) => {
          navigationStore.setCurrentSection(section);
          if (section === 'library') navigationStore.setLibraryViewMode('games');
        }}
      />

      <Show when={navigationStore.currentSection() === 'library'}>
        <GameLibraryView
            games={libraryStore.games()}
            platforms={systemStore.platforms()}
            emulators={systemStore.emulators()}
            selectedPlatform={selectedPlatform()}
            focusedIndex={navigationStore.focusedGameIndex()}
            onSelectPlatform={(platform) => {
              soundFx.playSelect();
              setSelectedPlatform(platform);
              navigationStore.setFocusedGameIndex(0);
            }}
            onFocusIndex={(idx) => navigationStore.setFocusedGameIndex(idx)}
            downloadingIds={libraryStore.downloadingIds()}
            downloadError={libraryStore.downloadError()}
            onSelectGame={handleGameActivate}
            onDownloadGame={(game) => { void libraryStore.openSources(game); }}
            onToggleFavorite={(id) => {
              soundFx.playFavorite();
              libraryStore.toggleFavorite(id);
            }}
        />
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
            navigationStore.setLibraryViewMode('games');
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

      <DownloadSourceModal store={libraryStore} onConfirm={confirmSource} />

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
