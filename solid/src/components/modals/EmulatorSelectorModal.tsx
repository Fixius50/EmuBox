import { Component, For, Show, createSignal, createEffect, onMount, onCleanup } from 'solid-js';
import { Dialog } from '@kobalte/core/dialog';
import { animateModalOpen } from '@animations/modal-animations';
import type { EmulatorSelectorModalProps } from '@contracts/modal.types';
import { emulatorBlockReason } from '@services/compatibility/launch-capability';
import type { InputAction } from '@contracts/input.types';
import { ConsoleHardwareVisual } from '@components/common/ConsoleHardwareVisual';
import { ArrowLeft, Play } from 'lucide-solid';

export const EmulatorSelectorModal: Component<EmulatorSelectorModalProps & { onControllerReady?: (handler: ((action: InputAction) => void) | null) => void }> = (props) => {
  const [selectedEmulatorId, setSelectedEmulatorId] = createSignal<string>('');
  let modalContentRef!: HTMLDivElement;

  const compatibleEmulators = () => {
    if (!props.game) return [];
    const list = props.emulators.filter(e => e.supportedPlatforms.includes(props.game!.platform));
    return list;
  };

  const selectedEmulator = () => {
    const list = compatibleEmulators();
    const id = selectedEmulatorId();
    if (id) {
      const found = list.find(e => e.id === id);
      if (found) return found;
    }
    return list.find(emulator => !emulatorBlockReason(emulator)) || list[0] || null;
  };

  const controller = (action: InputAction) => {
    if (!props.isOpen) return;
    const entries = compatibleEmulators();
    const current = Math.max(0, entries.findIndex(entry => entry.id === selectedEmulator()?.id));
    if (action === 'NAV_DOWN' || action === 'NAV_UP') {
      setSelectedEmulatorId(entries[Math.max(0, Math.min(entries.length - 1, current + (action === 'NAV_DOWN' ? 1 : -1)))]?.id || '');
    } else if (action === 'BUTTON_B') props.onClose();
    else if (action === 'BUTTON_A') {
      const emulator = selectedEmulator();
      if (props.game && emulator && !emulatorBlockReason(emulator)) props.onConfirmLaunch(props.game, emulator);
    }
  };
  onMount(() => props.onControllerReady?.(controller));
  onCleanup(() => props.onControllerReady?.(null));

  createEffect(() => {
    if (props.isOpen) {
      setTimeout(() => {
        if (modalContentRef) {
          animateModalOpen(modalContentRef);
        }
      }, 10);
    }
  });

  return (
    <Dialog open={props.isOpen} onOpenChange={(open) => { if (!open) props.onClose(); }}>
      <Dialog.Portal>
        <Dialog.Overlay class="console-modal-backdrop" />
        <div class="console-modal-center-container">
          <Dialog.Content class="console-game-blade emulator-selector-blade" ref={modalContentRef} onKeyDown={event => {
            event.stopPropagation();
            const action = ({ ArrowDown: 'NAV_DOWN', ArrowUp: 'NAV_UP', Escape: 'BUTTON_B' } as const)[event.key];
            if (action) { event.preventDefault(); controller(action); }
          }}>
            <Show when={props.game}>
              {(g) => (
                <div class="emulator-selector-layout">
                  {/* Left Game Preview */}
                  <div class="selector-game-preview">
                    <Show when={g().coverImage} fallback={<ConsoleHardwareVisual platformId={g().platform} size="lg" />}>
                      <img src={g().coverImage} alt={g().title} class="preview-cover-art" />
                    </Show>
                    <div class="preview-info-dock">
                      <div class="preview-game-title">{g().title}</div>
                      <div class="preview-platform-tag">{g().platformName.toUpperCase()}</div>
                    </div>
                  </div>

                  {/* Right Emulator Core Selection */}
                  <div class="selector-cores-panel">
                    <div>
                      <div class="selector-header-badge">{g().platformName}</div>
                      <Dialog.Title class="blade-game-title" style={{ "font-size": '1.5rem', "margin-bottom": '0.5rem' }}>
                        Emulador
                      </Dialog.Title>
                      <Dialog.Description class="selector-sub-note">
                        {compatibleEmulators().length} disponibles para esta plataforma
                      </Dialog.Description>
                    </div>

                    <div class="cores-list-scroll">
                      <Show when={compatibleEmulators().length === 0}>
                        <p>No hay emuladores para esta plataforma.</p>
                      </Show>
                      <For each={compatibleEmulators()}>
                        {(emu) => {
                          const isSelected = () => (selectedEmulator()?.id === emu.id);

                          return (
                            <button type="button" aria-pressed={isSelected()}
                              class={`emulator-core-row ${isSelected() ? 'active-core' : ''}`}
                              onClick={() => setSelectedEmulatorId(emu.id)}
                            >
                              <div class="core-radio-indicator">
                                {isSelected() ? '●' : '○'}
                              </div>

                              <div class="core-text-group">
                                <div class="core-title-line">
                                  <span class="core-name-text">{emu.name}</span>
                                  <span class="core-type-chip">{emu.coreType.toUpperCase()}</span>
                                  <span class="core-version-chip">v{emu.version}</span>
                                </div>
                                <div class="core-command-line">
                                  <code>{emu.executable} {emu.arguments.join(' ')}</code>
                                </div>
                              </div>

                              <div class="core-status-pill" title={emulatorBlockReason(emu) ?? 'Listo'}>
                                {emulatorBlockReason(emu) ?? 'LISTO'}
                              </div>
                            </button>
                          );
                        }}
                      </For>
                    </div>

                    {/* Launch Actions */}
                    <div class="blade-action-buttons" style={{ "margin-top": '1rem' }}>
                      <button
                        class="console-btn primary-glow-btn"
                        id="btn-launch-with-core"
                        disabled={Boolean(emulatorBlockReason(selectedEmulator()))}
                        onClick={() => {
                          const emu = selectedEmulator();
                          if (emu && !emulatorBlockReason(emu)) {
                            props.onConfirmLaunch(g(), emu);
                          }
                        }}
                      >
                        <Play size={18} /><span>Jugar</span>
                      </button>

                      <button
                        class="console-btn ghost-btn"
                        id="btn-cancel-core-select"
                        onClick={props.onClose}
                      >
                        <ArrowLeft size={18} /><span>Volver</span>
                      </button>
                    </div>
                  </div>
                </div>
              )}
            </Show>
          </Dialog.Content>
        </div>
      </Dialog.Portal>
    </Dialog>
  );
};
