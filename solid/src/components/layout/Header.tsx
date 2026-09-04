import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import type { InputDeviceStatus } from '@contracts/input.types';

interface HeaderProps {
  inputStatus: InputDeviceStatus;
  totalGamesCount: number;
  currentSection: 'library' | 'settings';
  onNavigate: (section: 'library' | 'settings') => void;
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

      <nav class="console-main-navigation" aria-label="Navegación principal">
        <button
          class={`nav-blade-btn ${props.currentSection === 'library' ? 'active' : ''}`}
          onClick={() => props.onNavigate('library')}
        >
          Juegos
        </button>
        <button
          class={`nav-blade-btn ${props.currentSection === 'settings' ? 'active' : ''}`}
          onClick={() => props.onNavigate('settings')}
        >
          Ajustes
        </button>
      </nav>

      {/* 3. Right: Device Controller Status & Real-time Clock */}
      <div class="console-telemetry-cluster">
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
