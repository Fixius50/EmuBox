import { Component, createSignal, For, createEffect } from 'solid-js';
import { Dialog } from '@kobalte/core/dialog';
import type { IEmuBoxBackend } from '@contracts/backend.types';

export interface MaintenanceAction {
  id: string;
  tag: string;
  title: string;
  description: string;
  variant?: 'primary' | 'danger' | 'warning' | 'default';
  action: () => Promise<void> | void;
}

interface MaintenanceModalProps {
  isOpen: boolean;
  onClose: () => void;
  backend: IEmuBoxBackend;
  focusedIndex?: number;
  onSelectIndex?: (idx: number) => void;
}

export const MAINTENANCE_ACTIONS_COUNT = 5;

export const MaintenanceModal: Component<MaintenanceModalProps> = (props) => {
  const [feedbackMsg, setFeedbackMsg] = createSignal<string>('');
  const [isLoading, setIsLoading] = createSignal<boolean>(false);

  const activeIndex = () => (props.focusedIndex !== undefined ? props.focusedIndex : 0);

  const actions: MaintenanceAction[] = [
    {
      id: 'restart-app',
      tag: 'SESION',
      title: 'Reiniciar Sesion de EmuBox',
      description: 'Recarga la interfaz grafica de usuario sin reiniciar el sistema operativo Arch Linux',
      variant: 'primary',
      action: async () => {
        setIsLoading(true);
        setFeedbackMsg('Reiniciando interfaz de EmuBox...');
        await props.backend.restartAppSession();
        setTimeout(() => {
          setIsLoading(false);
          props.onClose();
        }, 1000);
      }
    },
    {
      id: 'repair-dirs',
      tag: 'STORAGE',
      title: 'Reparar Permisos y Directorios XDG',
      description: 'Restaura y valida la estructura de carpetas de ROMs, partidas, BIOS y logs',
      variant: 'default',
      action: async () => {
        setIsLoading(true);
        setFeedbackMsg('Comprobando y asegurando directorios en ~/.local/share/emubox...');
        await props.backend.getStorageLocations();
        setTimeout(() => {
          setIsLoading(false);
          setFeedbackMsg('Directorios y permisos verificados correctamente.');
        }, 800);
      }
    },
    {
      id: 'check-updates',
      tag: 'OTA',
      title: 'Forzar Comprobacion de Actualizacion OTA',
      description: 'Consulta los servidores de releases en GitHub para verificar nuevas versiones',
      variant: 'default',
      action: async () => {
        setIsLoading(true);
        setFeedbackMsg('Consultando releases en GitHub...');
        const res = await props.backend.checkForUpdates('stable');
        setTimeout(() => {
          setIsLoading(false);
          setFeedbackMsg(res.updateAvailable ? `Actualizacion disponible: ${res.targetVersion}` : 'El sistema ya se encuentra en la version mas reciente.');
        }, 800);
      }
    },
    {
      id: 'reboot',
      tag: 'ENERGIA',
      title: 'Reiniciar Consola',
      description: 'Reinicia completamente el hardware y el sistema operativo Arch Linux',
      variant: 'warning',
      action: async () => {
        setIsLoading(true);
        setFeedbackMsg('Reiniciando sistema...');
        await props.backend.restart();
      }
    },
    {
      id: 'poweroff',
      tag: 'ENERGIA',
      title: 'Apagar Consola',
      description: 'Cierra los procesos de emulacion de forma segura y apaga la maquina',
      variant: 'danger',
      action: async () => {
        setIsLoading(true);
        setFeedbackMsg('Apagando sistema...');
        await props.backend.shutdown();
      }
    }
  ];

  const handleExecute = async (idx: number) => {
    props.onSelectIndex?.(idx);
    const item = actions[idx];
    if (item && !isLoading()) {
      await item.action();
    }
  };

  // Expose execution on current item
  (window as any).__EMUBOX_TRIGGER_MAINTENANCE__ = () => {
    handleExecute(activeIndex());
  };

  return (
    <Dialog open={props.isOpen} onOpenChange={(open) => { if (!open) props.onClose(); }}>
      <Dialog.Portal>
        <Dialog.Overlay class="console-modal-backdrop" style={{ "background": "rgba(0, 0, 0, 0.92)", "backdrop-filter": "blur(2rem)" }} />
        <div class="console-modal-center-container">
          <Dialog.Content class="crud-modal-box" style={{ width: "42rem", "border-color": "#f59e0b", "box-shadow": "0 1rem 5rem rgba(0, 0, 0, 0.98), 0 0 2rem rgba(245, 158, 11, 0.3)" }}>
            <div style={{ display: "flex", "align-items": "center", "justify-content": "space-between", "border-bottom": "0.0625rem solid rgba(255, 255, 255, 0.1)", "padding-bottom": "0.75rem" }}>
              <div>
                <Dialog.Title class="crud-modal-title" style={{ color: "#f59e0b", "font-size": "1.125rem", "letter-spacing": "0.0625rem" }}>
                  MODO DE MANTENIMIENTO Y RECUPERACION
                </Dialog.Title>
                <Dialog.Description style={{ "font-size": "0.6875rem", color: "var(--text-secondary)", "margin-top": "0.125rem" }}>
                  Usa el D-Pad o Flechas para navegar y [A] / Enter para ejecutar
                </Dialog.Description>
              </div>
              <span class="readonly-badge boost" style={{ "font-size": "0.625rem" }}>RESCUE MODE</span>
            </div>

            {feedbackMsg() && (
              <div style={{ background: "rgba(0, 240, 255, 0.1)", border: "0.0625rem solid rgba(0, 240, 255, 0.4)", padding: "0.625rem 1rem", "border-radius": "var(--border-radius-sm)", "font-size": "0.75rem", color: "#00f0ff", "font-weight": "800" }}>
                {feedbackMsg()}
              </div>
            )}

            <div style={{ display: "flex", "flex-direction": "column", gap: "0.625rem", "margin-top": "0.5rem" }}>
              <For each={actions}>
                {(item, idx) => {
                  const isFocused = () => activeIndex() === idx();

                  return (
                    <div
                      class={`setting-card-row ${isFocused() ? 'focused' : ''}`}
                      style={{
                        padding: "0.875rem 1.25rem",
                        "border-color": isFocused() ? "#00f0ff" : "rgba(255, 255, 255, 0.1)",
                        background: isFocused() ? "rgba(20, 35, 65, 0.95)" : "rgba(15, 23, 42, 0.65)",
                        "box-shadow": isFocused() ? "0 0 1.25rem rgba(0, 240, 255, 0.5)" : "none"
                      }}
                      onClick={() => handleExecute(idx())}
                      onMouseEnter={() => props.onSelectIndex?.(idx())}
                    >
                      <div class="setting-info">
                        <div style={{ display: "flex", "align-items": "center", gap: "0.5rem" }}>
                          <span class="readonly-badge" style={{ "font-size": "0.5625rem", padding: "0.125rem 0.375rem" }}>{item.tag}</span>
                          <span class="setting-title" style={{ "font-size": "0.875rem", color: isFocused() ? "#ffffff" : "#cbd5e1" }}>{item.title}</span>
                        </div>
                        <span class="setting-desc" style={{ "font-size": "0.6875rem", "margin-top": "0.125rem" }}>{item.description}</span>
                      </div>

                      <span class={`readonly-badge interactive-badge ${isFocused() ? 'highlight' : ''}`} style={{ "font-size": "0.625rem" }}>
                        [A] EJECUTAR
                      </span>
                    </div>
                  );
                }}
              </For>
            </div>

            <div class="crud-modal-actions" style={{ "margin-top": "0.75rem" }}>
              <button
                class="crud-btn-cancel"
                onClick={props.onClose}
                disabled={isLoading()}
              >
                CERRAR MENU (B)
              </button>
            </div>
          </Dialog.Content>
        </div>
      </Dialog.Portal>
    </Dialog>
  );
};
