import { Component, For, Show } from "solid-js";
import { Dialog } from "@kobalte/core/dialog";
import type { MaintenanceModalProps } from "@contracts/modal.types";
import { useMaintenanceController } from "@hooks/useMaintenanceController";
import { X } from "lucide-solid";
import { SettingCardRow } from "@components/common/SettingCardRow";

export const MaintenanceModal: Component<MaintenanceModalProps> = (props) => {
  const { actions, feedbackMsg, isLoading, handleExecute, activeIndex } =
    useMaintenanceController({
      backend: props.backend,
      isOpen: () => props.isOpen,
      onControllerReady: props.onControllerReady,
      onClose: props.onClose,
      focusedIndex: () => props.focusedIndex ?? 0,
      onSelectIndex: props.onSelectIndex,
    });

  return (
    <Dialog
      open={props.isOpen}
      onOpenChange={(open) => {
        if (!open) props.onClose();
      }}
    >
      <Dialog.Portal>
        <Dialog.Overlay class="console-modal-backdrop" />
        <div class="console-modal-center-container">
          <Dialog.Content class="crud-modal-box xmb-maintenance">
            <div
              style={{
                display: "flex",
                "align-items": "center",
                "justify-content": "space-between",
                "border-bottom": "0.0625rem solid rgba(255, 255, 255, 0.1)",
                "padding-bottom": "0.75rem",
              }}
            >
              <div>
                <Dialog.Title class="crud-modal-title">
                  Mantenimiento
                </Dialog.Title>
                <Dialog.Description
                  style={{
                    "font-size": "0.6875rem",
                    color: "var(--text-secondary)",
                    "margin-top": "0.125rem",
                  }}
                >
                  Sistema y recuperacion
                </Dialog.Description>
              </div>
              <Dialog.CloseButton
                class="xmb-icon-button"
                title="Cerrar mantenimiento"
                aria-label="Cerrar mantenimiento"
              >
                <X size={20} />
              </Dialog.CloseButton>
            </div>

            <Show when={feedbackMsg()}>
              <div
                style={{
                  background: "rgba(0, 240, 255, 0.1)",
                  border: "0.0625rem solid rgba(0, 240, 255, 0.4)",
                  padding: "0.625rem 1rem",
                  "border-radius": "var(--border-radius-sm)",
                  "font-size": "0.75rem",
                  color: "#00f0ff",
                  "font-weight": "800",
                }}
              >
                {feedbackMsg()}
              </div>
            </Show>

            <div
              style={{
                display: "flex",
                "flex-direction": "column",
                gap: "0.625rem",
                "margin-top": "0.5rem",
              }}
            >
              <For each={actions}>
                {(item, idx) => {
                  const isFocused = () => activeIndex() === idx();

                  return (
                    <SettingCardRow
                      title={item.title}
                      description={item.description}
                      tag={item.tag}
                      isFocused={isFocused()}
                      actionBadge="Ejecutar"
                      onClick={() => handleExecute(idx())}
                      style={{
                        padding: "0.875rem 1.25rem",
                        "border-color": isFocused()
                          ? "var(--xmb-accent)"
                          : "var(--xmb-border)",
                        background: isFocused() ? "#28423f" : "#16262b",
                        "box-shadow": "none",
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
                Cerrar
              </button>
            </div>
          </Dialog.Content>
        </div>
      </Dialog.Portal>
    </Dialog>
  );
};
