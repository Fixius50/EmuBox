import { Component, For, Show } from 'solid-js';
import type { UpdateTabProps } from '@contracts/settings.types';
import { useOtaUpdate } from '@hooks/useOtaUpdate';
import { SettingSwitch } from '../SettingSwitch';
import { Badge } from '@components/common/Badge';

export const UpdateTab: Component<UpdateTabProps> = (props) => {
  const {
    isCheckingUpdate,
    isUpdating,
    updateProgressVal,
    updateStatusMsg,
    handleCheckForUpdates,
    handleApplyUpdate,
    handleTriggerAction,
    setProgressBarRef
  } = useOtaUpdate({
    updateInfo: () => props.updateInfo,
    onCheckUpdates: props.onCheckUpdates,
    onApplyUpdate: props.onApplyUpdate,
    onSelectContentArea: props.onSelectContentArea
  });

  return (
    <div class="settings-tab-panel">
      <div class="panel-header-block">
        <div class="panel-header-titles">
          <h3 class="panel-section-title">Actualización OTA & Mantenimiento Desacoplado</h3>
          <p class="panel-section-desc">Actualiza la aplicación EmuBox automáticamente sin tocar Arch Linux ni tus ROMs y partidas</p>
        </div>
      </div>

      <div class="settings-form-stack">
        {/* Row 0: Auto-Update Switch */}
        <SettingSwitch
          title="Actualización Automática"
          description="Instala automáticamente nuevas versiones estables de EmuBox cuando estén disponibles"
          checked={props.settings?.updates?.autoUpdate ?? true}
          isFocused={props.isRowFocused(0)}
          onChange={(val) => {
            props.onSelectContentArea?.();
            props.onUpdateSettings((s) => {
              if (!s.updates) s.updates = { autoUpdate: true, channel: 'stable', checkOnStartup: true };
              s.updates.autoUpdate = val;
            });
          }}
        />
      </div>

      {/* Row 1: Update Hero Blade */}
      <div
        class={`update-hero-blade ${props.isRowFocused(1) ? 'focused' : ''}`}
        style={{ "margin-top": "0.5rem", cursor: "pointer" }}
        onClick={handleTriggerAction}
      >
        <div class="update-hero-main">
          <div class="update-version-title">
            <span>EmuBox OS {props.updateInfo?.currentVersion || 'v1.0.0'}</span>
            <span class={`update-status-pill ${props.updateInfo?.hasUpdate ? 'has-update' : ''}`}>
              {props.updateInfo?.hasUpdate ? 'ACTUALIZACIÓN DISPONIBLE (v1.0.1)' : 'SISTEMA AL DÍA'}
            </span>
          </div>
          <div class="update-meta-text">
            Canal: <strong style={{ color: "#00f0ff" }}>GitHub Releases (Estable)</strong> • Estado: {props.settings?.updates?.autoUpdate ?? true ? 'Auto-Update Activado' : 'Manual'}
          </div>
          <Show when={updateStatusMsg()}>
            <div class="update-meta-text" style={{ color: "#00f0ff", "font-weight": "800", "margin-top": "0.25rem" }}>
              {updateStatusMsg()}
            </div>
          </Show>
        </div>

        <div class="settings-header-actions-row">
          <Show
            when={props.updateInfo?.hasUpdate}
            fallback={
              <button
                class="settings-action-btn"
                onClick={(e) => {
                  e.stopPropagation();
                  props.onSelectContentArea?.();
                  handleCheckForUpdates();
                }}
                disabled={isCheckingUpdate() || isUpdating()}
              >
                <span>{isCheckingUpdate() ? 'BUSCANDO...' : '[A] BUSCAR EN GITHUB'}</span>
              </button>
            }
          >
            <button
              class="settings-action-btn"
              style={{ background: "linear-gradient(135deg, rgba(16, 185, 129, 0.4) 0%, rgba(0, 240, 255, 0.6) 100%)", "border-color": "#10b981" }}
              onClick={(e) => {
                e.stopPropagation();
                props.onSelectContentArea?.();
                handleApplyUpdate();
              }}
              disabled={isUpdating()}
            >
              <span>{isUpdating() ? 'ACTUALIZANDO...' : '[A] APLICAR v1.0.1 AHORA'}</span>
            </button>
          </Show>
        </div>
      </div>

      {/* Progress Bar */}
      <Show when={isUpdating()}>
        <div class="update-progress-container">
          <div class="update-progress-bar-fill" ref={setProgressBarRef} style={{ width: `${updateProgressVal()}%` }} />
        </div>
      </Show>

      {/* Changelog Card */}
      <Show when={props.updateInfo?.hasUpdate}>
        <div class="update-changelog-card">
          <div class="update-changelog-title">
            <span>Novedades en EmuBox {props.updateInfo?.latestVersion || 'v1.0.1'} ({props.updateInfo?.releaseDate || 'Reciente'})</span>
          </div>
          <div class="update-changelog-list">
            <For each={props.updateInfo?.releaseNotes || []}>
              {(note) => <div class="update-changelog-item">• {note}</div>}
            </For>
          </div>
        </div>
      </Show>

      {/* Safety Guarantees Grid */}
      <div class="safety-guarantee-grid">
        <div class="safety-guarantee-card">
          <Badge variant="default" style={{ "font-size": "0.625rem" }}>ROMS</Badge>
          <div>
            <div class="safety-title">ROMs & BIOS Intactas</div>
            <div class="safety-desc">/var/lib/emubox/games</div>
          </div>
        </div>
        <div class="safety-guarantee-card">
          <Badge variant="default" style={{ "font-size": "0.625rem" }}>SAVES</Badge>
          <div>
            <div class="safety-title">Partidas & Saves Seguras</div>
            <div class="safety-desc">/var/lib/emubox/saves</div>
          </div>
        </div>
        <div class="safety-guarantee-card">
          <Badge variant="default" style={{ "font-size": "0.625rem" }}>OS</Badge>
          <div>
            <div class="safety-title">Arch Linux Intacto</div>
            <div class="safety-desc">Drivers y Kernel aislados</div>
          </div>
        </div>
      </div>
    </div>
  );
};
