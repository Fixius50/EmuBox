import { createSignal, createMemo } from 'solid-js';
import type { MaintenanceAction, UseMaintenanceOptions, UseMaintenanceReturn } from '@contracts/modal.types';

export function useMaintenanceController(options: UseMaintenanceOptions): UseMaintenanceReturn {
  const [feedbackMsg, setFeedbackMsg] = createSignal<string>('');
  const [isLoading, setIsLoading] = createSignal<boolean>(false);

  const activeIndex = createMemo(() => (options.focusedIndex ? options.focusedIndex() : 0));

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
        await options.backend.restartAppSession();
        setTimeout(() => {
          setIsLoading(false);
          options.onClose();
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
        await options.backend.getStorageLocations();
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
        const res = await options.backend.checkForUpdates('stable');
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
        await options.backend.restart();
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
        await options.backend.shutdown();
      }
    }
  ];

  const handleExecute = async (idx: number) => {
    options.onSelectIndex?.(idx);
    const item = actions[idx];
    if (item && !isLoading()) {
      await item.action();
    }
  };

  (window as any).__EMUBOX_TRIGGER_MAINTENANCE__ = () => {
    handleExecute(activeIndex());
  };

  return {
    actions,
    feedbackMsg,
    isLoading,
    handleExecute,
    activeIndex
  };
}
