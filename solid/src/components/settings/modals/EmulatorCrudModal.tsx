import { Component, Show } from "solid-js";
import type { EmulatorCrudModalProps } from "@contracts/modal.types";
import { useEmulatorCrud } from "@hooks/useEmulatorCrud";

export const EmulatorCrudModal: Component<EmulatorCrudModalProps> = (props) => {
  const {
    formData,
    setFormData,
    modalFocusIdx,
    isTyping,
    setIsTyping,
    isPending,
    error,
    handleClose,
    handleSave,
    handleDelete,
    setElementRefs,
  } = useEmulatorCrud({
    isOpen: () => props.isOpen,
    onControllerReady: props.onControllerReady,
    initialData: () => props.initialData,
    onClose: props.onClose,
    onSave: props.onSave,
    onDelete: props.onDelete,
  });

  return (
    <Show when={props.isOpen}>
      <div class="crud-modal-backdrop" onClick={handleClose}>
        <div
          class="crud-modal-box"
          aria-busy={isPending()}
          ref={(element) => setElementRefs({ modalBoxRef: element })}
          onClick={(event) => event.stopPropagation()}
          onKeyDown={(event) => {
            if (
              event.target instanceof HTMLInputElement &&
              !["Enter", "Escape"].includes(event.key)
            )
              event.stopPropagation();
          }}
        >
          <h3 class="crud-modal-title">
            {formData().id
              ? "Gestionar Motor / Núcleo de Emulación"
              : "Añadir Nuevo Motor de Emulación"}
          </h3>

          <div class="crud-form-group">
            <label class="crud-label">Nombre del Motor</label>
            <input
              ref={(element) => setElementRefs({ nameInputRef: element })}
              type="text"
              disabled={isPending()}
              class={`crud-input ${modalFocusIdx() === 0 ? "focused" : ""} ${isTyping() && modalFocusIdx() === 0 ? "typing" : ""}`}
              value={formData().name || ""}
              onFocus={() => setIsTyping(true)}
              onBlur={() => setIsTyping(false)}
              onInput={(e) =>
                setFormData({ ...formData(), name: e.currentTarget.value })
              }
            />
          </div>

          <div class="crud-form-group">
            <label class="crud-label">Binario / Ejecutable</label>
            <input
              ref={(element) => setElementRefs({ execInputRef: element })}
              type="text"
              disabled={isPending()}
              class={`crud-input ${modalFocusIdx() === 1 ? "focused" : ""} ${isTyping() && modalFocusIdx() === 1 ? "typing" : ""}`}
              value={formData().executable || ""}
              onFocus={() => setIsTyping(true)}
              onBlur={() => setIsTyping(false)}
              onInput={(e) =>
                setFormData({
                  ...formData(),
                  executable: e.currentTarget.value,
                })
              }
            />
          </div>

          <div class="crud-form-group">
            <label class="crud-label">Tipo de Motor</label>
            <input
              ref={(element) => setElementRefs({ typeInputRef: element })}
              type="text"
              disabled={isPending()}
              class={`crud-input ${modalFocusIdx() === 2 ? "focused" : ""} ${isTyping() && modalFocusIdx() === 2 ? "typing" : ""}`}
              value={formData().coreType || "libretro"}
              onFocus={() => setIsTyping(true)}
              onBlur={() => setIsTyping(false)}
              onInput={(e) =>
                setFormData({
                  ...formData(),
                  coreType: e.currentTarget.value as any,
                })
              }
            />
          </div>

          <Show when={error()}>
            <p role="alert">{error()}</p>
          </Show>

          <div class="crud-modal-actions">
            <Show when={formData().id}>
              <button
                ref={(element) => setElementRefs({ deleteBtnRef: element })}
                class={`crud-btn-delete ${modalFocusIdx() === 3 ? "focused" : ""}`}
                disabled={isPending()}
                onClick={handleDelete}
              >
                ELIMINAR NÚCLEO
              </button>
            </Show>
            <button
              ref={(element) => setElementRefs({ cancelBtnRef: element })}
              class={`crud-btn-cancel ${(formData().id ? modalFocusIdx() === 4 : modalFocusIdx() === 3) ? "focused" : ""}`}
              disabled={isPending()}
              onClick={handleClose}
            >
              CANCELAR [B]
            </button>
            <button
              ref={(element) => setElementRefs({ saveBtnRef: element })}
              class={`crud-btn-save ${(formData().id ? modalFocusIdx() === 5 : modalFocusIdx() === 4) ? "focused" : ""}`}
              disabled={isPending()}
              onClick={handleSave}
            >
              GUARDAR CAMBIOS [A]
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};
