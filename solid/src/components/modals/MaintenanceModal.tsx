import { Component, For, Show } from 'solid-js';
import { Dialog } from '@kobalte/core/dialog';
import type { MaintenanceModalProps } from '@contracts/modal.types';
import { useMaintenanceController } from '@hooks/useMaintenanceController';
import { Badge } from '@components/common/Badge';
import { SettingCardRow } from '@components/common/SettingCardRow';

export const MaintenanceModal: Component<MaintenanceModalProps> = (props) => {
  const {
    actions,
    feedbackMsg,
    isLoading,
    handleExecute,
    activeIndex
  } = useMaintenanceController({
    backend: props.backend,
    onClose: props.onClose,
    focusedIndex: () => props.focusedIndex ?? 0,
    onSelectIndex: props.onSelectIndex
  });

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
              <Badge variant="boost" style={{ "font-size": "0.625rem" }}>RESCUE MODE</Badge>
            </div>

            <Show when={feedbackMsg()}>
              <div style={{ background: "rgba(0, 240, 255, 0.1)", border: "0.0625rem solid rgba(0, 240, 255, 0.4)", padding: "0.625rem 1rem", "border-radius": "var(--border-radius-sm)", "font-size": "0.75rem", color: "#00f0ff", "font-weight": "800" }}>
                {feedbackMsg()}
              </div>
            </Show>

            <div style={{ display: "flex", "flex-direction": "column", gap: "0.625rem", "margin-top": "0.5rem" }}>
              <For each={actions}>
                {(item, idx) => {
                  const isFocused = () => activeIndex() === idx();

                  return (
                    <SettingCardRow
                      title={item.title}
                      description={item.description}
                      tag={item.tag}
                      isFocused={isFocused()}
                      actionBadge="[A] EJECUTAR"
                      onClick={() => handleExecute(idx())}
                      style={{
                        padding: "0.875rem 1.25rem",
                        "border-color": isFocused() ? "#00f0ff" : "rgba(255, 255, 255, 0.1)",
                        background: isFocused() ? "rgba(20, 35, 65, 0.95)" : "rgba(15, 23, 42, 0.65)",
                        "box-shadow": isFocused() ? "0 0 1.25rem rgba(0, 240, 255, 0.5)" : "none"
                      }}
                    />
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
                [B] CERRAR MENU
              </button>
            </div>
          </Dialog.Content>
        </div>
      </Dialog.Portal>
    </Dialog>
  );
};
