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

# 4. Si se ejecuta desde una consola TTY sin servidor grafico, iniciar sesion con Cage o Gamescope
if command -v cage >/dev/null 2>&1; then
  echo "[EmuBox] Iniciando sesión gráfica con Cage (Wayland)..."
  exec cage -- "${EMUBOX_BIN}" "$@"
elif command -v gamescope >/dev/null 2>&1; then
  echo "[EmuBox] Iniciando sesión gráfica con Gamescope..."
  exec gamescope -f -W 1920 -H 1080 -- "${EMUBOX_BIN}" "$@"
elif command -v xinit >/dev/null 2>&1; then
  echo "[EmuBox] Iniciando sesión gráfica con X11..."
  exec xinit "${EMUBOX_BIN}" "$@" -- :0
else
  echo "[ERROR] No se detecto ninguna sesion grafica (\$WAYLAND_DISPLAY / \$DISPLAY)." >&2
  echo "Para arrancar EmuBox directamente desde TTY, instala cage:" >&2
  echo "  sudo pacman -S --needed cage" >&2
  exit 1
fi
