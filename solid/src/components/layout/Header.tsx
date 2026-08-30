import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import type { InputDeviceStatus } from '@contracts/input.types';

interface HeaderProps {
  inputStatus: InputDeviceStatus;
  totalGamesCount: number;
  onOpenTerminal?: () => void;
}

export const Header: Component<HeaderProps> = (props) => {
  const [timeString, setTimeString] = createSignal<string>('');
  const [dateString, setDateString] = createSignal<string>('');

  const updateClock = () => {
    const now = new Date();
    const hours = String(now.getHours()).padStart(2, '0');
    const minutes = String(now.getMinutes()).padStart(2, '0');
    setTimeString(`${hours}:${minutes}`);

    const options: Intl.DateTimeFormatOptions = { weekday: 'short', month: 'short', day: 'numeric' };
    setDateString(now.toLocaleDateString('es-ES', options));
  };

  onMount(() => {
    updateClock();
    const interval = setInterval(updateClock, 10000);
    onCleanup(() => clearInterval(interval));
  });

  return (
    <header class="console-dashboard-header">
      {/* 1. Left: Player Profile */}
      <div class="console-user-profile">
        <div class="user-avatar-hex">
          <span>EB</span>
          <div class="avatar-online-dot"></div>
        </div>
        <div class="user-info-text">
          <div class="user-handle">PLAYER 1</div>
          <div class="user-system-tag">EmuBox OS • Arch Linux</div>
        </div>
      </div>

      {/* 2. Center: Clean Minimalist Space */}
      <div class="header-center-spacer" />

      {/* 3. Right: Terminal Pill, Device Controller Status & Real-time Clock */}
      <div class="console-telemetry-cluster">
        {props.onOpenTerminal && (
          <button
            class="telemetry-chip"
            onClick={props.onOpenTerminal}
            title="Abrir Terminal de Diagnóstico y Red (F12)"
            style={{
              background: "rgba(0, 240, 255, 0.15)",
              border: "1px solid #00f0ff",
              color: "#00f0ff",
              cursor: "pointer",
              "font-weight": "700",
              gap: "0.35rem"
            }}
          >
            <span>💻</span>
            <span>TERMINAL / RED [F12]</span>
          </button>
        )}

        <div class="telemetry-chip input-status-chip">
          <span class="status-indicator-dot online"></span>
          <span class="chip-label">{props.inputStatus.deviceName}</span>
        </div>

        <div class="telemetry-clock-box">
          <span class="clock-digits">{timeString()}</span>
          <span class="clock-date-sub">{dateString()}</span>
        </div>
      </div>
    </header>
  );
};
