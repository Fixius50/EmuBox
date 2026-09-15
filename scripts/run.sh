#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - SCRIPT MAESTRO DE ARRANQUE Y EJECUCIÓN (MODO ADAPTATIVO)
# ==============================================================================
#
# Este script:
#   1. Configura Wayland y ajustes de compatibilidad especificos del driver.
#   2. Localiza el binario nativo de EmuBox (/opt/emubox/bin/emubox o release local).
#   3. Si está en TTY:
#      - Sondea aceleracion Vulkan y OpenGL, independiente de CPU y virtualizacion.
#      - Detecta resolución nativa DRM de los conectores (/sys/class/drm/*/modes).
#      - Selecciona backend acelerado y luego un compositor compatible.
#      - Sondeo inconcluso conserva auto; software confirmado o fallback explicito.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
source "$ROOT_DIR/installer/lib/architecture.sh"
source "$ROOT_DIR/installer/lib/graphics.sh"

# 1. Variables de entorno indispensables para WebKitGTK / Wayland
export GDK_BACKEND="${GDK_BACKEND:-wayland}"
export XCURSOR_THEME="${XCURSOR_THEME:-Adwaita}"
export XCURSOR_SIZE="${XCURSOR_SIZE:-32}"
configure_emubox_host_logging "$(systemd-detect-virt --vm 2>/dev/null || true)"

# Asegurar XDG_RUNTIME_DIR válido para compositores Wayland (Cage / Gamescope)
if [[ -z "${XDG_RUNTIME_DIR:-}" ]]; then
  CURRENT_UID="$(id -u)"
  if [[ -d "/run/user/${CURRENT_UID}" ]]; then
    export XDG_RUNTIME_DIR="/run/user/${CURRENT_UID}"
  else
    export XDG_RUNTIME_DIR="/tmp/run-user-${CURRENT_UID}"
    mkdir -p -m 0700 "${XDG_RUNTIME_DIR}" 2>/dev/null || true
  fi
fi
export XDG_SESSION_TYPE="${XDG_SESSION_TYPE:-wayland}"

if [[ -z "${DBUS_SESSION_BUS_ADDRESS:-}" && -S "${XDG_RUNTIME_DIR}/bus" ]]; then
  export DBUS_SESSION_BUS_ADDRESS="unix:path=${XDG_RUNTIME_DIR}/bus"
fi

# 2. Localizar binario ejecutable
EMUBOX_BIN="$ROOT_DIR/bin/emubox"
if [[ ! -x "${EMUBOX_BIN}" ]]; then
  if [[ -x "${ROOT_DIR}/src-tauri/target/release/emubox" ]]; then
    EMUBOX_BIN="${ROOT_DIR}/src-tauri/target/release/emubox"
  elif [[ -x "${ROOT_DIR}/bin/emubox" ]]; then
    EMUBOX_BIN="${ROOT_DIR}/bin/emubox"
  else
    echo "[ERROR] No se encontró el binario compilado de EmuBox." >&2
    echo "Por favor, compila primero con: bash scripts/build.sh" >&2
    exit 1
  fi
fi

# 3. Si ya existe un servidor gráfico activo (Wayland o X11), ejecutar directamente
validate_emubox_binary "$EMUBOX_BIN"
RENDER_MODE="${EMUBOX_RENDER_MODE:-}"
if [[ -z "$RENDER_MODE" && -f /etc/emubox/graphics-mode ]]; then
  RENDER_MODE=$(< /etc/emubox/graphics-mode)
fi
RENDER_MODE="${RENDER_MODE:-auto}"
export EMUBOX_RENDER_MODE="$RENDER_MODE"
configure_emubox_render_mode auto
detect_emubox_graphics
if [[ "$GPU_DRIVER" == vmwgfx ]]; then
  export WEBKIT_DISABLE_DMABUF_RENDERER="${WEBKIT_DISABLE_DMABUF_RENDERER:-1}"
