#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - SCRIPT MAESTRO DE ARRANQUE Y EJECUCIÓN (MODO ADAPTATIVO)
# ==============================================================================
#
# Este script:
#   1. Exporta las variables gráficas críticas (WEBKIT_DISABLE_DMABUF_RENDERER=1).
#   2. Localiza el binario nativo de EmuBox (/opt/emubox/bin/emubox o release local).
#   3. Si está en TTY:
#      - Sondea si existe aceleración Vulkan real.
#      - Detecta resolución nativa DRM de los conectores (/sys/class/drm/*/modes).
#      - Si Vulkan está activo -> ejecuta Gamescope con resolución detectada.
#      - Si Vulkan no está disponible (VM / driver software) -> ejecuta Cage.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
source "$ROOT_DIR/installer/lib/architecture.sh"
source "$ROOT_DIR/installer/lib/graphics.sh"

# 1. Variables de entorno indispensables para WebKitGTK / Wayland
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export GDK_BACKEND="${GDK_BACKEND:-wayland}"
export XCURSOR_THEME="${XCURSOR_THEME:-Adwaita}"
export XCURSOR_SIZE="${XCURSOR_SIZE:-32}"

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
if [[ -n "${WAYLAND_DISPLAY:-}" || -n "${DISPLAY:-}" ]]; then
  configure_emubox_render_mode "$RENDER_MODE"
  exec "${EMUBOX_BIN}" "$@"
fi

detect_emubox_graphics
REQUESTED_RENDER_MODE="$RENDER_MODE"
RENDER_MODE=$(select_emubox_render_mode "$RENDER_MODE" "$HAS_HW_VULKAN" "$HAS_HW_OPENGL")
configure_emubox_render_mode "$RENDER_MODE"
if [[ "$RENDER_MODE" == software ]]; then
  EMUBOX_COMPOSITOR=cage
fi
CPU_MODEL=$(awk -F ': ' '/model name|Hardware|Model/ {print $2; exit}' /proc/cpuinfo)
echo "[EmuBox] cpu=${CPU_MODEL:-$(uname -m)} cores=$(getconf _NPROCESSORS_ONLN)"
echo "[EmuBox] requestedRenderMode=$REQUESTED_RENDER_MODE renderMode=$RENDER_MODE WLR_RENDERER=${WLR_RENDERER:-auto} LIBGL_ALWAYS_SOFTWARE=${LIBGL_ALWAYS_SOFTWARE:-0}"
echo "[EmuBox] architecture=$(get_emubox_architecture) gpu=$GPU_VENDOR renderer=$RENDERER_DESC drm=$HAS_DRM vulkan=$HAS_HW_VULKAN opengl=$HAS_OPENGL openglAccelerated=$HAS_HW_OPENGL gamescope=$HAS_GAMESCOPE compositor=$EMUBOX_COMPOSITOR device=$DEVICE_MODEL"

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

# 1. PIPELINE PRINCIPAL: GPU Acelerada -> Gamescope directo
if [[ "$EMUBOX_COMPOSITOR" == gamescope ]]; then
  echo "[EmuBox] Iniciando en Modo ACELERADO (Gamescope -> EmuBox)..."
  if [[ -n "${DBUS_RUN}" ]]; then
    exec dbus-run-session gamescope -f -- "${EMUBOX_BIN}" "$@"
  else
    exec gamescope -f -- "${EMUBOX_BIN}" "$@"
  fi

# 2. Cage: OpenGL acelerado disponible o CPU cuando no se detecta aceleracion
elif command -v cage >/dev/null 2>&1; then
  configure_emubox_cursor "$GPU_DRIVER"
  echo "[EmuBox] WLR_NO_HARDWARE_CURSORS=${WLR_NO_HARDWARE_CURSORS:-0} WLR_DRM_NO_ATOMIC=${WLR_DRM_NO_ATOMIC:-0} driver=$GPU_DRIVER"
  echo "[EmuBox] Iniciando con Cage (sin Gamescope/Vulkan disponible)..."
  if [[ -n "${DBUS_RUN}" ]]; then
    exec dbus-run-session cage -- "${EMUBOX_BIN}" "$@"
  else
    exec cage -- "${EMUBOX_BIN}" "$@"
  fi

# 3. FALLBACK DIRECTO
else
  echo "[ERROR] No se detectó ninguna sesión gráfica (\$WAYLAND_DISPLAY / \$DISPLAY)." >&2
  echo "Para arrancar EmuBox en consola dedicada, instala cage:" >&2
  echo "  sudo pacman -S --needed cage" >&2
  exit 1
fi
