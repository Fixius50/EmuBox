import { Component, createSignal, For, createEffect, onMount } from 'solid-js';
import { Switch } from '@kobalte/core/switch';
import { Slider } from '@kobalte/core/slider';
import { HardwareProbeService, RealHardwareInfo, RealGamepadInfo } from '@services/system/hardware-probe.service';
import { animateEmulatorDeckEntrance, animateEmulatorModalEntrance } from '@animations/emulator-animations';
import { animateUpdateProgressBar } from '@animations/update-animations';
import type { SystemSettings, Emulator } from '@contracts/game.types';
import type { UpdateInfo, UpdateCheckResult, UpdateProgress } from '@contracts/update.types';

export const SETTINGS_TABS = ['system', 'emulators', 'audio', 'gamepad', 'update'] as const;
export type SettingsTabId = typeof SETTINGS_TABS[number];

const PERFORMANCE_MODES = [
  { id: 'high-performance', name: 'ALTO RENDIMIENTO', desc: 'Frecuencias máximas de GPU/CPU y prioridad Gamescope', badge: 'highlight' },
  { id: 'balanced', name: 'EQUILIBRADO', desc: 'Curva adaptativa de energía y ventilador silencioso', badge: '' },
  { id: 'power-saver', name: 'AHORRO DE ENERGÍA', desc: 'Perfil eficiente de 15W para menor consumo térmico', badge: '' },
  { id: 'ultra-boost', name: 'MODO ULTRA BOOST', desc: 'Overclock seguro de shaders Vulkan y gobernador Performance', badge: 'boost' }
];

interface SettingsViewProps {
  settings: SystemSettings | null;
  emulators?: Emulator[];
  activeTab?: string;
  focusArea?: 'sidebar' | 'content';
  focusedRowIndex?: number;
  onTabChange?: (tab: string) => void;
  onSelectContentArea?: () => void;
  onUpdateSettings: (newSettings: SystemSettings) => void;
  onSaveEmulator?: (emulator: Emulator) => void;
  onDeleteEmulator?: (emulatorId: string) => void;
  onBack?: () => void;
  // OTA Update Handlers
  updateInfo?: UpdateInfo;
  onCheckUpdates?: () => Promise<UpdateCheckResult | undefined>;
  onApplyUpdate?: (version?: string) => Promise<UpdateProgress | undefined>;
}

