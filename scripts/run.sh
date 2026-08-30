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

# 1. Variables de entorno indispensables para WebKitGTK / Wayland
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export GDK_BACKEND="${GDK_BACKEND:-wayland}"

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
EMUBOX_BIN="/opt/emubox/bin/emubox"
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
if [[ -n "${WAYLAND_DISPLAY:-}" || -n "${DISPLAY:-}" ]]; then
  exec "${EMUBOX_BIN}" "$@"
fi

# 4. Detección de Capacidades Gráficas para Consola TTY
HAS_VULKAN=0
if command -v vulkaninfo >/dev/null 2>&1; then
  if vulkaninfo --summary 2>&1 | grep -qE 'deviceName|GPU0|GPU id'; then
    HAS_VULKAN=1
  fi
fi

# 5. Iniciar sincronizador reactivo de resolución DRM en segundo plano si existe cage
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

# MODO NATIVO: GPU física con soporte Vulkan real + Gamescope
if [[ ${HAS_VULKAN} -eq 1 ]] && command -v gamescope >/dev/null 2>&1; then
  if command -v cage >/dev/null 2>&1; then
    echo "[EmuBox] Iniciando en Modo NATIVO (Cage -> Gamescope -> EmuBox)..."
    if [[ -n "${DBUS_RUN}" ]]; then
      dbus-run-session cage -- gamescope -f -- "${EMUBOX_BIN}" "$@"
    else
      cage -- gamescope -f -- "${EMUBOX_BIN}" "$@"
    fi
  else
    echo "[EmuBox] Iniciando en Modo NATIVO DIRECTO (Gamescope -> EmuBox)..."
    if [[ -n "${DBUS_RUN}" ]]; then
      dbus-run-session gamescope -f -- "${EMUBOX_BIN}" "$@"
    else
      gamescope -f -- "${EMUBOX_BIN}" "$@"
    fi
  fi

# MODO COMPATIBILIDAD (VMware SVGA / VirtualBox / GPU sin Vulkan): Cage Wayland Kiosk
elif command -v cage >/dev/null 2>&1; then
  echo "[EmuBox] Iniciando en Modo COMPATIBILIDAD (Cage Wayland Kiosk -> EmuBox)..."
  if [[ -n "${DBUS_RUN}" ]]; then
    dbus-run-session cage -- "${EMUBOX_BIN}" "$@"
  else
    cage -- "${EMUBOX_BIN}" "$@"
  fi

# FALLBACK DIRECTO
else
  echo "[ERROR] No se detectó ninguna sesión gráfica (\$WAYLAND_DISPLAY / \$DISPLAY)." >&2
  echo "Para arrancar EmuBox en consola dedicada, instala cage:" >&2
  echo "  sudo pacman -S --needed cage" >&2
  exit 1
fi
