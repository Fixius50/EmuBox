import { createSignal } from 'solid-js';
import type { Game } from '@contracts/game.types';
import type { SpatialNavigatorService } from '@services/navigation/spatial-navigator.service';

export function createModalStore(navigator?: SpatialNavigatorService) {
  const [isDetailsOpen, setIsDetailsOpen] = createSignal<boolean>(false);
  const [isConfirmLaunchOpen, setIsConfirmLaunchOpen] = createSignal<boolean>(false);
  const [isEmulatorSelectorOpen, setIsEmulatorSelectorOpen] = createSignal<boolean>(false);
  const [isMaintenanceOpen, setIsMaintenanceOpen] = createSignal<boolean>(false);
  const [maintenanceIndex, setMaintenanceIndex] = createSignal<number>(0);
  const [selectedGame, setSelectedGame] = createSignal<Game | null>(null);

  const openGameDetails = (game: Game) => {
    setSelectedGame(game);
    setIsDetailsOpen(true);
    navigator?.pushContainer('modal', true);
  };

  const closeGameDetails = () => {
    setIsDetailsOpen(false);
    navigator?.popContainer();
  };

  const openConfirmLaunch = (game: Game) => {
    setSelectedGame(game);
    setIsConfirmLaunchOpen(true);
    navigator?.pushContainer('confirm-launch', true);
  };

  const closeConfirmLaunch = () => {
    setIsConfirmLaunchOpen(false);
    navigator?.popContainer();
  };

  const openEmulatorSelector = (game: Game) => {
    setSelectedGame(game);
    setIsEmulatorSelectorOpen(true);
  };

  const closeEmulatorSelector = () => {
    setIsEmulatorSelectorOpen(false);
  };

  const openMaintenance = () => {
    setMaintenanceIndex(0);
    setIsMaintenanceOpen(true);
  };

  const closeMaintenance = () => {
    setIsMaintenanceOpen(false);
  };

  return {
    isDetailsOpen,
    isConfirmLaunchOpen,
    isEmulatorSelectorOpen,
    setIsEmulatorSelectorOpen,
    isMaintenanceOpen,
    setIsMaintenanceOpen,
    maintenanceIndex,
    setMaintenanceIndex,
    selectedGame,
    setSelectedGame,
    openGameDetails,
    closeGameDetails,
    openConfirmLaunch,
    closeConfirmLaunch,
    openEmulatorSelector,
    closeEmulatorSelector,
    openMaintenance,
    closeMaintenance
  };
}

export type ModalStore = ReturnType<typeof createModalStore>;
