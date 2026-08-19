import { Component, For, Show } from 'solid-js';
import type { GamepadTabProps } from '@contracts/settings.types';
import { useGamepadDevices } from '@hooks/useGamepadDevices';
import { SettingSwitch } from '../SettingSwitch';
import { SettingCardRow } from '@components/common/SettingCardRow';
import { Badge } from '@components/common/Badge';

export const GamepadTab: Component<GamepadTabProps> = (props) => {
  const { gamepadsList } = useGamepadDevices();

  return (
    <div class="settings-tab-panel">
      <div class="panel-header-block">
        <div class="panel-header-titles">
          <h3 class="panel-section-title">Mandos, Gamepads y Dispositivos de Entrada</h3>
          <p class="panel-section-desc">Detección de mandos en puertos 1-4 via Gilrs y Web Gamepad API</p>
        </div>
      </div>

      <div class="settings-form-stack">
        {/* Row 0: Rumble Switch */}
        <SettingSwitch
          title="Vibración y Respuesta Háptica"
          description="Efectos de vibración en mandos compatibles"
          checked={props.settings?.gamepad?.vibration ?? true}
          isFocused={props.isRowFocused(0)}
          onChange={(val) => {
            props.onSelectContentArea?.();
            props.onUpdateSettings((s) => {
              s.gamepad.vibration = val;
            });
          }}
        />

        {/* Row 1: Primary Controller */}
        <SettingCardRow
          title="Mando de Navegación Principal"
          description="Dispositivo asignado al Jugador 1 para controlar la consola"
          isFocused={props.isRowFocused(1)}
          onClick={props.onSelectContentArea}
        >
          <Badge variant="highlight">
            {gamepadsList().length > 0 ? gamepadsList()[0].id.slice(0, 24) : 'TECLADO / MANDO 1'}
          </Badge>
        </SettingCardRow>
      </div>

      {/* Connected Gamepads List */}
      <div class="gamepads-detected-section" style={{ "margin-top": "1.25rem" }}>
        <h4 class="gamepads-section-title">Puertos y Dispositivos Físicos Detectados</h4>

        <div class="gamepad-cards-grid">
          <For each={gamepadsList()}>
            {(pad, idx) => {
              const rowIndex = () => 2 + idx();
              const isFocused = () => props.isRowFocused(rowIndex());

              return (
                <div
                  class={`gamepad-device-card ${isFocused() ? 'focused' : ''}`}
                  onClick={() => {
                    props.onSelectContentArea?.();
                    props.onTriggerGamepadTest?.(pad.index);
                  }}
                >
                  <div class="gamepad-device-header">
                    <div class="gamepad-icon-circle" style={{ "font-size": "0.75rem", "font-weight": "800" }}>
                      P{pad.index + 1}
                    </div>
                    <div>
                      <div class="gamepad-device-title">{pad.id}</div>
                      <div class="gamepad-device-sub">
                        Puerto {pad.index + 1} • {pad.buttonsCount} Botones • {pad.axesCount} Ejes • {pad.hasVibration ? 'Háptica Soportada (Pulsar [A] para Test)' : 'Sin vibrador'}
                      </div>
                    </div>
                  </div>
                  <Badge variant="highlight">CONECTADO</Badge>
                </div>
              );
            }}
          </For>

          <Show when={gamepadsList().length === 0}>
            <div class="gamepad-device-card" style={{ opacity: "0.7" }}>
              <div class="gamepad-device-header">
                <div class="gamepad-icon-circle" style={{ "font-size": "0.75rem", "font-weight": "800" }}>PAD</div>
                <div>
                  <div class="gamepad-device-title">Esperando Mando Físico en Puertos 1 - 4...</div>
                  <div class="gamepad-device-sub">Conecta un mando Xbox, PlayStation o USB/Bluetooth para asignación directa</div>
                </div>
              </div>
              <Badge variant="default">EN ESPERA</Badge>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
};
