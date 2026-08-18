import { Component, For } from 'solid-js';
import type { Emulator } from '@contracts/game.types';

interface EmulatorCardProps {
  emulator: Emulator;
}

export const EmulatorCard: Component<EmulatorCardProps> = (props) => {
  const isReady = () => props.emulator.status === 'active';

  return (
    <div class="emulator-card" id={`emu-card-${props.emulator.id}`}>
      <div class="emulator-card-header">
        <div class="emulator-name-pane">
          <div class="emulator-name">{props.emulator.name}</div>
          <div class="emulator-version">v{props.emulator.version} • {props.emulator.coreType}</div>
        </div>
        <span class={`status-badge ${isReady() ? 'ready' : 'pending'}`}>
          {isReady() ? '● LISTO' : `○ ${props.emulator.status.toUpperCase()}`}
        </span>
      </div>

      <div class="emulator-binary-path">
        <code>{props.emulator.executable} {props.emulator.arguments.join(' ')}</code>
      </div>

      <div class="emulator-platforms-row">
        <span class="platforms-label">Sistemas Soportados:</span>
        <div class="platform-tags">
          <For each={props.emulator.supportedPlatforms}>
            {(plat) => <span class="platform-mini-tag">{plat.toUpperCase()}</span>}
          </For>
        </div>
      </div>
    </div>
  );
};
