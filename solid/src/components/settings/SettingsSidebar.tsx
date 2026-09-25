import { Component, For } from "solid-js";
import { SETTINGS_TABS } from "@contracts/settings.types";
import type { SettingsSidebarProps } from "@contracts/settings.types";
import { Gamepad2, Monitor, Settings2, Store, Volume2, Server } from "lucide-solid";

export const SettingsSidebar: Component<SettingsSidebarProps> = (props) => {
  return (
    <div class="console-sidebar-menu">
      <For each={SETTINGS_TABS}>
        {(tab) => {
          const isActive = () => props.activeTab === tab.id;
          const isFocused = () => isActive() && props.focusArea === "sidebar";

          return (
            <button
              class={`sidebar-tab-trigger ${isActive() ? "active" : ""} ${isFocused() ? "focused-sidebar" : ""}`}
              onClick={() => props.onTabChange(tab.id)}
            >
              <span class="tab-badge-icon">
                {tab.id === "system" ? (
                  <Monitor size={21} />
                ) : tab.id === "audio" ? (
                  <Volume2 size={21} />
                ) : tab.id === "gamepad" ? (
                  <Gamepad2 size={21} />
                ) : tab.id === "services" ? (
                  <Server size={21} />
                ) : tab.id === "stores" ? (
                  <Store size={21} />
                ) : (
                  <Settings2 size={21} />
                )}
              </span>
              <div class="tab-label-group">
                <span class="tab-main-text">{tab.name}</span>
                <span class="tab-sub-text">{tab.desc}</span>
              </div>
              {isActive() && <div class="tab-neon-caret" />}
            </button>
          );
        }}
      </For>
    </div>
  );
};
