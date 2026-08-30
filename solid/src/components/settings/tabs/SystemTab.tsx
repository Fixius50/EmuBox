import { Component } from 'solid-js';
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
          <h3 class="panel-section-title">Ajustes del Sistema y Motor Gráfico</h3>
          <p class="panel-section-desc">Configuración de bajo nivel de EmuBox OS en Arch Linux</p>
        </div>
      </div>

      <div class="settings-form-stack">
        {/* Row 0: Performance Mode */}
        <SettingCardRow
          title="Modo de Rendimiento del Kernel"
          description="Ajusta el perfil de escalado de frecuencia de CPU y GPU"
          isFocused={props.isRowFocused(0)}
          onClick={handleRotatePerformanceMode}
        >
          <Badge variant={getModeBadgeVariant()}>
            {currentMode().toUpperCase()} [A]
          </Badge>
        </SettingCardRow>

        {/* Row 1: Pipeline Gráfico */}
        <SettingCardRow
          title="Pipeline Gráfico Adaptativo"
          description="Gamescope para GPU acelerada / Cage para renderizado por software"
          isFocused={props.isRowFocused(1)}
          onClick={props.onSelectContentArea}
        >
          <Badge variant="highlight">
            AUTO (HARDWARE KMS)
          </Badge>
        </SettingCardRow>

        {/* Row 2: Resolution */}
        <SettingCardRow
          title="Resolución y Geometría"
          description="Adaptación automática en caliente ante cambios en la salida DRM"
          isFocused={props.isRowFocused(2)}
          onClick={props.onSelectContentArea}
        >
          <Badge variant="default">
            DINÁMICA (AUTO-AJUSTE)
          </Badge>
        </SettingCardRow>

        {/* Row 3: VSync */}
        <SettingSwitch
          title="Sincronización Vertical (VSync)"
          description="Elimina el desgarro de pantalla mediante sincronización KMS"
          checked={props.settings?.display?.vsync ?? true}
          isFocused={props.isRowFocused(3)}
          onChange={(val) => {
            props.onSelectContentArea?.();
            props.onUpdateSettings((s) => {
              s.display.vsync = val;
            });
          }}
        />

        {/* Row 4: Auto-Updates */}
        <SettingSwitch
          title="Actualizaciones Automáticas del Sistema"
          description="Comprobación y aplicación atómica de versiones estables en arranque"
          checked={props.settings?.updates?.autoUpdate ?? true}
          isFocused={props.isRowFocused(4)}
          onChange={(val) => {
            props.onSelectContentArea?.();
            props.onUpdateSettings((s) => {
              if (!s.updates) {
                s.updates = { autoUpdate: true, channel: 'stable', checkOnStartup: true };
              }
              s.updates.autoUpdate = val;
            });
          }}
        />
      </div>
    </div>
  );
};
