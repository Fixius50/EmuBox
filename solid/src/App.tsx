import { Component, onMount, onCleanup, createSignal, Show } from "solid-js";
import { listen } from "@tauri-apps/api/event";

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
        isTauriEnvironment: backend.isTauriEnvironment,
      });
      if (internals?.invoke) {
        await internals.invoke("frontend_probe", { message: probeInfo });
      } else {
        console.error("[EmuBox] Puente Tauri no detectado:", probeInfo);
      }
    } catch (probeError) {
      console.error("[EmuBox] Sonda de diagnóstico falló:", probeError);
    }

    if (backend.isTauriEnvironment) {
      try {
        const unlisten = await listen("library-updated", () => {
          void libraryStore
            .loadGames()
            .catch((error) => console.error("[Library]", error));
        });
        if (disposed) unlisten();
        else unlistenLibraryUpdated = unlisten;
      } catch (error) {
        console.error("[Library] No se pudo suscribir a cambios", error);
      }
    }

    if (backend.isTauriEnvironment) {
      await backend
        .getHardwareInfo()
        .then((hardware) => graphicsDetector.detectFromHardware(hardware))
        .catch((error) =>
          console.error("[Graphics] No se pudo consultar el hardware", error),
        );
    }
    await systemStore
      .loadSystemData()
      .catch((error) => console.error("[System]", error));
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
  });

  return (
    <div class="emubox-xmb-root">
      <div
        hidden={navigationStore.currentSection() !== "library"}
        class="xmb-library-layer"
      >
        <XmbLibrary
          games={libraryStore.catalogGames()}
          platforms={systemStore.platforms()}
          downloadingIds={libraryStore.catalogDownloadingIds()}
          loading={libraryStore.isLoading()}
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
