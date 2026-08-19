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
          <p class="panel-section-desc">Configuración de bajo nivel para Arch Linux y Gamescope KMS</p>
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

        {/* Row 1: Vulkan Renderer */}
        <SettingCardRow
          title="Controlador Gráfico Vulkan"
          description="Pipeline Mesa RADV con soporte para shaders asíncronos"
          isFocused={props.isRowFocused(1)}
          onClick={props.onSelectContentArea}
        >
          <Badge variant="highlight">
            RADV VULKAN 1.3
          </Badge>
        </SettingCardRow>

        {/* Row 2: Resolution */}
        <SettingCardRow
          title="Resolución de Salida"
          description="Resolución nativa renderizada por Gamescope"
          isFocused={props.isRowFocused(2)}
          onClick={props.onSelectContentArea}
        >
          <Badge variant="default">
            1920x1080 @ 60HZ
          </Badge>
        </SettingCardRow>

        {/* Row 3: VSync */}
        <SettingSwitch
          title="Sincronización Vertical (VSync)"
          description="Elimina el desgarro de pantalla mediante triple buffering adaptativo"
          checked={props.settings?.display?.vsync ?? true}
          isFocused={props.isRowFocused(3)}
          onChange={(val) => {
            props.onSelectContentArea?.();
            props.onUpdateSettings((s) => {
              s.display.vsync = val;
            });
          }}
        />
      </div>
    </div>
  );
};
