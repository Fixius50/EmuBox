import { Component } from 'solid-js';
import type { AudioTabProps } from '@contracts/settings.types';
import { SettingSwitch } from '../SettingSwitch';
import { SettingSlider } from '../SettingSlider';

export const AudioTab: Component<AudioTabProps> = (props) => {
  return (
    <div class="settings-tab-panel">
      <div class="panel-header-block">
        <div class="panel-header-titles">
          <h3 class="panel-section-title">Configuración de Audio y Efectos</h3>
          <p class="panel-section-desc">Sintetizador WebAudio de foco y ganancia maestra global</p>
        </div>
      </div>

      <div class="settings-form-stack">
        {/* Row 0: UI Sound Effects */}
        <SettingSwitch
          title="Efectos Sonoros de la Interfaz"
          description="Retroalimentación acústica sintetizada para navegación y selección"
          checked={props.settings?.audio?.uiSoundEffects ?? true}
          isFocused={props.isRowFocused(0)}
          onChange={(val) => {
            props.onSelectContentArea?.();
            props.onUpdateSettings((s) => {
              s.audio.uiSoundEffects = val;
            });
          }}
        />

        {/* Row 1: Master Volume Slider */}
        <SettingSlider
          title="Volumen Maestro Global"
          description="Nivel general de salida de audio para juegos e interfaz"
          value={props.settings?.audio?.masterVolume ?? 80}
          min={0}
          max={100}
          unit="%"
          isFocused={props.isRowFocused(1)}
          onChange={(val) => {
            props.onSelectContentArea?.();
            props.onUpdateSettings((s) => {
              s.audio.masterVolume = val;
            });
          }}
        />
      </div>
    </div>
  );
};
