import { Component, For } from 'solid-js';
import type { EmulatorsTabProps } from '@contracts/settings.types';
import { Badge } from '@components/common/Badge';
import { ConsoleButton } from '@components/common/ConsoleButton';

export const EmulatorsTab: Component<EmulatorsTabProps> = (props) => {
  return (
    <div class="settings-tab-panel">
      <div class="panel-header-block">
        <div class="panel-header-titles">
          <h3 class="panel-section-title">Gestor de Motores y Núcleos de Emulación</h3>
          <p class="panel-section-desc">Binarios y núcleos compilados para ejecución directa bajo Gamescope</p>
        </div>
        <div class="settings-header-actions-row">
          <ConsoleButton
            label="+ AÑADIR NÚCLEO"
            variant="action"
            onClick={() => {
              props.onSelectContentArea?.();
              props.onOpenEditModal();
            }}
          />
        </div>
      </div>

      <div class="cyber-emulator-deck-grid">
        <For each={props.emulators || []}>
          {(emu, idx) => {
            const isFocused = () => props.isRowFocused(idx());

            return (
              <div
                class={`cyber-emulator-blade ${isFocused() ? 'focused' : ''}`}
                onClick={() => {
                  props.onSelectContentArea?.();
                  props.onOpenEditModal(emu);
                }}
              >
                <div class="blade-main-meta">
                  <div class="blade-engine-icon-badge">
                    <span style={{ "font-size": "0.75rem", "font-weight": "900", color: "#00f0ff" }}>
                      {emu.coreType === 'libretro' ? 'CORE' : 'BIN'}
                    </span>
                  </div>
                  <div class="blade-details-col">
                    <div class="blade-engine-name">
                      <span>{emu.name}</span>
                      <span class={`blade-type-chip ${emu.coreType === 'standalone' ? 'standalone' : ''}`}>
                        {emu.coreType === 'libretro' ? 'Libretro Core' : 'Standalone Vulkan'}
                      </span>
                      <Badge variant="default" style={{ "font-size": "0.625rem", padding: "0.125rem 0.5rem" }}>
                        v{emu.version}
                      </Badge>
                    </div>
                    <div class="blade-exec-path">
                      {emu.executable} {emu.arguments.join(' ')}
                    </div>
                    <div class="blade-platform-chips-row">
                      <For each={emu.supportedPlatforms}>
                        {(plat) => <Badge variant="chip">{plat}</Badge>}
                      </For>
                    </div>
                  </div>
                </div>

                <div class="blade-actions-col">
                  <button
                    class="blade-btn-edit"
                    onClick={(e) => {
                      e.stopPropagation();
                      props.onSelectContentArea?.();
                      props.onOpenEditModal(emu);
                    }}
                  >
                    <span>[A] CONFIGURAR</span>
                  </button>
                </div>
              </div>
            );
          }}
        </For>
      </div>
    </div>
  );
};
