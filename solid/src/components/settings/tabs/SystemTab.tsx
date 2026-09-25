import { Component, For, Show } from 'solid-js';
import type { SystemTabProps } from '@contracts/settings.types';
import { SettingSwitch } from '../SettingSwitch';

export const SystemTab: Component<SystemTabProps> = (props) => {
  return (
    <div class="settings-tab-panel">
      <div class="panel-header-block">
        <div class="panel-header-titles">
          <h3 class="panel-section-title">Sistema y pantalla</h3>
        </div>
      </div>

      <div class="settings-form-stack">
        <SettingSwitch
          title="Sincronización Vertical (VSync)"
          description="Preferencia de sincronizacion de imagen"
          checked={props.settings?.display?.vsync ?? true}
          isFocused={props.isRowFocused(0)}
          onChange={(val) => {
            props.onSelectContentArea?.();
            props.onUpdateSettings((s) => {
              s.display.vsync = val;
            });
          }}
        />
      </div>
      <section class="settings-information" aria-label="Informacion del sistema">
        <h4>Informacion del sistema</h4>
        <dl>
          <div><dt>Rendimiento</dt><dd>Adaptativo segun recursos disponibles al iniciar</dd></div>
          <div><dt>Resolucion actual adaptativa</dt><dd>{props.displayInfo?.resolution || 'Detectando salida actual...'}</dd></div>
          <div><dt>Frecuencia actual adaptativa</dt><dd>{props.displayInfo?.refreshRate ? `${props.displayInfo.refreshRate} Hz` : 'Detectando salida actual...'}</dd></div>
          <div><dt>Compositor actual</dt><dd>{props.displayInfo?.activeCompositor || 'Detectando salida actual...'}</dd></div>
          <div><dt>Motores registrados</dt><dd>{props.emulators?.length ?? 0}</dd></div>
        </dl>
        <h4>Nucleos y emuladores</h4>
        <Show when={props.emulators?.length} fallback={<p>No hay motores registrados.</p>}>
          <ul class="settings-engine-list">
            <For each={props.emulators}>{emulator => (
              <li><strong>{emulator.name}</strong><span>{emulator.version || 'Version no disponible'} · {emulator.coreType === 'libretro' ? 'Nucleo Libretro' : 'Independiente'}</span></li>
            )}</For>
          </ul>
        </Show>
      </section>
    </div>
  );
};
