import type { InputAction } from "@contracts/input.types";
import {
  SETTINGS_TABS,
  type UseSettingsNavigationOptions,
} from "@contracts/settings.types";

export function useSettingsNavigation(options: UseSettingsNavigationOptions) {
  return (action: InputAction) => {
    const tab = options.activeSettingsTab();
    const row = options.settingsRowIndex();
    if (options.settingsFocusArea() === "sidebar") {
      const index = SETTINGS_TABS.findIndex((entry) => entry.id === tab);
      switch (action) {
        case "BUTTON_B":
          options.soundFx.playBack();
          options.onBack();
          break;
        case "NAV_DOWN":
        case "BUTTON_RB":
          options.onSettingsTabChange(
            SETTINGS_TABS[(index + 1) % SETTINGS_TABS.length].id,
          );
          options.soundFx.playMove();
          break;
        case "NAV_UP":
        case "BUTTON_LB":
          options.onSettingsTabChange(
            SETTINGS_TABS[
              (index - 1 + SETTINGS_TABS.length) % SETTINGS_TABS.length
            ].id,
          );
          options.soundFx.playMove();
          break;
        case "NAV_RIGHT":
        case "BUTTON_A":
          options.onSettingsFocusAreaChange("content");
          options.onSettingsRowIndexChange(0);
          options.soundFx.playSelect();
          break;
        default:
          break;
      }
      return;
    }

    const slider = tab === "audio" && row === 1;
    if (action === "BUTTON_B" || (action === "NAV_LEFT" && !slider)) {
      options.onSettingsFocusAreaChange("sidebar");
      options.soundFx.playBack();
      return;
    }
    const count =
      tab === "system"
        ? 4
        : tab === "emulators"
          ? Math.max(1, options.emulatorCount())
          : tab === "gamepad"
            ? 6
            : tab === "stores"
              ? Math.max(1, options.storeProviderCount?.() ?? 3)
              : 2;
    switch (action) {
      case "NAV_DOWN":
      case "NAV_UP":
        options.onSettingsRowIndexChange(
          Math.max(
            0,
            Math.min(count - 1, row + (action === "NAV_DOWN" ? 1 : -1)),
          ),
        );
        options.soundFx.playMove();
        break;
      case "BUTTON_A":
        options.onToggleCurrentSetting();
        options.soundFx.playSelect();
        break;
      case "NAV_LEFT":
      case "NAV_RIGHT":
        if (slider)
          options.onAdjustCurrentSlider(action === "NAV_RIGHT" ? 5 : -5);
        break;
      default:
        break;
    }
  };
}
