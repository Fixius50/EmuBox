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

# 4. Detección de GPU, Driver y Vendor mediante PCI/DRM
GPU_INFO="$(lspci -nnk 2>/dev/null | grep -A3 -Ei 'VGA compatible controller|3D controller|Display controller' | head -n 4 || true)"
GPU_DEVICE="$(echo "$GPU_INFO" | grep -Ei 'VGA|3D|Display' | sed -E 's/^[^:]+: //; s/ \(rev .*\)//' | head -n 1)"
[[ -z "$GPU_DEVICE" ]] && GPU_DEVICE="Dispositivo Gráfico Genérico / Desconocido"

GPU_DRIVER="$(echo "$GPU_INFO" | grep -Ei 'Kernel driver in use:' | awk '{print $NF}' | head -n 1)"
[[ -z "$GPU_DRIVER" ]] && GPU_DRIVER="desconocido"

GPU_VENDOR="Desconocido"
if echo "$GPU_DEVICE $GPU_DRIVER" | grep -Eiq 'AMD|ATI|Radeon|amdgpu'; then
  GPU_VENDOR="AMD"
elif echo "$GPU_DEVICE $GPU_DRIVER" | grep -Eiq 'Intel|i915|xe'; then
  GPU_VENDOR="Intel"
elif echo "$GPU_DEVICE $GPU_DRIVER" | grep -Eiq 'NVIDIA|nvidia|nouveau'; then
  GPU_VENDOR="NVIDIA"
elif echo "$GPU_DEVICE $GPU_DRIVER" | grep -Eiq 'VMware|vmwgfx|VirtualBox|vboxvideo|virtio|qxl'; then
  GPU_VENDOR="Virtual / Emulada"
fi

# 5. Detección de Renderer y Aceleración Real por Hardware (Descartando llvmpipe/software)
RENDERER_DESC="Software / Genérico"
HAS_GPU_ACCEL=0
HAS_HW_VULKAN=0

if command -v vulkaninfo >/dev/null 2>&1; then
  VK_SUMMARY="$(vulkaninfo --summary 2>&1 || true)"
  VK_DEVICE="$(echo "$VK_SUMMARY" | grep -Ei 'deviceName' | head -n 1 | sed -E 's/.*= //; s/^[ \t]*//' || true)"
  
  if [[ -n "$VK_DEVICE" ]]; then
    if echo "$VK_DEVICE" | grep -Eiq 'llvmpipe|softpipe|swrast|Software'; then
      RENDERER_DESC="llvmpipe (CPU Software Rasterizer)"
      HAS_GPU_ACCEL=0
      HAS_HW_VULKAN=0
    else
      RENDERER_DESC="$VK_DEVICE"
      HAS_GPU_ACCEL=1
      HAS_HW_VULKAN=1
    fi
  fi
fi

if [[ $HAS_GPU_ACCEL -eq 0 ]] && command -v glxinfo >/dev/null 2>&1; then
  GL_RENDERER="$(glxinfo -B 2>/dev/null | grep -Ei 'OpenGL renderer string:' | sed -E 's/.*: //; s/^[ \t]*//' || true)"
  if [[ -n "$GL_RENDERER" ]]; then
    if echo "$GL_RENDERER" | grep -Eiq 'llvmpipe|softpipe|swrast'; then
      RENDERER_DESC="llvmpipe (CPU Software Rasterizer)"
      HAS_GPU_ACCEL=0
    elif [[ "$GPU_VENDOR" =~ ^(AMD|Intel|NVIDIA)$ ]]; then
      RENDERER_DESC="$GL_RENDERER"
      HAS_GPU_ACCEL=1
    fi
  fi
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

# 1. PIPELINE PRINCIPAL: GPU Acelerada -> Gamescope directo
if [[ $HAS_GPU_ACCEL -eq 1 ]] && command -v gamescope >/dev/null 2>&1; then
  echo "[EmuBox] Iniciando en Modo ACELERADO (Gamescope -> EmuBox)..."
  if [[ -n "${DBUS_RUN}" ]]; then
    exec dbus-run-session gamescope -f -- "${EMUBOX_BIN}" "$@"
  else
    exec gamescope -f -- "${EMUBOX_BIN}" "$@"
  fi

# 2. PIPELINE DE EMERGENCIA: CPU Software (llvmpipe / VM sin aceleración) -> Cage Kiosk
elif command -v cage >/dev/null 2>&1; then
  echo "[EmuBox] Iniciando en Modo EMERGENCIA CPU/SOFTWARE (Cage Kiosk -> EmuBox)..."
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
