import type { DownloadJob } from "@contracts/download.types";
import type { Game } from "@contracts/game.types";
import type { InputAction, InputDeviceStatus } from "@contracts/input.types";
import type { LibraryStore } from "@stores/library.store";

export interface DownloadSourceModalProps {
  store: LibraryStore;
  inputStatus: InputDeviceStatus;
  onConfirm: () => void;
  onPlay?: (game: Game) => void;
  playBlockReason?: string | null;
  searchDisabled?: boolean;
  onControllerReady?: (handler: ((action: InputAction) => void) | null) => void;
}

export function downloadModalJobCommand(
  action: InputAction,
  status?: DownloadJob["status"],
) {
  if (action === "BUTTON_START") {
    if (status === "cancelled") return "retry";
    if (status === "downloading" || status === "queued") return "pause";
    if (status === "paused" || status === "failed" || status === "downloaded")
      return "resume";
  }
  if (action === "BUTTON_SELECT") {
    if (status === "cancelled") return "delete";
    if (
      status === "downloading" ||
      status === "queued" ||
      status === "paused" ||
      status === "failed"
    )
      return "cancel";
  }
  return null;
}

/**
 * DownloadSourceModal ha sido sustituido íntegramente por el interior de la carátula
 * en el cajón lateral derecho (.xmb-detail-drawer).
 */
export function DownloadSourceModal(_props: DownloadSourceModalProps) {
  return null;
}
