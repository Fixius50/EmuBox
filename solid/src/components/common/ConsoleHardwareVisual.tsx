import { Component } from 'solid-js';
import type { PlatformId } from '@contracts/game.types';

interface ConsoleHardwareVisualProps {
  platformId: PlatformId | string;
  class?: string;
  size?: 'sm' | 'md' | 'lg' | 'xl';
}

export const ConsoleHardwareVisual: Component<ConsoleHardwareVisualProps> = (props) => {
  const sizeClass = () => {
    switch (props.size) {
      case 'sm': return 'console-hw-sm';
      case 'md': return 'console-hw-md';
      case 'lg': return 'console-hw-lg';
      case 'xl': return 'console-hw-xl';
      default: return 'console-hw-md';
    }
  };

  // Clasificación por compañía / fabricante para unificar y optimizar iconos
  const getCompanyFamily = (id: string): 'sony' | 'nintendo' | 'sega' | 'arcade' | 'settings' | 'generic' => {
    const lower = id.toLowerCase();
    if (lower.startsWith('ps') || lower === 'psp') return 'sony';
    if (['snes', 'n64', 'gamecube', 'gba', 'nds', 'nes', 'gb', 'gbc', '3ds', 'wii', 'wiiu', 'switch'].includes(lower)) return 'nintendo';
    if (['genesis', 'dreamcast', 'mastersystem', 'saturn', 'gamegear', 'segacd'].includes(lower)) return 'sega';
    if (['arcade', 'mame', 'fbneo', 'neogeo', 'cps1', 'cps2', 'cps3'].includes(lower)) return 'arcade';
    if (['settings', 'system-settings'].includes(lower)) return 'settings';
    return 'generic';
  };

  const family = () => getCompanyFamily(props.platformId);

  return (
    <div class={`console-minimal-icon-container ${sizeClass()} ${props.class || ''}`}>
      {(() => {
        switch (family()) {
          case 'sony':
            return (
              <svg viewBox="0 0 120 100" fill="none" xmlns="http://www.w3.org/2000/svg" class="console-minimal-svg">
                {/* Iconic Sony PlayStation Minimal Vector Logo */}
                <path d="M50 14 V86 L65 78 V26 L50 14 Z" fill="#006FCD" />
                <path d="M50 14 C72 8, 88 18, 88 34 C88 50, 70 54, 50 52" stroke="#e52521" stroke-width="10" stroke-linecap="round" fill="none" />
                <path d="M26 70 C36 62, 60 60, 74 66 C86 72, 84 84, 70 86 C54 88, 36 80, 26 74" stroke="#ffcc00" stroke-width="8" stroke-linecap="round" fill="none" />
                <path d="M62 64 C74 64, 86 72, 86 80" stroke="#00a859" stroke-width="6" stroke-linecap="round" fill="none" />
              </svg>
            );

          case 'nintendo':
            return (
              <svg viewBox="0 0 120 100" fill="none" xmlns="http://www.w3.org/2000/svg" class="console-minimal-svg">
                {/* Iconic Nintendo Pill Capsule Minimal Vector */}
                <rect x="15" y="26" width="90" height="48" rx="24" stroke="#e60012" stroke-width="6" fill="none" />
                <text x="60" y="56" font-family="'Rajdhani', sans-serif" font-weight="900" font-size="16" fill="#ffffff" text-anchor="middle" letter-spacing="1.5">Nintendo</text>
              </svg>
            );

          case 'sega':
            return (
              <svg viewBox="0 0 120 100" fill="none" xmlns="http://www.w3.org/2000/svg" class="console-minimal-svg">
                {/* Iconic Sega Minimal Stripe Typography */}
                <text x="60" y="58" font-family="'Rajdhani', sans-serif" font-weight="900" font-size="28" fill="#0088cc" text-anchor="middle" letter-spacing="2">SEGA</text>
                <line x1="24" y1="68" x2="96" y2="68" stroke="#00d2ff" stroke-width="4.5" stroke-linecap="round" />
              </svg>
            );

          case 'arcade':
            return (
              <svg viewBox="0 0 120 100" fill="none" xmlns="http://www.w3.org/2000/svg" class="console-minimal-svg">
                {/* Minimalist Arcade Coin-Op Emblem */}
                <rect x="30" y="54" width="60" height="26" rx="6" stroke="#f59e0b" stroke-width="5" fill="none" />
                <line x1="46" y1="54" x2="46" y2="30" stroke="#ffffff" stroke-width="6" stroke-linecap="round" />
                <circle cx="46" cy="24" r="9" fill="#ff0055" />
                <circle cx="68" cy="67" r="4" fill="#00ffff" />
                <circle cx="78" cy="63" r="4" fill="#ffcc00" />
              </svg>
            );

          case 'settings':
            return (
              <svg viewBox="0 0 120 100" fill="none" xmlns="http://www.w3.org/2000/svg" class="console-minimal-svg">
                {/* Minimalist Precision Settings Gear */}
                <circle cx="60" cy="50" r="28" stroke="#10b981" stroke-width="7" stroke-dasharray="14 8" fill="none" />
                <circle cx="60" cy="50" r="14" stroke="#10b981" stroke-width="4" fill="none" />
              </svg>
            );

          default:
            return (
              <svg viewBox="0 0 120 100" fill="none" xmlns="http://www.w3.org/2000/svg" class="console-minimal-svg">
                {/* Universal Gamepad Controller */}
                <rect x="25" y="32" width="70" height="42" rx="16" stroke="#00d2ff" stroke-width="5" fill="none" />
                <path d="M38 48 H48 M43 43 V53" stroke="#ffffff" stroke-width="3" stroke-linecap="round" />
                <circle cx="74" cy="53" r="3" fill="#ffffff" />
                <circle cx="82" cy="45" r="3" fill="#ffffff" />
              </svg>
            );
        }
      })()}
    </div>
  );
};
