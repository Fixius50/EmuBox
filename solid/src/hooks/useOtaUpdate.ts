import { createSignal, Accessor } from 'solid-js';
import type { UpdateInfo } from '@contracts/update.types';
import { animateUpdateProgressBar } from '@animations/update-animations';

export interface UseOtaUpdateOptions {
  updateInfo?: () => UpdateInfo | undefined;
  onCheckUpdates?: () => Promise<any>;
  onApplyUpdate?: (ver: string) => Promise<any>;
  onSelectContentArea?: () => void;
}

export interface UseOtaUpdateReturn {
  isCheckingUpdate: Accessor<boolean>;
  isUpdating: Accessor<boolean>;
  updateProgressVal: Accessor<number>;
  updateStatusMsg: Accessor<string>;
  handleCheckForUpdates: () => Promise<void>;
  handleApplyUpdate: () => Promise<void>;
  handleTriggerAction: () => void;
  setProgressBarRef: (el: HTMLDivElement) => void;
}

export function useOtaUpdate(options: UseOtaUpdateOptions): UseOtaUpdateReturn {
  const [isCheckingUpdate, setIsCheckingUpdate] = createSignal<boolean>(false);
  const [isUpdating, setIsUpdating] = createSignal<boolean>(false);
  const [updateProgressVal, setUpdateProgressVal] = createSignal<number>(0);
  const [updateStatusMsg, setUpdateStatusMsg] = createSignal<string>('');
  let progressBarRef: HTMLDivElement | undefined;

  const setProgressBarRef = (el: HTMLDivElement) => {
    progressBarRef = el;
  };

  const handleCheckForUpdates = async () => {
    setIsCheckingUpdate(true);
    setUpdateStatusMsg('Consultando releases en GitHub API...');
    try {
      if (options.onCheckUpdates) {
        await options.onCheckUpdates();
      }
      setUpdateStatusMsg('Comprobacion completada.');
    } finally {
      setTimeout(() => setIsCheckingUpdate(false), 800);
    }
  };

  const handleApplyUpdate = async () => {
    setIsUpdating(true);
    setUpdateProgressVal(15);
    setUpdateStatusMsg('Descargando paquete de actualizacion en /opt/emubox/releases...');
    if (progressBarRef) animateUpdateProgressBar(progressBarRef, 15);

    setTimeout(async () => {
      setUpdateProgressVal(60);
      setUpdateStatusMsg('Verificando checksum SHA256 y desempaquetando...');
      if (progressBarRef) animateUpdateProgressBar(progressBarRef, 60);

      setTimeout(async () => {
        setUpdateProgressVal(90);
        setUpdateStatusMsg('Reasignando enlace atomico /opt/emubox/current...');
        if (progressBarRef) animateUpdateProgressBar(progressBarRef, 90);

        setTimeout(async () => {
          if (options.onApplyUpdate) {
            await options.onApplyUpdate('v1.0.1');
          }
          setUpdateProgressVal(100);
          setUpdateStatusMsg('Actualizacion aplicada con exito. Reiniciando sesion de EmuBox...');
          if (progressBarRef) animateUpdateProgressBar(progressBarRef, 100);
          setTimeout(() => setIsUpdating(false), 1200);
        }, 600);
      }, 600);
    }, 600);
  };

  const handleTriggerAction = () => {
    options.onSelectContentArea?.();
    const hasUpdate = options.updateInfo?.()?.hasUpdate;
    if (hasUpdate) {
      handleApplyUpdate();
    } else {
      handleCheckForUpdates();
    }
  };

  return {
    isCheckingUpdate,
    isUpdating,
    updateProgressVal,
    updateStatusMsg,
    handleCheckForUpdates,
    handleApplyUpdate,
    handleTriggerAction,
    setProgressBarRef
  };
}
