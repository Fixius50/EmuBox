#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX SETUP - FIRST-RUN DETECTOR & VERSIONED CONFIG GENERATOR
#  Auto-detects GPU, displays, controllers and generates /etc/emubox/config.json
# ==============================================================================

set -euo pipefail

EMUBOX_CONFIG_DIR="/etc/emubox"
EMUBOX_DATA_DIR="/var/lib/emubox"
CONFIG_JSON="${EMUBOX_CONFIG_DIR}/config.json"

echo "  -> Ejecutando detección de primer arranque (First Run)..."

# 1. Detect GPU Vendor
GPU_VENDOR="generic"
GPU_INFO="$(lspci -nnk 2>/dev/null | grep -A3 -Ei 'VGA compatible controller|3D controller|Display controller' || true)"
if echo "${GPU_INFO}" | grep -Eiq 'VMware|vmwgfx|VirtualBox|vboxvideo|virtio|qxl'; then
  GPU_VENDOR="generic"
elif echo "${GPU_INFO}" | grep -Eiq 'NVIDIA|nvidia|nouveau'; then
  GPU_VENDOR="nvidia"
elif echo "${GPU_INFO}" | grep -Eiq 'Intel|i915|xe'; then
  GPU_VENDOR="intel"
elif echo "${GPU_INFO}" | grep -Eiq 'AMD|ATI|Radeon|amdgpu'; then
  GPU_VENDOR="amd"
fi
echo "  ✓ GPU detectada: ${GPU_VENDOR}"

# 2. Generate Master config.json if not already present
if [[ ! -f "${CONFIG_JSON}" ]]; then
  echo "  -> Generando archivo de configuración inicial versionado..."
  cat > "${CONFIG_JSON}" << EOF
{
  "version": 1,
  "paths": {
    "roms": "${EMUBOX_DATA_DIR}/games",
    "saves": "${EMUBOX_DATA_DIR}/saves",
    "states": "${EMUBOX_DATA_DIR}/states",
    "screenshots": "${EMUBOX_DATA_DIR}/screenshots",
    "covers": "/var/cache/emubox/covers",
    "logs": "/var/log/emubox"
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
