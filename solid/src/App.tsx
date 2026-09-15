import { Component, onMount, onCleanup, createSignal, Show, batch } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import { LoaderCircle, LogOut } from 'lucide-solid';
import { startupErrorMessage, startupMessage, waitForStartup } from '@services/system/startup';
import type { StartupReport } from '@contracts/startup.types';

// Types
import type { Game } from "@contracts/game.types";
import type { InputAction } from "@contracts/input.types";
import { gameBlockReason } from "@services/compatibility/launch-capability";

// Services
import { TauriBackendService } from "@services/backend/tauri-backend.service";
import { SoundFxService } from "@services/audio/sound-fx.service";
import { GraphicsDetectorService } from "@services/graphics/graphics-detector.service";
import { ViewportService } from "@services/system/viewport.service";

// Stores
import { createLibraryStore } from "@stores/library.store";
import { createSystemStore } from "@stores/system.store";
import { createNavigationStore } from "@stores/navigation.store";
import { createModalStore } from "@stores/modal.store";

// Hooks & Controllers
import { useConsoleInput } from "@hooks/useConsoleInput";
import { useSettingsNavigation } from "@hooks/useSettingsNavigation";
import { useGameLauncher } from "@hooks/useGameLauncher";
import { useSettingsController } from "@hooks/useSettingsController";

// Components
import { XmbLibrary } from "@components/library/XmbLibrary";
import { EmulatorSelectorModal } from "@components/modals/EmulatorSelectorModal";
import { SettingsView } from "@components/settings/SettingsView";
import { MaintenanceModal } from "@components/modals/MaintenanceModal";
import { DownloadSourceModal } from "@components/modals/DownloadSourceModal";

