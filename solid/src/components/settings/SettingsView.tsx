import {
  Component,
  createEffect,
  onMount,
  onCleanup,
  Switch,
  Match,
} from "solid-js";
import type { SystemSettings } from "@contracts/game.types";
import type { SettingsViewProps } from "@contracts/settings.types";

// Subcomponents
import { SettingsSidebar } from "./SettingsSidebar";
import { SystemTab } from "./tabs/SystemTab";
import { AudioTab } from "./tabs/AudioTab";
import { GamepadTab } from "./tabs/GamepadTab";
import { StoresTab } from "./tabs/StoresTab";
import { useDisplayInfo } from "@hooks/useDisplayInfo";

// Animations
import {
  animateSettingsEntrance,
  animateTabTransition,
} from "@animations/settings-animations";

export const SettingsView: Component<SettingsViewProps> = (props) => {
  let rootContainerRef: HTMLDivElement | undefined;
  let contentPaneRef: HTMLDivElement | undefined;

  const currentTab = () => props.activeTab || "system";
  const display = useDisplayInfo(props.storeBackend);
  const isRowFocused = (row: number) =>
    props.focusArea === "content" && props.focusedRowIndex === row;

  onMount(() => {
    if (rootContainerRef) {
      animateSettingsEntrance(rootContainerRef);
    }
  });

  createEffect(() => {
    const tab = currentTab();
    if (contentPaneRef && tab) {
      animateTabTransition(contentPaneRef);
    }
  });

  // Auto-scroll when focused row changes
  createEffect(() => {
    const row = props.focusedRowIndex;
    if (props.focusArea === "content" && contentPaneRef && row !== undefined) {
      setTimeout(() => {
        const focusedEl = contentPaneRef?.querySelector(
          ".setting-card-row.focused, .settings-device-row.focused, .gamepad-device-card.focused, .store-provider-card.focused",
        ) as HTMLElement;
        if (focusedEl) {
          focusedEl.scrollIntoView({ behavior: "smooth", block: "nearest" });
        }
      }, 30);
    }
  });

  onMount(() =>
    props.onControllerReady?.((action) => {
      if (currentTab() === 'audio' && props.focusArea === 'content' && (props.focusedRowIndex ?? 0) < 2) {
        const select = contentPaneRef?.querySelector<HTMLSelectElement>(`[data-setting-row="${props.focusedRowIndex ?? 0}"] select`);
        if (select && !select.disabled && ['BUTTON_A', 'NAV_LEFT', 'NAV_RIGHT'].includes(action)) {
          if (action === 'BUTTON_A') select.focus();
          else {
            select.selectedIndex = Math.max(0, Math.min(select.options.length - 1, select.selectedIndex + (action === 'NAV_RIGHT' ? 1 : -1)));
            select.dispatchEvent(new Event('change', { bubbles: true }));
          }
          return;
        }
      }
      props.onNavigate?.(action);
    }),
  );
  onCleanup(() => props.onControllerReady?.(null));

  const handleUpdate = (updater: (s: SystemSettings) => void) => {
    if (props.settings && props.onUpdateSettings) {
      const clone = JSON.parse(JSON.stringify(props.settings));
      updater(clone);
      props.onUpdateSettings(clone);
    }
  };

  return (
    <div class="console-settings-container" ref={rootContainerRef}>
      {/* Main Settings Master-Detail Layout */}
      <div class="console-tabs-system">
        {/* Left Sidebar Tabs */}
        <SettingsSidebar
          activeTab={currentTab()}
          focusArea={props.focusArea}
          onTabChange={(tabId) => props.onTabChange?.(tabId)}
        />

        {/* Right Content Pane */}
        <div class="console-settings-content-pane" ref={contentPaneRef}>
          <Switch>
            <Match when={currentTab() === "system"}>
              <SystemTab
                settings={props.settings}
                emulators={props.emulators}
                displayInfo={display.info()}
                isRowFocused={isRowFocused}
                onSelectContentArea={props.onSelectContentArea}
                onUpdateSettings={handleUpdate}
              />
            </Match>

            <Match when={currentTab() === "audio"}>
              <AudioTab
                settings={props.settings}
                backend={props.storeBackend}
                isRowFocused={isRowFocused}
                onSelectContentArea={props.onSelectContentArea}
                onUpdateSettings={handleUpdate}
              />
            </Match>

            <Match when={currentTab() === "gamepad"}>
              <GamepadTab
                settings={props.settings}
                isRowFocused={isRowFocused}
                onSelectContentArea={props.onSelectContentArea}
                onUpdateSettings={handleUpdate}
              />
            </Match>

            <Match when={currentTab() === "stores"}>
              <StoresTab
                backend={props.storeBackend}
                isRowFocused={isRowFocused}
                onSelectContentArea={props.onSelectContentArea}
              />
            </Match>
          </Switch>
        </div>
      </div>

    </div>
  );
};

export default SettingsView;