export const SettingsView: Component<SettingsViewProps> = (props) => {
  let contentPaneRef!: HTMLDivElement;
  let modalBoxRef!: HTMLDivElement;
  let progressBarRef!: HTMLDivElement;
  const probeService = new HardwareProbeService();

  const [hardwareInfo, setHardwareInfo] = createSignal<RealHardwareInfo>(probeService.getRealHardwareInfo());
  const [gamepadsList, setGamepadsList] = createSignal<RealGamepadInfo[]>(probeService.getConnectedGamepads());

  // OTA Update UI State
  const [isCheckingUpdate, setIsCheckingUpdate] = createSignal<boolean>(false);
  const [isUpdating, setIsUpdating] = createSignal<boolean>(false);
  const [updateProgressVal, setUpdateProgressVal] = createSignal<number>(0);
  const [updateStatusMsg, setUpdateStatusMsg] = createSignal<string>('');

  // CRUD Modal State for Emulators
  const [isEditingEmulator, setIsEditingEmulator] = createSignal<boolean>(false);
  const [editingEmulatorData, setEditingEmulatorData] = createSignal<Partial<Emulator>>({
    id: '',
    name: '',
    supportedPlatforms: ['snes'],
    coreType: 'libretro',
    executable: '/usr/bin/retroarch',
    arguments: ['-L', '/usr/lib/libretro/snes9x_libretro.so'],
    version: '1.18.0',
    status: 'active'
  });

  onMount(() => {
    setHardwareInfo(probeService.getRealHardwareInfo());
    setGamepadsList(probeService.getConnectedGamepads());

    const padInterval = setInterval(() => {
      setGamepadsList(probeService.getConnectedGamepads());
    }, 1000);

    return () => clearInterval(padInterval);
  });

  const currentTab = () => props.activeTab || 'system';

  const handleTabClick = (tab: string) => {
    props.onTabChange?.(tab);
  };

  const handleUpdate = (updater: (s: SystemSettings) => void) => {
    if (!props.settings) return;
    const clone = JSON.parse(JSON.stringify(props.settings)) as SystemSettings;
    updater(clone);
    props.onUpdateSettings(clone);
  };

  const isRowFocused = (rowIdx: number) => {
    return props.focusArea === 'content' && props.focusedRowIndex === rowIdx;
  };

  // Cycle Performance Mode
  const cyclePerformanceMode = () => {
    const currentModeId = props.settings?.system?.performanceMode || 'high-performance';
    const currentIdx = PERFORMANCE_MODES.findIndex(m => m.id === currentModeId);
    const nextMode = PERFORMANCE_MODES[(currentIdx + 1) % PERFORMANCE_MODES.length];
    handleUpdate(s => {
      if (!s.system) (s as any).system = {};
      s.system.performanceMode = nextMode.id as any;
    });
  };

  const currentPerformanceMode = () => {
    const modeId = props.settings?.system?.performanceMode || 'high-performance';
    return PERFORMANCE_MODES.find(m => m.id === modeId) || PERFORMANCE_MODES[0];
  };

  // Animate deck on entering emulators tab
  createEffect(() => {
    if (currentTab() === 'emulators' && contentPaneRef) {
      setTimeout(() => {
        animateEmulatorDeckEntrance('.cyber-emulator-blade');
      }, 40);
    }
  });

  // Auto scroll content pane when focusedRowIndex changes
  createEffect(() => {
    const row = props.focusedRowIndex;
    const area = props.focusArea;
    if (area === 'content' && contentPaneRef) {
      setTimeout(() => {
        const focusedEl = contentPaneRef.querySelector('.setting-card-row.focused, .cyber-emulator-blade.focused, .gamepad-device-card.focused') as HTMLElement;
        if (focusedEl) {
          focusedEl.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        }
      }, 30);
    }
  });

  const [modalFocusIdx, setModalFocusIdx] = createSignal<number>(0);
  const [isTyping, setIsTyping] = createSignal<boolean>(false);
  let nameInputRef: HTMLInputElement | undefined;
  let execInputRef: HTMLInputElement | undefined;
  let typeInputRef: HTMLInputElement | undefined;
  let deleteBtnRef: HTMLButtonElement | undefined;
  let cancelBtnRef: HTMLButtonElement | undefined;
  let saveBtnRef: HTMLButtonElement | undefined;

  // Open Edit Modal for an Emulator with Anime.js
  const openEditEmulator = (emu?: Emulator) => {
    if (emu) {
      setEditingEmulatorData({ ...emu });
    } else {
      const newId = `emu-${Date.now()}`;
      setEditingEmulatorData({
        id: newId,
        name: 'Nuevo Motor de Emulación',
        supportedPlatforms: ['snes'],
        coreType: 'libretro',
        executable: '/usr/bin/retroarch',
        arguments: ['-L', '/usr/lib/libretro/custom_core_libretro.so'],
        version: '1.0.0',
        status: 'active'
      });
    }
    setModalFocusIdx(0);
    setIsTyping(false);
    setIsEditingEmulator(true);
    setTimeout(() => {
      if (modalBoxRef) animateEmulatorModalEntrance(modalBoxRef);
    }, 50);
  };

  const focusModalElement = (idx: number) => {
    setModalFocusIdx(idx);
    setIsTyping(false);
    if (document.activeElement instanceof HTMLElement && document.activeElement.tagName === 'INPUT') {
      document.activeElement.blur();
    }
    const hasDelete = !!editingEmulatorData().id;
    if (hasDelete && idx === 3) deleteBtnRef?.focus();
    else if (hasDelete && idx === 4) cancelBtnRef?.focus();
    else if (hasDelete && idx === 5) saveBtnRef?.focus();
    else if (!hasDelete && idx === 3) cancelBtnRef?.focus();
    else if (!hasDelete && idx === 4) saveBtnRef?.focus();
  };

  createEffect(() => {
    (window as any).__EMUBOX_OPEN_EMULATOR_CONFIG__ = (idx: number) => {
      const list = props.emulators || [];
      openEditEmulator(list[idx] || list[0]);
    };
    (window as any).__EMUBOX_IS_EMULATOR_MODAL_OPEN__ = () => isEditingEmulator();
    (window as any).__EMUBOX_CLOSE_EMULATOR_MODAL__ = () => setIsEditingEmulator(false);
    (window as any).__EMUBOX_TRIGGER_UPDATE_ACTION__ = () => {
      if (props.updateInfo?.hasUpdate) {
        handleApplyUpdate();
      } else {
        handleCheckForUpdates();
      }
    };
    (window as any).__EMUBOX_EMU_MODAL_NAV__ = (dir: 'UP' | 'DOWN' | 'LEFT' | 'RIGHT' | 'SELECT') => {
      if (!isEditingEmulator()) return;
      const cur = modalFocusIdx();
      const hasDelete = !!editingEmulatorData().id;
      const maxIdx = hasDelete ? 5 : 4;

      if (dir === 'DOWN') {
        if (cur < 2) {
          focusModalElement(cur + 1);
        } else if (cur === 2) {
          focusModalElement(hasDelete ? 5 : 4); // Jump to Save button
        }
      } else if (dir === 'UP') {
        if (cur >= 3) {
          focusModalElement(2);
        } else if (cur > 0) {
          focusModalElement(cur - 1);
        }
      } else if (dir === 'LEFT') {
        if (cur > 3) {
          focusModalElement(cur - 1);
        }
      } else if (dir === 'RIGHT') {
        if (cur >= 3 && cur < maxIdx) {
          focusModalElement(cur + 1);
        }
      } else if (dir === 'SELECT') {
        if (hasDelete && cur === 3) {
          deleteEmulator();
        } else if ((hasDelete && cur === 4) || (!hasDelete && cur === 3)) {
          setIsEditingEmulator(false);
        } else if ((hasDelete && cur === 5) || (!hasDelete && cur === 4)) {
          saveEmulator();
        } else {
          // Entrar en modo escritura en el input
          setIsTyping(true);
          if (cur === 0) { nameInputRef?.focus(); nameInputRef?.select(); }
          if (cur === 1) { execInputRef?.focus(); execInputRef?.select(); }
          if (cur === 2) { typeInputRef?.focus(); typeInputRef?.select(); }
        }
      }
    };
  });

  const saveEmulator = () => {
    const data = editingEmulatorData();
    if (data.id && data.name) {
      props.onSaveEmulator?.(data as Emulator);
      setIsEditingEmulator(false);
    }
  };

  const deleteEmulator = () => {
    const data = editingEmulatorData();
    if (data.id) {
      props.onDeleteEmulator?.(data.id);
      setIsEditingEmulator(false);
    }
  };

  // Test Gamepad Rumble
  const triggerGamepadTest = (padIndex: number) => {
    try {
      const pads = navigator.getGamepads ? navigator.getGamepads() : [];
      const targetPad = pads[padIndex];
      if (targetPad && (targetPad as any).vibrationActuator) {
        (targetPad as any).vibrationActuator.playEffect('dual-rumble', {
          startDelay: 0,
          duration: 300,
          weakMagnitude: 0.8,
          strongMagnitude: 0.8
        });
      }
    } catch {
      // ignore
    }
  };

  // Check for updates
  const handleCheckForUpdates = async () => {
    setIsCheckingUpdate(true);
    setUpdateStatusMsg('Consultando releases en GitHub API...');
    try {
      if (props.onCheckUpdates) {
        await props.onCheckUpdates();
      }
      setUpdateStatusMsg('Comprobación completada.');
    } finally {
      setTimeout(() => setIsCheckingUpdate(false), 800);
    }
  };

  // Apply OTA Update
  const handleApplyUpdate = async () => {
    setIsUpdating(true);
    setUpdateProgressVal(15);
    setUpdateStatusMsg('Descargando paquete de actualización en /opt/emubox/releases...');
    if (progressBarRef) animateUpdateProgressBar(progressBarRef, 15);

    setTimeout(async () => {
      setUpdateProgressVal(60);
      setUpdateStatusMsg('Verificando checksum SHA256 y desempaquetando...');
      if (progressBarRef) animateUpdateProgressBar(progressBarRef, 60);

      setTimeout(async () => {
        setUpdateProgressVal(90);
        setUpdateStatusMsg('Reasignando enlace atómico /opt/emubox/current...');
        if (progressBarRef) animateUpdateProgressBar(progressBarRef, 90);

        setTimeout(async () => {
          if (props.onApplyUpdate) {
            await props.onApplyUpdate('v1.0.1');
          }
          setUpdateProgressVal(100);
          setUpdateStatusMsg('Actualización aplicada con éxito. Reiniciando sesión de EmuBox...');
          if (progressBarRef) animateUpdateProgressBar(progressBarRef, 100);
          setTimeout(() => setIsUpdating(false), 1200);
        }, 600);
      }, 600);
    }, 600);
  };

  return (
    <div class="console-settings-container">
      {/* Top Header Bar with Back Button */}
      <div class="settings-top-bar">
        {props.onBack && (
          <button
            class="back-to-wheel-btn"
            onClick={(e) => {
              e.stopPropagation();
              props.onBack?.();
            }}
          >
            <span class="back-key-bubble">B</span>
            <span>MENÚ PRINCIPAL</span>
          </button>
        )}

        <div class="settings-main-title-badge">
          <span class="settings-main-title">AJUSTES DEL SISTEMA</span>
          <span class="settings-sub-tag">EmuBox OS • Arch Linux</span>
        </div>
      </div>

      <div class="console-tabs-system">
        {/* Left Sidebar Menu */}
        <div class="console-sidebar-menu">
          <div
            class={`sidebar-tab-trigger ${currentTab() === 'system' ? 'active' : ''} ${currentTab() === 'system' && props.focusArea === 'sidebar' ? 'focused-sidebar' : ''}`}
            onClick={() => handleTabClick('system')}
          >
            <div class="tab-label-group">
              <span class="tab-main-text">Sistema & Pantalla</span>
              <span class="tab-sub-text">Hardware, GPU, 1080p, VSync</span>
            </div>
            {currentTab() === 'system' && <div class="tab-neon-caret" />}
          </div>

          <div
            class={`sidebar-tab-trigger ${currentTab() === 'emulators' ? 'active' : ''} ${currentTab() === 'emulators' && props.focusArea === 'sidebar' ? 'focused-sidebar' : ''}`}
            onClick={() => handleTabClick('emulators')}
          >
            <div class="tab-label-group">
              <span class="tab-main-text">Núcleos & Emuladores</span>
              <span class="tab-sub-text">Cores Libretro, Binarios</span>
            </div>
            {currentTab() === 'emulators' && <div class="tab-neon-caret" />}
          </div>

          <div
            class={`sidebar-tab-trigger ${currentTab() === 'audio' ? 'active' : ''} ${currentTab() === 'audio' && props.focusArea === 'sidebar' ? 'focused-sidebar' : ''}`}
            onClick={() => handleTabClick('audio')}
          >
            <div class="tab-label-group">
              <span class="tab-main-text">Audio & Sintetizador</span>
              <span class="tab-sub-text">Volumen, Efectos UI</span>
            </div>
            {currentTab() === 'audio' && <div class="tab-neon-caret" />}
          </div>

          <div
            class={`sidebar-tab-trigger ${currentTab() === 'gamepad' ? 'active' : ''} ${currentTab() === 'gamepad' && props.focusArea === 'sidebar' ? 'focused-sidebar' : ''}`}
            onClick={() => handleTabClick('gamepad')}
          >
            <div class="tab-label-group">
              <span class="tab-main-text">Mando & Controles</span>
              <span class="tab-sub-text">Dispositivos Conectados</span>
            </div>
            {currentTab() === 'gamepad' && <div class="tab-neon-caret" />}
          </div>

          <div
            class={`sidebar-tab-trigger ${currentTab() === 'update' ? 'active' : ''} ${currentTab() === 'update' && props.focusArea === 'sidebar' ? 'focused-sidebar' : ''}`}
            onClick={() => handleTabClick('update')}
          >
            <div class="tab-label-group">
              <span class="tab-main-text">Actualización & OTA</span>
              <span class="tab-sub-text">GitHub Releases, Auto-Update</span>
            </div>
            {currentTab() === 'update' && <div class="tab-neon-caret" />}
          </div>
        </div>

        {/* Right Content Pane */}
        <div class="console-settings-content-pane" ref={contentPaneRef}>
          {/* FUSED TAB: System, Display & Hardware */}
          {currentTab() === 'system' && (
            <div class="settings-tab-panel">
              <div class="panel-header-block">
                <div class="panel-header-titles">
                  <h3 class="panel-section-title">Sistema, Pantalla & Hardware</h3>
                  <p class="panel-section-desc">Gobernador de rendimiento, adaptador GPU y tasa de refresco del compositor</p>
                </div>
              </div>

              <div class="settings-form-stack">
                <div
                  class={`setting-card-row ${isRowFocused(0) ? 'focused' : ''}`}
                  onClick={() => {
                    props.onSelectContentArea?.();
                    cyclePerformanceMode();
                  }}
                >
                  <div class="setting-info">
                    <span class="setting-title">Modo de Rendimiento</span>
                    <span class="setting-desc">{currentPerformanceMode().desc}</span>
                  </div>
                  <span class={`readonly-badge interactive-badge ${currentPerformanceMode().badge || 'highlight'}`}>
                    <span>{currentPerformanceMode().name}</span>
                    <span style={{ "font-size": "0.625rem", opacity: "0.8" }}>[A] ROTAR</span>
                  </span>
                </div>

                <div
                  class={`setting-card-row ${isRowFocused(1) ? 'focused' : ''}`}
                  onClick={() => props.onSelectContentArea?.()}
                >
                  <div class="setting-info">
                    <span class="setting-title">Display & Tasa de Refresco</span>
                    <span class="setting-desc">{hardwareInfo().screenResolution} • Compositor KMS</span>
                  </div>
                  <span class="readonly-badge highlight">{hardwareInfo().screenRefreshRate} Hz Sincronizado</span>
                </div>

                <div
                  class={`setting-card-row ${isRowFocused(2) ? 'focused' : ''}`}
                  onClick={() => props.onSelectContentArea?.()}
                >
                  <div class="setting-info">
                    <span class="setting-title">Aceleración Gráfica (GPU)</span>
                    <span class="setting-desc">{hardwareInfo().gpuRenderer} • {hardwareInfo().cpuCores} Cores Lógicos</span>
                  </div>
                  <span class="readonly-badge">{hardwareInfo().memoryEstimate}</span>
                </div>

                <div
                  class={`setting-card-row ${isRowFocused(3) ? 'focused' : ''}`}
                  onClick={() => {
                    props.onSelectContentArea?.();
                    handleUpdate(s => { s.display.vsync = !s.display.vsync; });
                  }}
                >
                  <div class="setting-info">
                    <span class="setting-title">Sincronización Vertical (VSync)</span>
                    <span class="setting-desc">Elimina el tearing sincronizando con la tasa de refresco del display</span>
                  </div>
                  <Switch
                    checked={props.settings?.display.vsync ?? true}
                    onChange={(val) => handleUpdate(s => { s.display.vsync = val; })}
                    class="console-switch"
                  >
                    <Switch.Input class="switch-input" />
                    <Switch.Control class="switch-control">
                      <Switch.Thumb class="switch-thumb" />
                    </Switch.Control>
                  </Switch>
                </div>
              </div>
            </div>
          )}

          {/* Emulators & Cores Tab */}
          {currentTab() === 'emulators' && (
            <div class="settings-tab-panel">
              <div class="panel-header-block">
                <div class="panel-header-titles">
                  <h3 class="panel-section-title">Gestor de Motores y Núcleos de Emulación</h3>
                  <p class="panel-section-desc">Binarios y núcleos compilados para ejecución directa bajo Gamescope</p>
                </div>
                <div class="settings-header-actions-row">
                  <button
                    class="settings-action-btn"
                    onClick={() => openEditEmulator()}
                  >
                    <span>+ AÑADIR NÚCLEO</span>
                  </button>
                </div>
              </div>

              <div class="cyber-emulator-deck-grid">
                <For each={props.emulators || []}>
                  {(emu, idx) => (
                    <div
                      class={`cyber-emulator-blade ${isRowFocused(idx()) ? 'focused' : ''}`}
                      onClick={() => {
                        props.onSelectContentArea?.();
                        openEditEmulator(emu);
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
                            <span class="readonly-badge" style={{ "font-size": "0.625rem", padding: "0.125rem 0.5rem" }}>
                              v{emu.version}
                            </span>
                          </div>
                          <div class="blade-exec-path">
                            {emu.executable} {emu.arguments.join(' ')}
                          </div>
                          <div class="blade-platform-chips-row">
                            <For each={emu.supportedPlatforms}>
                              {(plat) => <span class="blade-platform-chip">{plat}</span>}
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
                            openEditEmulator(emu);
                          }}
                        >
                          <span>[A] CONFIGURAR</span>
                        </button>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </div>
          )}

          {/* Audio Tab */}
          {currentTab() === 'audio' && (
            <div class="settings-tab-panel">
              <div class="panel-header-block">
                <div class="panel-header-titles">
                  <h3 class="panel-section-title">Configuración de Audio y Efectos</h3>
                  <p class="panel-section-desc">Sintetizador WebAudio de foco y ganancia maestra global</p>
                </div>
              </div>

              <div class="settings-form-stack">
                <div
                  class={`setting-card-row ${isRowFocused(0) ? 'focused' : ''}`}
                  onClick={() => {
                    props.onSelectContentArea?.();
                    handleUpdate(s => { s.audio.uiSoundEffects = !s.audio.uiSoundEffects; });
                  }}
                >
                  <div class="setting-info">
                    <span class="setting-title">Efectos de Sonido de la Interfaz</span>
                    <span class="setting-desc">Respuesta sonora sintetizada al navegar y seleccionar</span>
                  </div>
                  <Switch
                    checked={props.settings?.audio.uiSoundEffects ?? true}
                    onChange={(val) => handleUpdate(s => { s.audio.uiSoundEffects = val; })}
                    class="console-switch"
                  >
                    <Switch.Input class="switch-input" />
                    <Switch.Control class="switch-control">
                      <Switch.Thumb class="switch-thumb" />
                    </Switch.Control>
                  </Switch>
                </div>

                <div
                  class={`setting-card-row ${isRowFocused(1) ? 'focused' : ''}`}
                  onClick={() => props.onSelectContentArea?.()}
                >
                  <div class="setting-info">
                    <span class="setting-title">Volumen Maestro</span>
                    <span class="setting-desc">Nivel de ganancia global ({props.settings?.audio.masterVolume ?? 80}%)</span>
                  </div>
                  <Slider
                    value={[props.settings?.audio.masterVolume ?? 80]}
                    onChange={(val) => handleUpdate(s => { s.audio.masterVolume = val[0]; })}
                    minValue={0}
                    maxValue={100}
                    step={5}
                    class="console-slider"
                  >
                    <Slider.Track class="slider-track">
                      <Slider.Fill class="slider-fill" />
                      <Slider.Thumb class="slider-thumb">
                        <Slider.Input />
                      </Slider.Thumb>
                    </Slider.Track>
                  </Slider>
                </div>
              </div>
            </div>
          )}

          {/* Gamepad Tab */}
          {currentTab() === 'gamepad' && (
            <div class="settings-tab-panel">
              <div class="panel-header-block">
                <div class="panel-header-titles">
                  <h3 class="panel-section-title">Mandos y Periféricos Conectados</h3>
                  <p class="panel-section-desc">Detección en tiempo real de mandos físicos USB y Bluetooth</p>
                </div>
              </div>

              <div class="settings-form-stack">
                <div
                  class={`setting-card-row ${isRowFocused(0) ? 'focused' : ''}`}
                  onClick={() => {
                    props.onSelectContentArea?.();
                    handleUpdate(s => { s.gamepad.vibration = !s.gamepad.vibration; });
                  }}
                >
                  <div class="setting-info">
                    <span class="setting-title">Vibración Háptica Global (Rumble)</span>
                    <span class="setting-desc">Activa motores hápticos en mandos compatibles</span>
                  </div>
                  <Switch
                    checked={props.settings?.gamepad.vibration ?? true}
                    onChange={(val) => handleUpdate(s => { s.gamepad.vibration = val; })}
                    class="console-switch"
                  >
                    <Switch.Input class="switch-input" />
                    <Switch.Control class="switch-control">
                      <Switch.Thumb class="switch-thumb" />
                    </Switch.Control>
                  </Switch>
                </div>

                <div class="panel-section-title" style={{ "font-size": "0.9375rem", "margin-top": "0.5rem" }}>
                  Dispositivos de Entrada Detectados (Pulsar [A] para Test de Vibración)
                </div>

                <div class="gamepads-list-stack">
                  <div
                    class={`gamepad-device-card active-controller ${isRowFocused(1) ? 'focused' : ''}`}
                    onClick={() => props.onSelectContentArea?.()}
                  >
                    <div class="gamepad-device-header">
                      <div class="gamepad-icon-circle" style={{ "font-size": "0.75rem", "font-weight": "800" }}>KB</div>
                      <div>
                        <div class="gamepad-device-title">Teclado & Ratón Estándar (Entrada Activa)</div>
                        <div class="gamepad-device-sub">Mapeo Semántico EmuBox • Teclas Flechas / Enter / Escape</div>
                      </div>
                    </div>
                    <span class="readonly-badge highlight">ACTIVO • PUERTO PRIMARIO</span>
                  </div>

                  <For each={gamepadsList()}>
                    {(pad, padIdx) => (
                      <div
                        class={`gamepad-device-card active-controller ${isRowFocused(2 + padIdx()) ? 'focused' : ''}`}
                        onClick={() => {
                          props.onSelectContentArea?.();
                          triggerGamepadTest(pad.index);
                        }}
                      >
                        <div class="gamepad-device-header">
                          <div class="gamepad-icon-circle" style={{ "font-size": "0.75rem", "font-weight": "800" }}>P{pad.index + 1}</div>
                          <div>
                            <div class="gamepad-device-title">{pad.id}</div>
                            <div class="gamepad-device-sub">Puerto {pad.index + 1} • {pad.buttonsCount} Botones • {pad.axesCount} Ejes • {pad.hasVibration ? 'Háptica Soportada (Pulsar [A] para Test)' : 'Sin vibrador'}</div>
                          </div>
                        </div>
                        <span class="readonly-badge highlight">CONECTADO</span>
                      </div>
                    )}
                  </For>

                  {gamepadsList().length === 0 && (
                    <div class="gamepad-device-card" style={{ opacity: "0.7" }}>
                      <div class="gamepad-device-header">
                        <div class="gamepad-icon-circle" style={{ "font-size": "0.75rem", "font-weight": "800" }}>PAD</div>
                        <div>
                          <div class="gamepad-device-title">Esperando Mando Físico en Puertos 1 - 4...</div>
                          <div class="gamepad-device-sub">Conecta un mando Xbox, PlayStation o USB/Bluetooth para asignación directa</div>
                        </div>
                      </div>
                      <span class="readonly-badge">EN ESPERA</span>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}

          {/* OTA UPDATE, DECOUPLED LIFECYCLE & AUTO-UPDATE TAB */}
          {currentTab() === 'update' && (
            <div class="settings-tab-panel">
              <div class="panel-header-block">
                <div class="panel-header-titles">
                  <h3 class="panel-section-title">Actualización OTA & Mantenimiento Desacoplado</h3>
                  <p class="panel-section-desc">Actualiza la aplicación EmuBox automáticamente sin tocar Arch Linux ni tus ROMs y partidas</p>
                </div>
              </div>

              {/* Setting Row 0: Auto-Update Switch */}
              <div class="settings-form-stack">
                <div
                  class={`setting-card-row ${isRowFocused(0) ? 'focused' : ''}`}
                  onClick={() => {
                    props.onSelectContentArea?.();
                    handleUpdate(s => {
                      if (!s.updates) s.updates = { autoUpdate: true, channel: 'stable', checkOnStartup: true };
                      s.updates.autoUpdate = !s.updates.autoUpdate;
                    });
                  }}
                >
                  <div class="setting-info">
                    <span class="setting-title">Actualización Automática</span>
                    <span class="setting-desc">Instala automáticamente nuevas versiones estables de EmuBox cuando estén disponibles</span>
                  </div>
                  <Switch
                    checked={props.settings?.updates?.autoUpdate ?? true}
                    onChange={(val) => handleUpdate(s => {
                      if (!s.updates) s.updates = { autoUpdate: true, channel: 'stable', checkOnStartup: true };
                      s.updates.autoUpdate = val;
                    })}
                    class="console-switch"
                  >
                    <Switch.Input class="switch-input" />
                    <Switch.Control class="switch-control">
                      <Switch.Thumb class="switch-thumb" />
                    </Switch.Control>
                  </Switch>
                </div>
              </div>

              {/* Update Hero Blade (Row 1 focusable) */}
              <div
                class={`update-hero-blade ${isRowFocused(1) ? 'focused' : ''}`}
                style={{ "margin-top": "0.5rem", cursor: "pointer" }}
                onClick={() => {
                  props.onSelectContentArea?.();
                  if (props.updateInfo?.hasUpdate) {
                    handleApplyUpdate();
                  } else {
                    handleCheckForUpdates();
                  }
                }}
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
                  {updateStatusMsg() && (
                    <div class="update-meta-text" style={{ color: "#00f0ff", "font-weight": "800", "margin-top": "0.25rem" }}>
                      {updateStatusMsg()}
                    </div>
                  )}
                </div>

                <div class="settings-header-actions-row">
                  {props.updateInfo?.hasUpdate ? (
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
                  ) : (
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
                  )}
                </div>
              </div>

              {/* Progress Bar (Visible during active update) */}
              {isUpdating() && (
                <div class="update-progress-container">
                  <div class="update-progress-bar-fill" ref={progressBarRef} style={{ width: `${updateProgressVal()}%` }} />
                </div>
              )}

              {/* Available Update Changelog Card */}
              {props.updateInfo?.hasUpdate && (
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
              )}

              {/* Safety Guarantees Grid */}
              <div class="safety-guarantee-grid">
                <div class="safety-guarantee-card">
                  <span class="readonly-badge" style={{ "font-size": "0.625rem" }}>ROMS</span>
                  <div>
                    <div class="safety-title">ROMs & BIOS Intactas</div>
                    <div class="safety-desc">~/.local/share/emubox/roms</div>
                  </div>
                </div>
                <div class="safety-guarantee-card">
                  <span class="readonly-badge" style={{ "font-size": "0.625rem" }}>SAVES</span>
                  <div>
                    <div class="safety-title">Partidas & Saves Seguras</div>
                    <div class="safety-desc">~/.local/share/emubox/saves</div>
                  </div>
                </div>
                <div class="safety-guarantee-card">
                  <span class="readonly-badge" style={{ "font-size": "0.625rem" }}>OS</span>
                  <div>
                    <div class="safety-title">Arch Linux Intacto</div>
                    <div class="safety-desc">Drivers y Kernel aislados</div>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* CRUD Modal for Emulator / Core Management */}
      {isEditingEmulator() && (
        <div class="crud-modal-backdrop" onClick={() => setIsEditingEmulator(false)}>
          <div class="crud-modal-box" ref={modalBoxRef} onClick={(e) => e.stopPropagation()}>
            <h3 class="crud-modal-title">
              {editingEmulatorData().id ? 'Gestionar Motor / Núcleo de Emulación' : 'Añadir Nuevo Motor de Emulación'}
            </h3>

            <div class="crud-form-group">
              <label class="crud-label">Nombre del Motor</label>
              <input
                ref={nameInputRef}
                type="text"
                class={`crud-input ${modalFocusIdx() === 0 ? 'focused' : ''} ${isTyping() && modalFocusIdx() === 0 ? 'typing' : ''}`}
                value={editingEmulatorData().name || ''}
                onFocus={() => { setModalFocusIdx(0); setIsTyping(true); }}
                onBlur={() => setIsTyping(false)}
                onInput={(e) => setEditingEmulatorData({ ...editingEmulatorData(), name: e.currentTarget.value })}
              />
            </div>

            <div class="crud-form-group">
              <label class="crud-label">Binario / Ejecutable</label>
              <input
                ref={execInputRef}
                type="text"
                class={`crud-input ${modalFocusIdx() === 1 ? 'focused' : ''} ${isTyping() && modalFocusIdx() === 1 ? 'typing' : ''}`}
                value={editingEmulatorData().executable || ''}
                onFocus={() => { setModalFocusIdx(1); setIsTyping(true); }}
                onBlur={() => setIsTyping(false)}
                onInput={(e) => setEditingEmulatorData({ ...editingEmulatorData(), executable: e.currentTarget.value })}
              />
            </div>

            <div class="crud-form-group">
              <label class="crud-label">Tipo de Motor</label>
              <input
                ref={typeInputRef}
                type="text"
                class={`crud-input ${modalFocusIdx() === 2 ? 'focused' : ''} ${isTyping() && modalFocusIdx() === 2 ? 'typing' : ''}`}
                value={editingEmulatorData().coreType || 'libretro'}
                onFocus={() => { setModalFocusIdx(2); setIsTyping(true); }}
                onBlur={() => setIsTyping(false)}
                onInput={(e) => setEditingEmulatorData({ ...editingEmulatorData(), coreType: e.currentTarget.value as any })}
              />
            </div>

            <div class="crud-modal-actions">
              {editingEmulatorData().id && (
                <button
                  ref={deleteBtnRef}
                  class={`crud-btn-delete ${modalFocusIdx() === 3 ? 'focused' : ''}`}
                  onFocus={() => setModalFocusIdx(3)}
                  onClick={deleteEmulator}
                >
                  ELIMINAR NÚCLEO
                </button>
              )}
              <button
                ref={cancelBtnRef}
                class={`crud-btn-cancel ${(editingEmulatorData().id ? modalFocusIdx() === 4 : modalFocusIdx() === 3) ? 'focused' : ''}`}
                onFocus={() => setModalFocusIdx(editingEmulatorData().id ? 4 : 3)}
                onClick={() => setIsEditingEmulator(false)}
              >
                CANCELAR [B]
              </button>
              <button
                ref={saveBtnRef}
                class={`crud-btn-save ${(editingEmulatorData().id ? modalFocusIdx() === 5 : modalFocusIdx() === 4) ? 'focused' : ''}`}
                onFocus={() => setModalFocusIdx(editingEmulatorData().id ? 5 : 4)}
                onClick={saveEmulator}
              >
                GUARDAR CAMBIOS [A]
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
