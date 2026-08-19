import { Component, Show } from 'solid-js';
import { Dialog } from '@kobalte/core/dialog';
import type { ConfirmLaunchModalProps } from '@contracts/modal.types';

export const ConfirmLaunchModal: Component<ConfirmLaunchModalProps> = (props) => {
  return (
    <Dialog open={props.isOpen} onOpenChange={(open) => { if (!open) props.onClose(); }}>
      <Dialog.Portal>
        <Dialog.Overlay class="console-modal-backdrop" />
        <div class="console-modal-center-container">
          <Dialog.Content class="console-game-blade" style={{ width: '45rem', height: 'auto' }}>
            <Show when={props.game}>
              {(g) => (
                <div class="confirm-launch-box">
                  <div class="launch-header-row">
                    <div class="launch-pulse-icon">RUN</div>
                    <div>
                      <Dialog.Title class="blade-game-title" style={{ "font-size": '1.5rem', "margin-bottom": '0.25rem' }}>
                        Iniciar Sesión de Emulación
                      </Dialog.Title>
                      <Dialog.Description class="launch-sub-note">
                        Se suspenderá la UI de EmuBox para transferir el control DRM/KMS al emulador.
                      </Dialog.Description>
                    </div>
                  </div>

                  <div class="launch-target-card">
                    <img src={g().coverImage} alt={g().title} class="launch-thumb" />
                    <div>
                      <div class="launch-game-name">{g().title}</div>
                      <div class="launch-console-tag">{g().platformName.toUpperCase()}</div>
                      <div class="launch-pipeline-info">Direct KMS/DRM • Gamescope Compositor</div>
                    </div>
                  </div>

                  <div class="terminal-command-dock">
                    <span class="dock-cmd-label">COMANDO DE ARRANQUE GENERADO:</span>
                    <span class="dock-cmd-code">gamescope -W 1920 -H 1080 -f -r 60 -- retroarch -L {g().platform} "{g().romPath || `${g().title}.rom`}"</span>
                  </div>

                  <div class="blade-action-buttons">
                    <button
                      class="console-btn primary-glow-btn"
                      id="btn-confirm-exec"
                      onClick={() => props.onConfirmLaunch(g())}
                    >
                      <span>[A] LANZAR JUEGO AHORA</span>
                    </button>

                    <button
                      class="console-btn ghost-btn"
                      id="btn-cancel-exec"
                      onClick={props.onClose}
                    >
                      <span>[B] CANCELAR</span>
                    </button>
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