fi
REQUESTED_RENDER_MODE="$RENDER_MODE"
RENDER_MODE=$(select_emubox_render_mode "$RENDER_MODE" "$HAS_HW_VULKAN" "$HAS_HW_OPENGL" "$GRAPHICS_DETECTION_STATE")
configure_emubox_render_mode "$RENDER_MODE"
export EMUBOX_GRAPHICS_BACKEND="$GRAPHICS_BACKEND"
export EMUBOX_OPERATIONAL_BACKEND="$GRAPHICS_OPERATIONAL_BACKEND"
echo "[EmuBox] detection=$GRAPHICS_DETECTION_STATE probes=$GRAPHICS_PROBE_REASONS selectedDevice=$GPU_DEVICE activeDevice=$GPU_ACTIVE_DEVICE operationalBackend=$GRAPHICS_OPERATIONAL_BACKEND fallback=$GRAPHICS_FALLBACK_REASON"
CPU_MODEL=$(awk -F ': ' '/model name|Hardware|Model/ {print $2; exit}' /proc/cpuinfo)
echo "[EmuBox] cpu=${CPU_MODEL:-$(uname -m)} cores=$(getconf _NPROCESSORS_ONLN)"
echo "[EmuBox] requestedRenderMode=$REQUESTED_RENDER_MODE renderMode=$RENDER_MODE WLR_RENDERER=${WLR_RENDERER:-auto} LIBGL_ALWAYS_SOFTWARE=${LIBGL_ALWAYS_SOFTWARE:-0}"
echo "[EmuBox] architecture=$(get_emubox_architecture) gpu=$GPU_VENDOR gpuKind=$GPU_KIND vm=$IS_VIRTUAL_MACHINE accelerated=$HAS_ACCELERATION backend=$GRAPHICS_BACKEND renderer=$RENDERER_DESC drm=$HAS_DRM vulkan=$HAS_HW_VULKAN opengl=$HAS_OPENGL openglAccelerated=$HAS_HW_OPENGL gamescopeReady=$GAMESCOPE_READY compositor=$EMUBOX_COMPOSITOR device=$DEVICE_MODEL"
if [[ -n "${WAYLAND_DISPLAY:-}" || -n "${DISPLAY:-}" ]]; then
  exec "${EMUBOX_BIN}" "$@"
fi

# Iniciar sincronizador reactivo de resolución DRM en segundo plano si existe cage
SYNC_PID=""
if [[ -f "${SCRIPT_DIR}/emubox-drm-sync.sh" && -x "$(command -v cage 2>/dev/null || true)" ]]; then
  bash "${SCRIPT_DIR}/emubox-drm-sync.sh" >/dev/null 2>&1 &
  SYNC_PID=$!
  trap '[[ -n "${SYNC_PID:-}" ]] && kill -TERM "$SYNC_PID" 2>/dev/null || true' EXIT INT TERM
fi

DBUS_RUN=""
if command -v dbus-run-session >/dev/null 2>&1 && [[ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]]; then
  DBUS_RUN="dbus-run-session"
fi

# 1. Gamescope solo cuando seleccionado y compatible
if [[ "$EMUBOX_COMPOSITOR" == gamescope ]]; then
  echo "[EmuBox] Iniciando en Modo ACELERADO (Gamescope -> EmuBox)..."
  if [[ -n "${DBUS_RUN}" ]]; then
    exec dbus-run-session gamescope -f -- "${EMUBOX_BIN}" "$@"
  else
    exec gamescope -f -- "${EMUBOX_BIN}" "$@"
  fi

# 2. Cage: OpenGL acelerado disponible o CPU cuando no se detecta aceleracion
elif [[ "$EMUBOX_COMPOSITOR" == cage ]]; then
  if [[ "$GRAPHICS_OPERATIONAL_BACKEND" == opengl ]]; then export WLR_RENDERER=gles2; fi
  configure_emubox_cursor "$GPU_DRIVER"
  configure_emubox_presentation "$GPU_DRIVER"
  configure_emubox_cage_command "$GPU_DRIVER" "$ROOT_DIR" "$(get_emubox_architecture)"
  echo "[EmuBox] WLR_NO_HARDWARE_CURSORS=${WLR_NO_HARDWARE_CURSORS:-0} WLR_DRM_NO_ATOMIC=${WLR_DRM_NO_ATOMIC:-0} driver=$GPU_DRIVER"
  echo "[EmuBox] vmwgfxCompat=$([[ ${#CAGE_COMMAND[@]} -gt 1 ]] && echo 1 || echo 0) SVGA_NO_LOGGING=${SVGA_NO_LOGGING:-0}"
  echo "[EmuBox] presentation: renderer=${WLR_RENDERER:-auto} damage=${WLR_SCENE_DEBUG_DAMAGE:-partial} disableDirectScanout=${WLR_SCENE_DISABLE_DIRECT_SCANOUT:-0} webkitDmabufDisabled=${WEBKIT_DISABLE_DMABUF_RENDERER:-0}"
  echo "[EmuBox] Iniciando con Cage (backend=$GRAPHICS_BACKEND)..."
  if [[ -n "${DBUS_RUN}" ]]; then
    exec dbus-run-session "${CAGE_COMMAND[@]}" -- "${EMUBOX_BIN}" "$@"
  else
    exec "${CAGE_COMMAND[@]}" -- "${EMUBOX_BIN}" "$@"
  fi

# 3. FALLBACK DIRECTO
else
  echo "[ERROR] No hay compositor compatible disponible para backend=$GRAPHICS_BACKEND DRM=$HAS_DRM." >&2
  echo "Para arrancar EmuBox en consola dedicada, instala cage:" >&2
  echo "  sudo pacman -S --needed cage" >&2
  exit 1
fi