const NativeApp: Component = () => {
  // 1. Singletons & Stores Initialization
  const viewport = ViewportService.getInstance();
  onCleanup(() => viewport.destroy());

  const graphicsDetector = new GraphicsDetectorService();

  const backend = new TauriBackendService();
  const soundFx = new SoundFxService();

  const libraryStore = createLibraryStore(backend);
  const systemStore = createSystemStore(backend);
  const navigationStore = createNavigationStore();
  const modalStore = createModalStore();
  const [startupStatus, setStartupStatus] = createSignal('Preparando EmuBox...');
  const [startupError, setStartupError] = createSignal('');
  const [startupReady, setStartupReady] = createSignal(false);
  const [startupFailed, setStartupFailed] = createSignal(false);

  const handleGameActivate = (game: Game) => {
    soundFx.playSelect();
    void libraryStore.openSources(game);
  };
  let sourceController: ((action: InputAction) => void) | null = null;
  let xmbController: ((action: InputAction) => void) | null = null;
  let emulatorController: ((action: InputAction) => void) | null = null;
  let maintenanceController: ((action: InputAction) => void) | null = null;
  let settingsController: ((action: InputAction) => void) | null = null;

  const [activeSettingsTab, setActiveSettingsTab] =
    createSignal<string>("system");
  const confirmSource = () => {
    void libraryStore.confirmSource();
  };
  const [settingsFocusArea, setSettingsFocusArea] = createSignal<
    "sidebar" | "content"
  >("sidebar");
  const [settingsRowIndex, setSettingsRowIndex] = createSignal<number>(0);

  // Global hotkey: F12 (o Ctrl+Q) cierra la interfaz gráfica y sale a la consola Linux (TTY1)
  onMount(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (
        e.key === "F12" ||
        (e.ctrlKey && e.key.toLowerCase() === "q") ||
        (e.ctrlKey && e.altKey && e.key.toLowerCase() === "t")
      ) {
        e.preventDefault();
        backend.exitToLinuxShell();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    onCleanup(() => window.removeEventListener("keydown", handleKeyDown));
  });

  // 2. Settings Controller (Business Logic & OTA Lifecycle)
  const {
    handleSaveEmulator,
    handleDeleteEmulator,
    handleToggleCurrentSetting,
    handleAdjustCurrentSlider,
  } = useSettingsController({
    systemStore,
    soundFx,
    activeSettingsTab,
    settingsRowIndex,
  });

  // 4. Composable Logic Hooks
  const { launchWithEmulator, getPreferredEmulator, saveEmulatorPreference } = useGameLauncher({
    backend,
    systemStore,
    modalStore,
    soundFx,
  });

  const handleSettingsAction = useSettingsNavigation({
    soundFx,
    emulatorCount: () => systemStore.emulators().length,
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
    onBack: () => navigationStore.setCurrentSection("library"),
  });

  const { inputStatus } = useConsoleInput({
    onAction: (action) => {
      if (!startupReady()) return;
      if (action === "MAINTENANCE_MENU") {
        modalStore.openMaintenance();
        return;
      }
      if (modalStore.isMaintenanceOpen()) {
        maintenanceController?.(action);
        return;
      }
      if (libraryStore.sourceGame()) {
        sourceController?.(action);
        return;
      }
      if (modalStore.isEmulatorSelectorOpen()) {
        emulatorController?.(action);
        return;
      }
      if (navigationStore.currentSection() === "settings")
        settingsController?.(action);
      else xmbController?.(action);
    },
  });

  // 5. Initial Dataset Bootstrap
  let unlistenLibraryUpdated: (() => void) | undefined;
  let libraryRefreshTimer: ReturnType<typeof setTimeout> | undefined;
  let refreshingLibrary = false;
  let libraryDirty = false;
  let disposed = false;
  const refreshLibrary = async () => {
    libraryRefreshTimer = undefined;
    if (disposed || refreshingLibrary) return;
    if (!startupReady()) { libraryDirty = true; return; }
    if (libraryStore.isLoading()) {
      libraryRefreshTimer = setTimeout(refreshLibrary, 1000);
      return;
    }
    libraryDirty = false;
    refreshingLibrary = true;
    try {
      await libraryStore.loadGames();
    } catch (error) {
      console.error('[Library]', error);
    } finally {
      refreshingLibrary = false;
      if (libraryDirty && !disposed) libraryRefreshTimer = setTimeout(refreshLibrary, 1000);
    }
  };
  onCleanup(() => {
    disposed = true;
    clearTimeout(libraryRefreshTimer);
    unlistenLibraryUpdated?.();
  });
  onMount(() => {
    const controller = new AbortController();
    let frame: number | undefined;
    const failed = (error: unknown) => {
      if (disposed || controller.signal.aborted) return;
      setStartupError(startupErrorMessage(error));
      setStartupFailed(true);
      setStartupReady(false);
      controller.abort();
    };
    const timer = setTimeout(() => failed(new Error('La interfaz no pudo completar el arranque en 90 segundos')), 90000);
    onCleanup(() => {
      controller.abort();
      clearTimeout(timer);
      if (frame !== undefined) cancelAnimationFrame(frame);
    });
    const prepare = async () => {
      const unlisten = await listen("library-updated", () => {
          libraryDirty = true;
          if (startupReady() && !refreshingLibrary && libraryRefreshTimer === undefined) {
            libraryRefreshTimer = setTimeout(refreshLibrary, 1000);
          }
      });
      if (disposed || controller.signal.aborted) { unlisten(); return; }
      unlistenLibraryUpdated = unlisten;
      await waitForStartup(() => backend.getStartupStatus(),
        handler => listen<StartupReport>('startup-status', event => handler(event.payload)),
        report => setStartupStatus(startupMessage(report)), controller.signal);
      const data = await backend.getStartupData();
      if (disposed || controller.signal.aborted) return;
      setStartupStatus('Organizando biblioteca...');
      const hydrationStarted = performance.now();
      batch(() => {
        systemStore.setSettings(data.settings);
        systemStore.setPlatforms(data.platforms);
        systemStore.setEmulators(data.emulators);
        graphicsDetector.detectFromHardware(data.hardware);
      });
      await libraryStore.loadGames(data.games);
      if (disposed || controller.signal.aborted) return;
      soundFx.setEnabled(data.settings.audio.uiSoundEffects);
      await new Promise<void>(resolve => {
        frame = requestAnimationFrame(() => { frame = requestAnimationFrame(() => resolve()); });
      });
      if (disposed || controller.signal.aborted) return;
      setStartupReady(true);
      await new Promise<void>(resolve => {
        frame = requestAnimationFrame(() => { frame = requestAnimationFrame(() => resolve()); });
      });
      if (disposed || controller.signal.aborted) return;
      const report = await backend.startupFrontendReady();
      if (disposed || controller.signal.aborted) return;
      setStartupError(report.warnings.length ? `Inicio con avisos: ${report.warnings[0]}` : '');
      setStartupReady(true);
      setStartupStatus('');
      clearTimeout(timer);
      console.info(`[Startup] biblioteca preparada en ${(performance.now() - hydrationStarted).toFixed(0)} ms de hidratacion`);
      if (libraryDirty && libraryRefreshTimer === undefined) libraryRefreshTimer = setTimeout(refreshLibrary, 1000);
    };
    void prepare().catch(failed);
  });

  return (
    <div class="emubox-xmb-root">
      <Show when={!startupReady()}>
        <section class="emubox-startup" aria-label="Arranque de EmuBox">
          <h1>EmuBox</h1>
          <div role="status" aria-live="polite" aria-atomic="true">
            <Show when={!startupFailed()}><LoaderCircle class="xmb-loading-spinner" size={24} aria-hidden="true" /></Show>
            <p>{startupFailed() ? startupError() : startupStatus()}</p>
          </div>
          <Show when={startupFailed()}>
            <button onClick={() => { void backend.exitToLinuxShell(); }}><LogOut size={18} />Salir a consola</button>
          </Show>
        </section>
      </Show>
      <div
        hidden={navigationStore.currentSection() !== "library"}
        class="xmb-library-layer"
        style={{ visibility: startupReady() ? 'visible' : 'hidden' }}
        inert={!startupReady()}
        aria-hidden={!startupReady()}
      >
        <XmbLibrary
          games={libraryStore.catalogGames()}
          platforms={systemStore.platforms()}
          downloadingIds={libraryStore.catalogDownloadingIds()}
          loading={Boolean(startupStatus()) || libraryStore.isLoading()}
          loadingMessage={startupStatus() || 'Actualizando biblioteca...'}
          loadError={libraryStore.loadError() || startupError()}
          inputStatus={inputStatus()}
          error={libraryStore.downloadError()}
          onOpenGame={handleGameActivate}
          onOpenSettings={(tab) => {
            setActiveSettingsTab(tab);
            setSettingsFocusArea("content");
            setSettingsRowIndex(0);
            navigationStore.setCurrentSection("settings");
          }}
          onMaintenance={() => modalStore.openMaintenance()}
          onFavorite={(id) => {
            void libraryStore
              .toggleFavorite(id)
              .catch((error) => console.error("[Favorite]", error));
          }}
          onMove={() => soundFx.playMove()}
          onControllerReady={(handler) => {
            xmbController = handler;
          }}
        />
      </div>

      <Show when={navigationStore.currentSection() === "settings"}>
        <SettingsView
          onNavigate={handleSettingsAction}
          onControllerReady={(handler) => {
            settingsController = handler;
          }}
          settings={systemStore.settings()}
          emulators={systemStore.emulators()}
          activeTab={activeSettingsTab()}
          focusArea={settingsFocusArea()}
          focusedRowIndex={settingsRowIndex()}
          onSelectContentArea={() => setSettingsFocusArea("content")}
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
            navigationStore.setCurrentSection("library");
          }}
        />
      </Show>

      <EmulatorSelectorModal
        onControllerReady={(handler) => {
          emulatorController = handler;
        }}
        game={modalStore.selectedGame()}
        emulators={systemStore.emulators()}
        isOpen={modalStore.isEmulatorSelectorOpen()}
        onClose={() => {
          soundFx.playBack();
          modalStore.closeEmulatorSelector();
        }}
        onConfirmLaunch={launchWithEmulator}
        getPreferredEmulator={getPreferredEmulator}
        onSavePreference={saveEmulatorPreference}
      />

      <DownloadSourceModal
        store={libraryStore}
        onConfirm={confirmSource}
        onControllerReady={(handler) => {
          sourceController = handler;
        }}
        playBlockReason={
          libraryStore.sourceGame()
            ? gameBlockReason(
                libraryStore.sourceGame()!,
                systemStore.emulators(),
              )
            : null
        }
        onPlay={(game) => {
          libraryStore.closeSources();
          modalStore.openEmulatorSelector(game);
        }}
      />

      <MaintenanceModal
        onControllerReady={(handler) => {
          maintenanceController = handler;
        }}
        isOpen={modalStore.isMaintenanceOpen()}
        onClose={() => {
          soundFx.playBack();
          modalStore.closeMaintenance();
        }}
        backend={backend}
        focusedIndex={modalStore.maintenanceIndex()}
        onSelectIndex={(idx) => modalStore.setMaintenanceIndex(idx)}
      />
    </div>
  );
};

export const App: Component = () => (
  <Show when={new TauriBackendService().isTauriEnvironment} fallback={
    <main class="native-runtime-required" role="alert">
      <h1>EmuBox</h1><p>Runtime nativo Tauri no disponible.</p>
    </main>
  }><NativeApp /></Show>
);

export default App;
