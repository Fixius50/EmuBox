import { createSignal, createMemo, onMount, onCleanup } from "solid-js";
import type { InputAction } from "@contracts/input.types";
import type {
  MaintenanceAction,
  UseMaintenanceOptions,
  UseMaintenanceReturn,
} from "@contracts/modal.types";

export function useMaintenanceController(
  options: UseMaintenanceOptions,
): UseMaintenanceReturn {
  const [feedbackMsg, setFeedbackMsg] = createSignal<string>("");
  const [isLoading, setIsLoading] = createSignal<boolean>(false);

  const activeIndex = createMemo(() =>
    options.focusedIndex ? options.focusedIndex() : 0,
  );

  const actions: MaintenanceAction[] = [
    {
      id: "restart-app",
      tag: "SESION",
      title: "Reiniciar Sesion de EmuBox",
      description:
        "Recarga la interfaz grafica de usuario sin reiniciar el sistema operativo Arch Linux",
      variant: "primary",
      action: async () => {
        setIsLoading(true);
        setFeedbackMsg("Reiniciando interfaz de EmuBox...");
        await options.backend.restartAppSession();
        setTimeout(() => {
          setIsLoading(false);
          options.onClose();
        }, 1000);
      },
    },
    {
      id: "repair-dirs",
      tag: "STORAGE",
      title: "Inspeccionar Almacenamiento",
      description:
        "Consulta directorios de juegos, partidas, BIOS y registros",
      variant: "default",
      action: async () => {
        setIsLoading(true);
        setFeedbackMsg(
          "Inspeccionando directorios del appliance EmuBox...",
        );
        const locations = await options.backend.getStorageLocations();
        const unavailable = Object.values(locations).filter(location => !location.accessible || !location.isWritable);
        setTimeout(() => {
          setIsLoading(false);
          setFeedbackMsg(unavailable.length ? `Directorios ausentes o sin escritura: ${unavailable.map(location => location.label).join(', ')}` : "Directorios accesibles y con permiso de escritura. No se han modificado permisos.");
        }, 800);
      },
    },
    {
      id: "check-updates",
      tag: "OTA",
      title: "Forzar Comprobacion de Actualizacion OTA",
      description:
        "Consulta los servidores de releases en GitHub para verificar nuevas versiones",
      variant: "default",
      action: async () => {
        setIsLoading(true);
        setFeedbackMsg("Consultando releases en GitHub...");
        const res = await options.backend.checkForUpdates("stable");
        setTimeout(() => {
          setIsLoading(false);
          setFeedbackMsg(
            res.updateAvailable
              ? `Actualizacion disponible: ${res.targetVersion}`
              : "El sistema ya se encuentra en la version mas reciente.",
          );
        }, 800);
      },
    },
    {
      id: "reboot",
      tag: "ENERGIA",
      title: "Reiniciar Consola",
      description:
        "Reinicia completamente el hardware y el sistema operativo Arch Linux",
      variant: "warning",
      action: async () => {
        setIsLoading(true);
        setFeedbackMsg("Reiniciando sistema...");
        await options.backend.restart();
      },
    },
    {
      id: "poweroff",
      tag: "ENERGIA",
      title: "Apagar Consola",
      description:
        "Cierra los procesos de emulacion de forma segura y apaga la maquina",
      variant: "danger",
      action: async () => {
        setIsLoading(true);
        setFeedbackMsg("Apagando sistema...");
        await options.backend.shutdown();
      },
    },
  ];

  const handleExecute = async (idx: number) => {
    options.onSelectIndex?.(idx);
    const item = actions[idx];
    if (item && !isLoading()) {
      try {
        await item.action();
      } catch (error) {
        setFeedbackMsg(
          error instanceof Error
            ? error.message
            : "No se pudo completar la accion.",
        );
        setIsLoading(false);
      }
    }
  };

  const controller = (action: InputAction) => {
    if (!options.isOpen() || isLoading()) return;
    switch (action) {
      case "NAV_DOWN":
      case "NAV_UP":
        options.onSelectIndex?.(
          Math.max(
            0,
            Math.min(
              actions.length - 1,
              activeIndex() + (action === "NAV_DOWN" ? 1 : -1),
            ),
          ),
        );
        break;
      case "BUTTON_A":
        void handleExecute(activeIndex());
        break;
      case "BUTTON_B":
        options.onClose();
        break;
      default:
        break;
    }
  };
  onMount(() => options.onControllerReady?.(controller));
  onCleanup(() => options.onControllerReady?.(null));

  return {
    actions,
    feedbackMsg,
    isLoading,
    handleExecute,
    activeIndex,
  };
}
