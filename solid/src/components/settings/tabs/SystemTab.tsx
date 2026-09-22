import { Component, For, Show } from 'solid-js';
import type { PerformanceMode } from '@contracts/game.types';
import type { SystemTabProps } from '@contracts/settings.types';
import { SettingCardRow } from '@components/common/SettingCardRow';
import { Badge } from '@components/common/Badge';
import { SettingSwitch } from '../SettingSwitch';

const PERFORMANCE_MODES_LIST: readonly PerformanceMode[] = [
  'high-performance',
  'balanced',
  'power-saver',
  'ultra-boost'
] as const;

export const SystemTab: Component<SystemTabProps> = (props) => {
  const handleRotatePerformanceMode = () => {
    props.onSelectContentArea?.();
    props.onUpdateSettings((s) => {
      const current = s.system?.performanceMode || 'high-performance';
      const curIdx = PERFORMANCE_MODES_LIST.indexOf(current as PerformanceMode);
      const nextMode = PERFORMANCE_MODES_LIST[(curIdx + 1) % PERFORMANCE_MODES_LIST.length];
      if (!s.system) s.system = {};
      s.system.performanceMode = nextMode;
    });
  };

  const currentMode = () => props.settings?.system?.performanceMode || 'high-performance';

  const getModeBadgeVariant = () => {
    switch (currentMode()) {
      case 'ultra-boost':
        return 'boost';
      case 'high-performance':
        return 'highlight';
      default:
        return 'default';
    }
  };

  return (
    <div class="settings-tab-panel">
      <div class="panel-header-block">
        <div class="panel-header-titles">
          <h3 class="panel-section-title">Sistema y pantalla</h3>
        </div>
      </div>

      <div class="settings-form-stack">
        {/* Row 0: Performance Mode */}
        <SettingCardRow
          title="Preferencia de rendimiento"
          description="Perfil preferido de la consola"
          isFocused={props.isRowFocused(0)}
          onClick={handleRotatePerformanceMode}
        >
          <Badge variant={getModeBadgeVariant()}>
            {currentMode().toUpperCase()} [A]
          </Badge>
        </SettingCardRow>

        <SettingSwitch
          title="Sincronización Vertical (VSync)"
          description="Preferencia de sincronizacion de imagen"
          checked={props.settings?.display?.vsync ?? true}
          isFocused={props.isRowFocused(1)}
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
          <div><dt>Resolucion configurada</dt><dd>{props.settings?.display.resolution || 'No disponible'}</dd></div>
          <div><dt>Frecuencia configurada</dt><dd>{props.settings?.display.refreshRate ? `${props.settings.display.refreshRate} Hz` : 'No disponible'}</dd></div>
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
