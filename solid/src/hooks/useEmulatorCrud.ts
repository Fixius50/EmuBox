import { createSignal, createEffect } from 'solid-js';
import type { Emulator } from '@contracts/game.types';
import type { UseEmulatorCrudOptions, UseEmulatorCrudReturn } from '@contracts/modal.types';
import { animateEmulatorModalEntrance } from '@animations/settings-animations';

export function useEmulatorCrud(options: UseEmulatorCrudOptions): UseEmulatorCrudReturn {
  const [formData, setFormData] = createSignal<Partial<Emulator>>({
    name: 'Nuevo Motor de Emulacion',
    supportedPlatforms: ['snes'],
    coreType: 'libretro',
    executable: '/usr/bin/retroarch',
    arguments: ['-L', '/usr/lib/libretro/custom_core_libretro.so'],
    version: '1.0.0',
    status: 'active'
  });

  const [modalFocusIdx, setModalFocusIdx] = createSignal<number>(0);
  const [isTyping, setIsTyping] = createSignal<boolean>(false);

  let modalBoxRef: HTMLDivElement | undefined;
  let nameInputRef: HTMLInputElement | undefined;
  let execInputRef: HTMLInputElement | undefined;
  let typeInputRef: HTMLInputElement | undefined;
  let deleteBtnRef: HTMLButtonElement | undefined;
  let cancelBtnRef: HTMLButtonElement | undefined;
  let saveBtnRef: HTMLButtonElement | undefined;

  const setElementRefs = (refs: {
    modalBoxRef?: HTMLDivElement;
    nameInputRef?: HTMLInputElement;
    execInputRef?: HTMLInputElement;
    typeInputRef?: HTMLInputElement;
    deleteBtnRef?: HTMLButtonElement;
    cancelBtnRef?: HTMLButtonElement;
    saveBtnRef?: HTMLButtonElement;
  }) => {
    if (refs.modalBoxRef) modalBoxRef = refs.modalBoxRef;
    if (refs.nameInputRef) nameInputRef = refs.nameInputRef;
    if (refs.execInputRef) execInputRef = refs.execInputRef;
    if (refs.typeInputRef) typeInputRef = refs.typeInputRef;
    if (refs.deleteBtnRef) deleteBtnRef = refs.deleteBtnRef;
    if (refs.cancelBtnRef) cancelBtnRef = refs.cancelBtnRef;
    if (refs.saveBtnRef) saveBtnRef = refs.saveBtnRef;
  };

  createEffect(() => {
    if (options.isOpen()) {
      const initial = options.initialData();
      if (initial) {
        setFormData({ ...initial });
      } else {
        setFormData({
          id: `emu-${Date.now()}`,
          name: 'Nuevo Motor de Emulacion',
          supportedPlatforms: ['snes'],
          coreType: 'libretro',
          executable: '/usr/bin/retroarch',
          arguments: ['-L', '/usr/lib/libretro/custom_core_libretro.so'],
          version: '1.0.0',
          status: 'active'
        });
      }
      setModalFocusIdx(0);
      setIsTyping(false);
      setTimeout(() => {
        if (modalBoxRef) animateEmulatorModalEntrance(modalBoxRef);
      }, 50);
    }
  });

  const focusModalElement = (idx: number) => {
    setModalFocusIdx(idx);
    setIsTyping(false);
    if (typeof document !== 'undefined' && document.activeElement instanceof HTMLElement && document.activeElement.tagName === 'INPUT') {
      document.activeElement.blur();
    }
    const hasDelete = !!formData().id;
    switch (idx) {
      case 3:
        if (hasDelete) deleteBtnRef?.focus();
        else cancelBtnRef?.focus();
        break;
      case 4:
        if (hasDelete) cancelBtnRef?.focus();
        else saveBtnRef?.focus();
        break;
      case 5:
        if (hasDelete) saveBtnRef?.focus();
        break;
      default:
        break;
    }
  };

  createEffect(() => {
    (window as any).__EMUBOX_IS_EMULATOR_MODAL_OPEN__ = () => options.isOpen();
    (window as any).__EMUBOX_CLOSE_EMULATOR_MODAL__ = () => options.onClose();
    (window as any).__EMUBOX_EMU_MODAL_NAV__ = (dir: 'UP' | 'DOWN' | 'LEFT' | 'RIGHT' | 'SELECT') => {
      if (!options.isOpen()) return;
      const cur = modalFocusIdx();
      const hasDelete = !!formData().id;
      const maxIdx = hasDelete ? 5 : 4;

      switch (dir) {
        case 'DOWN':
          if (cur < 2) {
            focusModalElement(cur + 1);
          } else if (cur === 2) {
            focusModalElement(hasDelete ? 5 : 4);
          }
          break;
        case 'UP':
          if (cur >= 3) {
            focusModalElement(2);
          } else if (cur > 0) {
            focusModalElement(cur - 1);
          }
          break;
        case 'LEFT':
          if (cur > 3) {
            focusModalElement(cur - 1);
          }
          break;
        case 'RIGHT':
          if (cur >= 3 && cur < maxIdx) {
            focusModalElement(cur + 1);
          }
          break;
        case 'SELECT':
          if (hasDelete && cur === 3) {
            handleDelete();
          } else if ((hasDelete && cur === 4) || (!hasDelete && cur === 3)) {
            options.onClose();
          } else if ((hasDelete && cur === 5) || (!hasDelete && cur === 4)) {
            handleSave();
          } else {
            setIsTyping(true);
            switch (cur) {
              case 0:
                nameInputRef?.focus();
                nameInputRef?.select();
                break;
              case 1:
                execInputRef?.focus();
                execInputRef?.select();
                break;
              case 2:
                typeInputRef?.focus();
                typeInputRef?.select();
                break;
              default:
                break;
            }
          }
          break;
        default:
          break;
      }
    };
  });

  const handleSave = () => {
    const data = formData();
    if (data.id && data.name) {
      options.onSave(data as Emulator);
      options.onClose();
    }
  };

  const handleDelete = () => {
    const data = formData();
    if (data.id) {
      options.onDelete(data.id);
      options.onClose();
    }
  };

  return {
    formData,
    setFormData,
    modalFocusIdx,
    isTyping,
    setIsTyping,
    focusModalElement,
    handleSave,
    handleDelete,
    setElementRefs
  };
}
