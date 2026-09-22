import { createSignal, onCleanup, onMount } from "solid-js";
import type { InputActionListener } from "@contracts/input.types";
import type { SystemSettings } from "@contracts/game.types";
import { DevelopmentBackendService } from "@services/backend/development-backend.service";
import { SoundFxService } from "@services/audio/sound-fx.service";
import { ViewportService } from "@services/system/viewport.service";
import { createLibraryStore } from "@stores/library.store";
import { createSystemStore } from "@stores/system.store";
import { useConsoleInput } from "@hooks/useConsoleInput";
import { useSettingsController } from "@hooks/useSettingsController";
import { useSettingsNavigation } from "@hooks/useSettingsNavigation";

export function useDevelopmentPreview() {
  const backend = new DevelopmentBackendService();
  const library = createLibraryStore(backend);
  const system = createSystemStore(backend);
  const sound = new SoundFxService();
  sound.setEnabled(false);
  const viewport = ViewportService.getInstance();
  onCleanup(() => viewport.destroy());
  const [section, setSection] = createSignal<"library" | "settings">("library");
  const [activeTab, setActiveTab] = createSignal("system");
  const [focusArea, setFocusArea] = createSignal<"sidebar" | "content">(
    "sidebar",
  );
  const [row, setRow] = createSignal(0);
  const [notice, setNotice] = createSignal("");
  let libraryController: InputActionListener | null = null;
  let settingsController: InputActionListener | null = null;
  let sourcesController: InputActionListener | null = null;

  const settings = useSettingsController({
    systemStore: system,
    soundFx: sound,
    activeSettingsTab: activeTab,
    settingsRowIndex: row,
  });
  const changeTab = (tab: string) => {
    setActiveTab(tab);
    setRow(0);
  };
  const back = () => setSection("library");
  const navigateSettings = useSettingsNavigation({
    soundFx: sound,
    emulatorCount: () => system.emulators().length,
    activeSettingsTab: activeTab,
    settingsFocusArea: focusArea,
    settingsRowIndex: row,
    onSettingsTabChange: changeTab,
    onSettingsFocusAreaChange: setFocusArea,
    onSettingsRowIndexChange: setRow,
    onToggleCurrentSetting: settings.handleToggleCurrentSetting,
    onAdjustCurrentSlider: settings.handleAdjustCurrentSlider,
    onBack: back,
  });
  const blockedAction = () =>
    setNotice("Accion nativa deshabilitada en desarrollo.");
  const { inputStatus } = useConsoleInput({
    onAction: (action) => {
      if (library.sourceGame()) sourcesController?.(action);
      else if (action === "MAINTENANCE_MENU") blockedAction();
      else if (section() === "settings") settingsController?.(action);
      else libraryController?.(action);
    },
  });

  onMount(() => {
    void Promise.all([
      backend.getGames().then((games) => library.loadGames(games)),
      system.loadSystemData(),
    ]).catch(() => setNotice("No se pudo preparar la previsualizacion."));
  });

  return {
    backend,
    library,
    system,
    settings,
    section,
    activeTab,
    focusArea,
    row,
    inputStatus,
    notice,
    blockedAction,
    navigateSettings,
    changeTab,
    back,
    selectContent: () => setFocusArea("content"),
    openSettings: (tab: string) => {
      changeTab(tab);
      setFocusArea("content");
      setSection("settings");
    },
    updateSettings: (value: SystemSettings) => {
      void system.updateSettings(value);
    },
    toggleFavorite: (id: string) => {
      void library.toggleFavorite(id);
    },
    registerLibrary: (handler: InputActionListener | null) => {
      libraryController = handler;
    },
    registerSettings: (handler: InputActionListener | null) => {
      settingsController = handler;
    },
    registerSources: (handler: InputActionListener | null) => {
      sourcesController = handler;
    },
  };
}
