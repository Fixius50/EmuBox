#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX SETUP - FIRST-RUN DETECTOR & VERSIONED CONFIG GENERATOR
#  Auto-detects GPU, displays, controllers and generates ~/.config/emubox/config.json
# ==============================================================================

set -euo pipefail

XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
EMUBOX_CONFIG_DIR="${XDG_CONFIG_HOME}/emubox"
CONFIG_JSON="${EMUBOX_CONFIG_DIR}/config.json"

echo "  -> Ejecutando detección de primer arranque (First Run)..."

# 1. Detect GPU Vendor
GPU_VENDOR="generic"
if lspci 2>/dev/null | grep -i 'vga\|3d\|display' | grep -iq 'amd\|radeon'; then
  GPU_VENDOR="amd"
elif lspci 2>/dev/null | grep -i 'vga\|3d\|display' | grep -iq 'nvidia'; then
  GPU_VENDOR="nvidia"
elif lspci 2>/dev/null | grep -i 'vga\|3d\|display' | grep -iq 'intel'; then
  GPU_VENDOR="intel"
fi
echo "  ✓ GPU detectada: ${GPU_VENDOR}"

# 2. Generate Master config.json if not already present
if [[ ! -f "${CONFIG_JSON}" ]]; then
  echo "  -> Generando archivo de configuración inicial versionado..."
  cat > "${CONFIG_JSON}" << EOF
{
  "version": 1,
  "paths": {
    "roms": "${XDG_DATA_HOME}/emubox/roms",
    "saves": "${XDG_DATA_HOME}/emubox/saves",
    "states": "${XDG_DATA_HOME}/emubox/states",
    "screenshots": "${XDG_DATA_HOME}/emubox/screenshots",
    "covers": "${XDG_DATA_HOME}/emubox/covers",
    "logs": "${XDG_DATA_HOME}/emubox/logs"
  },
  "display": {
    "resolution": "1920x1080",
    "refreshRate": 60,
    "fullscreen": true,
    "vsync": true,
    "gamescopeEnabled": true,
    "gamescopeScaling": "integer",
    "crtShader": "none"
  },
  "audio": {
    "volume": 85,
    "uiSoundEffects": true,
    "backgroundMusic": false,
    "latencyMs": 16
  },
  "input": {
    "deadzone": 0.15,
    "vibrationEnabled": true,
    "swapSouthEastButtons": false,
    "pollRateHz": 250
  },
  "emulators": {
    "defaultMapping": {
      "snes": "snes9x",
      "ps1": "duckstation",
      "ps2": "pcsx2",
      "n64": "mupen64plus",
      "genesis": "genesis_plus_gx",
      "gba": "mgba",
      "dreamcast": "flycast",
      "arcade": "fbneo"
    }
  },
  "interface": {
    "locale": "es",
    "theme": "dark-cyber",
    "animations": true,
    "showFpsOverlay": false,
    "performanceMode": "high-performance"
  }
}
EOF
  echo "  ✓ Configuración creada en ${CONFIG_JSON}"
else
  echo "  ✓ Configuración existente respetada en ${CONFIG_JSON}"
fi
