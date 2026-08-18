import { Component, For } from 'solid-js';
import { EmulatorCard } from './EmulatorCard';
import type { Emulator } from '@contracts/game.types';

interface EmulatorsViewProps {
  emulators: Emulator[];
  onBack?: () => void;
}

export const EmulatorsView: Component<EmulatorsViewProps> = (props) => {
  return (
    <div class="console-emulators-container">
      {props.onBack && (
        <div class="system-view-subbar" style={{ "padding-left": "0", "margin-bottom": "1rem" }}>
          <button
            class="back-to-wheel-btn"
            onClick={(e) => {
              e.stopPropagation();
              props.onBack?.();
            }}
          >
            <span class="back-key-bubble">B</span>
            <span>MENÚ PRINCIPAL</span>
          </button>
        </div>
      )}

      <div class="emulators-hero-header">
        <div class="header-badge-chip">NÚCLEOS NATIVOS</div>
        <h2 class="emulators-title">Gestor de Motores de Emulación</h2>
        <p class="emulators-subtitle">
          Binarios y núcleos compilados para ejecución directa bajo Gamescope y DRM/KMS
        </p>
      </div>

      <div class="console-emulators-grid">
        <For each={props.emulators}>
          {(emu) => <EmulatorCard emulator={emu} />}
        </For>
      </div>
    </div>
  );
};
