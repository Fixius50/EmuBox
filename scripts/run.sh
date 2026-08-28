#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX - SCRIPT MAESTRO DE ARRANQUE Y EJECUCION DE LA CONSOLA
# ==============================================================================
#
# Este script:
#   1. Exporta las variables graficas criticas (WEBKIT_DISABLE_DMABUF_RENDERER=1).
#   2. Localiza el binario nativo de EmuBox (/opt/emubox/bin/emubox o release local).
#   3. Detecta el entorno (Wayland / X11 / Consola TTY).
#   4. Si esta en TTY, levanta automaticamente el compositor Cage / Gamescope.
#   5. Lanza la interfaz de EmuBox.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# 1. Variables de entorno indispensables para WebKitGTK / Wayland
export WEBKIT_DISABLE_DMABUF_RENDERER=1
export GDK_BACKEND="${GDK_BACKEND:-wayland,x11}"

# Asegurar XDG_RUNTIME_DIR valido para compositores Wayland (Cage / Gamescope)
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
    echo "[ERROR] No se encontro el binario compilado de EmuBox." >&2
    echo "Por favor, compila primero con: bash scripts/build.sh" >&2
    exit 1
  fi
fi

# 3. Si ya existe un servidor grafico activo (Wayland o X11), ejecutar directamente
if [[ -n "${WAYLAND_DISPLAY:-}" || -n "${DISPLAY:-}" ]]; then
  exec "${EMUBOX_BIN}" "$@"
fi

# 4. Si se ejecuta desde una consola TTY sin servidor gráfico, iniciar sesión Wayland con Cage + Gamescope
DBUS_RUN=""
if command -v dbus-run-session >/dev/null 2>&1 && [[ -z "${DBUS_SESSION_BUS_ADDRESS:-}" ]]; then
  DBUS_RUN="dbus-run-session"
fi

if command -v cage >/dev/null 2>&1 && command -v gamescope >/dev/null 2>&1; then
  echo "[EmuBox] Iniciando sesión gráfica de consola con Cage + Gamescope (Wayland)..."
  if [[ -n "${DBUS_RUN}" ]]; then
    exec dbus-run-session cage -- gamescope -f -W 1920 -H 1080 -r 60 -- "${EMUBOX_BIN}" "$@"
  else
    exec cage -- gamescope -f -W 1920 -H 1080 -r 60 -- "${EMUBOX_BIN}" "$@"
  fi
elif command -v cage >/dev/null 2>&1; then
  echo "[EmuBox] Iniciando sesión gráfica de kiosko con Cage (Wayland)..."
  if [[ -n "${DBUS_RUN}" ]]; then
    exec dbus-run-session cage -- "${EMUBOX_BIN}" "$@"
  else
    exec cage -- "${EMUBOX_BIN}" "$@"
  fi
elif command -v gamescope >/dev/null 2>&1; then
  echo "[EmuBox] Iniciando sesión gráfica directa con Gamescope (Wayland Direct DRM)..."
  if [[ -n "${DBUS_RUN}" ]]; then
    exec dbus-run-session gamescope -f -W 1920 -H 1080 -r 60 -- "${EMUBOX_BIN}" "$@"
  else
    exec gamescope -f -W 1920 -H 1080 -r 60 -- "${EMUBOX_BIN}" "$@"
  fi
else
  echo "[ERROR] No se detectó ninguna sesión gráfica (\$WAYLAND_DISPLAY / \$DISPLAY)." >&2
  echo "Para arrancar EmuBox en consola dedicada, instala cage y gamescope:" >&2
  echo "  sudo pacman -S --needed cage gamescope" >&2
  exit 1
fi

