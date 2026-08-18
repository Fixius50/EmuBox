import { Component, For, Show, createSignal, createEffect } from 'solid-js';
import { Dialog } from '@kobalte/core/dialog';
import { animateModalOpen } from '@animations/modal-animations';
import type { Game, Emulator } from '@contracts/game.types';

interface EmulatorSelectorModalProps {
  game: Game | null;
  emulators: Emulator[];
  isOpen: boolean;
  onClose: () => void;
  onConfirmLaunch: (game: Game, emulator: Emulator) => void;
}

export const EmulatorSelectorModal: Component<EmulatorSelectorModalProps> = (props) => {
  const [selectedEmulatorId, setSelectedEmulatorId] = createSignal<string>('');
  let modalContentRef!: HTMLDivElement;

  const compatibleEmulators = () => {
    if (!props.game) return [];
    const list = props.emulators.filter(e => e.supportedPlatforms.includes(props.game!.platform));
    if (list.length === 0) return props.emulators.slice(0, 2);
    return list;
  };

  const selectedEmulator = () => {
    const list = compatibleEmulators();
    const id = selectedEmulatorId();
    if (id) {
      const found = list.find(e => e.id === id);
      if (found) return found;
    }
    return list[0] || null;
  };

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
          <Dialog.Content class="console-game-blade emulator-selector-blade" ref={modalContentRef}>
            <Show when={props.game}>
              {(g) => (
                <div class="emulator-selector-layout">
                  {/* Left Game Preview */}
                  <div class="selector-game-preview">
                    <img src={g().coverImage} alt={g().title} class="preview-cover-art" />
                    <div class="preview-info-dock">
                      <div class="preview-game-title">{g().title}</div>
                      <div class="preview-platform-tag">{g().platformName.toUpperCase()}</div>
                    </div>
                  </div>

                  {/* Right Emulator Core Selection */}
                  <div class="selector-cores-panel">
                    <div>
                      <div class="selector-header-badge">SELECCIÓN DE MOTOR DE EJECUCIÓN</div>
                      <Dialog.Title class="blade-game-title" style={{ "font-size": '1.5rem', "margin-bottom": '0.5rem' }}>
                        ¿Con qué emulador deseas ejecutar el título?
                      </Dialog.Title>
                      <Dialog.Description class="selector-sub-note">
                        Elige el binario o núcleo libretro optimizado para esta sesión DRM/KMS
                      </Dialog.Description>
                    </div>

                    <div class="cores-list-scroll">
                      <For each={compatibleEmulators()}>
                        {(emu) => {
                          const isSelected = () => (selectedEmulator()?.id === emu.id);

                          return (
                            <div
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

                              <div class="core-status-pill ready">
                                LISTO
                              </div>
                            </div>
                          );
                        }}
                      </For>
                    </div>

                    {/* Launch Actions */}
                    <div class="blade-action-buttons" style={{ "margin-top": '1rem' }}>
                      <button
                        class="console-btn primary-glow-btn"
                        id="btn-launch-with-core"
                        onClick={() => {
                          const emu = selectedEmulator();
                          if (emu) {
                            props.onConfirmLaunch(g(), emu);
                          }
                        }}
                      >
                        <span class="btn-icon">▶</span>
                        <span>EJECUTAR CON {selectedEmulator()?.name.toUpperCase() || 'EMULADOR'} (A)</span>
                      </button>

                      <button
                        class="console-btn ghost-btn"
                        id="btn-cancel-core-select"
                        onClick={props.onClose}
                      >
                        <span>VOLVER (B)</span>
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
