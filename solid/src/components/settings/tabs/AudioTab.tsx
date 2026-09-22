import { Component, For, Show } from 'solid-js';
import { RefreshCw } from 'lucide-solid';
import { useAudioDevices } from '@hooks/useAudioDevices';
import type { AudioTabProps } from '@contracts/settings.types';
import { SettingSwitch } from '../SettingSwitch';
import { SettingSlider } from '../SettingSlider';

export const AudioTab: Component<AudioTabProps> = (props) => {
  const audio = useAudioDevices(props);
  return (
    <div class="settings-tab-panel">
      <div class="panel-header-block">
        <div class="panel-header-titles">
          <h3 class="panel-section-title">Audio general</h3>
        </div>
        <button class="xmb-icon-button" aria-label="Detectar dispositivos de audio" title="Detectar dispositivos de audio"
          disabled={audio.loading()} onClick={() => void audio.refresh()}><RefreshCw size={18} /></button>
      </div>

      <div class="settings-form-stack">
        <For each={[
          { key: 'inputDevice' as const, type: 'source', label: 'Audio de entrada', row: 0 },
          { key: 'outputDevice' as const, type: 'sink', label: 'Audio de salida', row: 1 },
        ]}>{device => (
          <label class="settings-device-row" data-setting-row={device.row} classList={{ focused: props.isRowFocused(device.row) }}>
            <span>{device.label}</span>
            <select aria-label={device.label} disabled={audio.loading()}
              value={props.settings?.audio[device.key] || 'default'}
              onFocus={props.onSelectContentArea}
              onKeyDown={event => { if (event.key !== 'Escape') event.stopPropagation(); }}
              onChange={event => { const id = event.currentTarget.value; props.onUpdateSettings(settings => { settings.audio[device.key] = id; }); }}>
              <option value="default">Automatico (sistema)</option>
              <For each={audio.devices().filter(entry => entry.type === device.type)}>{entry => (
                <option value={entry.id}>{entry.name}{entry.isDefault ? ' (predeterminado)' : ''}</option>
              )}</For>
            </select>
          </label>
        )}</For>
        <Show when={audio.loading()}><p role="status">Detectando dispositivos...</p></Show>
        <Show when={audio.error()}><p role="alert">{audio.error()}</p></Show>
        <Show when={audio.limited()}><p class="settings-device-note">El navegador limita la lista de dispositivos sin permisos. Se mantiene el audio predeterminado del sistema.</p></Show>
        <SettingSwitch
          title="Efectos Sonoros de la Interfaz"
          description="Retroalimentación acústica sintetizada para navegación y selección"
          checked={props.settings?.audio?.uiSoundEffects ?? true}
          isFocused={props.isRowFocused(2)}
          onChange={(val) => {
            props.onSelectContentArea?.();
            props.onUpdateSettings((s) => {
              s.audio.uiSoundEffects = val;
            });
          }}
        />

        <SettingSlider
          title="Volumen Maestro Global"
          description="Nivel general de salida de audio para juegos e interfaz"
          value={props.settings?.audio?.masterVolume ?? 80}
          min={0}
          max={100}
          unit="%"
          isFocused={props.isRowFocused(3)}
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
