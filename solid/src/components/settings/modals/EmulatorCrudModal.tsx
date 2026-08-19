import { Component, onMount, Show } from 'solid-js';
import type { EmulatorCrudModalProps } from '@contracts/modal.types';
import { useEmulatorCrud } from '@hooks/useEmulatorCrud';

export const EmulatorCrudModal: Component<EmulatorCrudModalProps> = (props) => {
  let modalBoxRef: HTMLDivElement | undefined;
  let nameInputRef: HTMLInputElement | undefined;
  let execInputRef: HTMLInputElement | undefined;
  let typeInputRef: HTMLInputElement | undefined;
  let deleteBtnRef: HTMLButtonElement | undefined;
  let cancelBtnRef: HTMLButtonElement | undefined;
  let saveBtnRef: HTMLButtonElement | undefined;

  const {
    formData,
    setFormData,
    modalFocusIdx,
    isTyping,
    setIsTyping,
    handleSave,
    handleDelete,
    setElementRefs
  } = useEmulatorCrud({
    isOpen: () => props.isOpen,
    initialData: () => props.initialData,
    onClose: props.onClose,
    onSave: props.onSave,
    onDelete: props.onDelete
  });

  onMount(() => {
    setElementRefs({
      modalBoxRef,
      nameInputRef,
      execInputRef,
      typeInputRef,
      deleteBtnRef,
      cancelBtnRef,
      saveBtnRef
    });
  });

  return (
    <Show when={props.isOpen}>
      <div class="crud-modal-backdrop" onClick={props.onClose}>
        <div class="crud-modal-box" ref={modalBoxRef} onClick={(e) => e.stopPropagation()}>
          <h3 class="crud-modal-title">
            {formData().id ? 'Gestionar Motor / Núcleo de Emulación' : 'Añadir Nuevo Motor de Emulación'}
          </h3>

          <div class="crud-form-group">
            <label class="crud-label">Nombre del Motor</label>
            <input
              ref={nameInputRef}
              type="text"
              class={`crud-input ${modalFocusIdx() === 0 ? 'focused' : ''} ${isTyping() && modalFocusIdx() === 0 ? 'typing' : ''}`}
              value={formData().name || ''}
              onFocus={() => setIsTyping(true)}
              onBlur={() => setIsTyping(false)}
              onInput={(e) => setFormData({ ...formData(), name: e.currentTarget.value })}
            />
          </div>

          <div class="crud-form-group">
            <label class="crud-label">Binario / Ejecutable</label>
            <input
              ref={execInputRef}
              type="text"
              class={`crud-input ${modalFocusIdx() === 1 ? 'focused' : ''} ${isTyping() && modalFocusIdx() === 1 ? 'typing' : ''}`}
              value={formData().executable || ''}
              onFocus={() => setIsTyping(true)}
              onBlur={() => setIsTyping(false)}
              onInput={(e) => setFormData({ ...formData(), executable: e.currentTarget.value })}
            />
          </div>

          <div class="crud-form-group">
            <label class="crud-label">Tipo de Motor</label>
            <input
              ref={typeInputRef}
              type="text"
              class={`crud-input ${modalFocusIdx() === 2 ? 'focused' : ''} ${isTyping() && modalFocusIdx() === 2 ? 'typing' : ''}`}
              value={formData().coreType || 'libretro'}
              onFocus={() => setIsTyping(true)}
              onBlur={() => setIsTyping(false)}
              onInput={(e) => setFormData({ ...formData(), coreType: e.currentTarget.value as any })}
            />
          </div>

          <div class="crud-modal-actions">
            <Show when={formData().id}>
              <button
                ref={deleteBtnRef}
                class={`crud-btn-delete ${modalFocusIdx() === 3 ? 'focused' : ''}`}
                onClick={handleDelete}
              >
                ELIMINAR NÚCLEO
              </button>
            </Show>
            <button
              ref={cancelBtnRef}
              class={`crud-btn-cancel ${(formData().id ? modalFocusIdx() === 4 : modalFocusIdx() === 3) ? 'focused' : ''}`}
              onClick={props.onClose}
            >
              CANCELAR [B]
            </button>
            <button
              ref={saveBtnRef}
              class={`crud-btn-save ${(formData().id ? modalFocusIdx() === 5 : modalFocusIdx() === 4) ? 'focused' : ''}`}
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
