import { Component, createSignal, createEffect, onMount, Switch, Match, Show } from 'solid-js';
import type { Emulator, SystemSettings } from '@contracts/game.types';
import type { SettingsViewProps } from '@contracts/settings.types';

// Subcomponents
import { SettingsSidebar } from './SettingsSidebar';
import { SystemTab } from './tabs/SystemTab';
import { EmulatorsTab } from './tabs/EmulatorsTab';
import { AudioTab } from './tabs/AudioTab';
import { GamepadTab } from './tabs/GamepadTab';
import { UpdateTab } from './tabs/UpdateTab';
import { EmulatorCrudModal } from './modals/EmulatorCrudModal';

// Animations
import { animateSettingsEntrance, animateTabTransition } from '@animations/settings-animations';

export const SettingsView: Component<SettingsViewProps> = (props) => {
  const [selectedEmulatorForEdit, setSelectedEmulatorForEdit] = createSignal<Emulator | null>(null);
  const [isCrudModalOpen, setIsCrudModalOpen] = createSignal<boolean>(false);

  let rootContainerRef: HTMLDivElement | undefined;
  let contentPaneRef: HTMLDivElement | undefined;

  const currentTab = () => props.activeTab || 'system';
  const isRowFocused = (row: number) => props.focusArea === 'content' && props.focusedRowIndex === row;

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
    if (props.focusArea === 'content' && contentPaneRef && row !== undefined) {
      setTimeout(() => {
        const focusedEl = contentPaneRef?.querySelector('.setting-card-row.focused, .cyber-emulator-blade.focused, .gamepad-device-card.focused, .update-hero-blade.focused') as HTMLElement;
        if (focusedEl) {
          focusedEl.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        }
      }, 30);
    }
  });

  const openEditEmulatorModal = (emu?: Emulator) => {
    setSelectedEmulatorForEdit(emu || null);
    setIsCrudModalOpen(true);
  };

  createEffect(() => {
    (window as any).__EMUBOX_OPEN_EMULATOR_CONFIG__ = (idx: number) => {
      const list = props.emulators || [];
      openEditEmulatorModal(list[idx] || list[0]);
    };
  });

  const handleUpdate = (updater: (s: SystemSettings) => void) => {
    if (props.settings && props.onUpdateSettings) {
      const clone = JSON.parse(JSON.stringify(props.settings));
      updater(clone);
      props.onUpdateSettings(clone);
    }
  };

  return (
    <div class="console-settings-container" ref={rootContainerRef}>
      {/* Top Header Bar with Back Button */}
      <div class="settings-top-bar">
        <Show when={props.onBack}>
          <button class="settings-back-pill" onClick={props.onBack}>
            <span class="back-key-badge">B</span>
            <span class="back-label">MENÚ PRINCIPAL</span>
          </button>
        </Show>
        <div class="settings-screen-title">AJUSTES DEL SISTEMA</div>
        <div class="settings-top-clock">EmuBox OS • Arch Linux</div>
      </div>

      {/* Main Settings Master-Detail Layout */}
      <div class="settings-layout-grid">
        {/* Left Sidebar Tabs */}
        <SettingsSidebar
          activeTab={currentTab()}
          focusArea={props.focusArea}
          onTabChange={(tabId) => props.onTabChange?.(tabId)}
        />

        {/* Right Content Pane */}
        <div class="settings-content-pane" ref={contentPaneRef}>
          <Switch>
            <Match when={currentTab() === 'system'}>
              <SystemTab
                settings={props.settings}
                isRowFocused={isRowFocused}
                onSelectContentArea={props.onSelectContentArea}
                onUpdateSettings={handleUpdate}
              />
            </Match>

            <Match when={currentTab() === 'emulators'}>
              <EmulatorsTab
                emulators={props.emulators}
                isRowFocused={isRowFocused}
                onSelectContentArea={props.onSelectContentArea}
                onOpenEditModal={openEditEmulatorModal}
              />
            </Match>

            <Match when={currentTab() === 'audio'}>
              <AudioTab
                settings={props.settings}
                isRowFocused={isRowFocused}
                onSelectContentArea={props.onSelectContentArea}
                onUpdateSettings={handleUpdate}
              />
            </Match>

            <Match when={currentTab() === 'gamepad'}>
              <GamepadTab
                settings={props.settings}
                isRowFocused={isRowFocused}
                onSelectContentArea={props.onSelectContentArea}
                onUpdateSettings={handleUpdate}
              />
            </Match>

            <Match when={currentTab() === 'update'}>
              <UpdateTab
                settings={props.settings}
                updateInfo={props.updateInfo}
                isRowFocused={isRowFocused}
                onSelectContentArea={props.onSelectContentArea}
                onUpdateSettings={handleUpdate}
                onCheckUpdates={props.onCheckUpdates}
                onApplyUpdate={props.onApplyUpdate}
              />
            </Match>
          </Switch>
        </div>
      </div>

      {/* CRUD Modal for Emulator / Core Management */}
      <EmulatorCrudModal
        isOpen={isCrudModalOpen()}
        initialData={selectedEmulatorForEdit()}
        onClose={() => setIsCrudModalOpen(false)}
        onSave={(emu) => props.onSaveEmulator?.(emu)}
        onDelete={(id) => props.onDeleteEmulator?.(id)}
      />
    </div>
  );
};

export default SettingsView;
