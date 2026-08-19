import { Component, For } from 'solid-js';
import { SETTINGS_TABS } from '@contracts/settings.types';
import type { SettingsSidebarProps } from '@contracts/settings.types';

export const SettingsSidebar: Component<SettingsSidebarProps> = (props) => {
  return (
    <div class="settings-sidebar">
      <For each={SETTINGS_TABS}>
        {(tab) => {
          const isActive = () => props.activeTab === tab.id;
          const isFocused = () => isActive() && props.focusArea === 'sidebar';

          return (
            <button
              class={`settings-nav-btn ${isActive() ? 'active' : ''} ${isFocused() ? 'focused' : ''}`}
              onClick={() => props.onTabChange(tab.id)}
            >
              <span class="settings-tab-tag">{tab.tag}</span>
              <div class="settings-tab-meta">
                <span class="settings-tab-name">{tab.name}</span>
                <span class="settings-tab-desc">{tab.desc}</span>
              </div>
            </button>
          );
        }}
      </For>
    </div>
  );
};
