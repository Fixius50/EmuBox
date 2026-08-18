#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX SETUP - EMULATOR BINARY DISCOVERY & REGISTRATION
#  Discovers installed emulators and generates ~/.config/emubox/emulators.json
# ==============================================================================

set -euo pipefail

XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
EMUBOX_CONFIG_DIR="${XDG_CONFIG_HOME}/emubox"
EMULATORS_JSON="${EMUBOX_CONFIG_DIR}/emulators.json"

echo "  -> Detectando ejecutables de emuladores en el sistema..."

check_binary() {
  local bin="$1"
  command -v "$bin" >/dev/null 2>&1
}

# Determine status for each standard engine
RETROARCH_STATUS="inactive"
if check_binary "retroarch"; then
  RETROARCH_STATUS="active"
fi

DUCKSTATION_STATUS="inactive"
if check_binary "duckstation-qt" || check_binary "duckstation-nogui"; then
  DUCKSTATION_STATUS="active"
fi

PCSX2_STATUS="inactive"
if check_binary "pcsx2-qt" || check_binary "pcsx2"; then
  PCSX2_STATUS="active"
fi

MGBA_STATUS="inactive"
if check_binary "mgba-qt" || check_binary "mgba"; then
  MGBA_STATUS="active"
fi

FLYCAST_STATUS="inactive"
if check_binary "flycast"; then
  FLYCAST_STATUS="active"
fi

cat > "${EMULATORS_JSON}" << EOF
[
  {
    "id": "snes9x",
    "name": "Snes9x (Libretro Core)",
    "version": "1.62.3",
    "supportedPlatforms": ["snes"],
    "coreType": "libretro",
    "status": "${RETROARCH_STATUS}",
    "executable": "retroarch",
    "arguments": ["-L", "snes9x_libretro.so"]
  },
  {
    "id": "duckstation",
    "name": "DuckStation (Vulkan Direct)",
    "version": "0.1-6824",
    "supportedPlatforms": ["ps1"],
    "coreType": "standalone",
    "status": "${DUCKSTATION_STATUS}",
    "executable": "duckstation-qt",
    "arguments": ["-batch", "-fullscreen"]
  },
  {
    "id": "pcsx2",
    "name": "PCSX2 (Vulkan Standalone)",
    "version": "2.0.2",
    "supportedPlatforms": ["ps2"],
    "coreType": "standalone",
    "status": "${PCSX2_STATUS}",
    "executable": "pcsx2-qt",
    "arguments": ["-fullscreen", "-batch"]
  },
  {
    "id": "mupen64plus",
    "name": "Mupen64Plus-Next (GLideN64)",
    "version": "2.5.9",
    "supportedPlatforms": ["n64"],
    "coreType": "libretro",
    "status": "${RETROARCH_STATUS}",
    "executable": "retroarch",
    "arguments": ["-L", "mupen64plus_next_libretro.so"]
  },
  {
    "id": "genesis_plus_gx",
    "name": "Genesis Plus GX (Libretro Core)",
    "version": "1.7.4",
    "supportedPlatforms": ["genesis"],
    "coreType": "libretro",
    "status": "${RETROARCH_STATUS}",
    "executable": "retroarch",
    "arguments": ["-L", "genesis_plus_gx_libretro.so"]
  },
  {
    "id": "mgba",
    "name": "mGBA (Vulkan Standalone)",
    "version": "0.10.3",
    "supportedPlatforms": ["gba"],
    "coreType": "standalone",
    "status": "${MGBA_STATUS}",
    "executable": "mgba-qt",
    "arguments": ["-f"]
  },
  {
    "id": "flycast",
    "name": "Flycast (Vulkan Direct)",
    "version": "2.4",
    "supportedPlatforms": ["dreamcast"],
    "coreType": "standalone",
    "status": "${FLYCAST_STATUS}",
    "executable": "flycast",
    "arguments": []
  },
  {
    "id": "fbneo",
    "name": "FinalBurn Neo (Libretro Core)",
    "version": "1.0.0.3",
    "supportedPlatforms": ["arcade"],
    "coreType": "libretro",
    "status": "${RETROARCH_STATUS}",
    "executable": "retroarch",
    "arguments": ["-L", "fbneo_libretro.so"]
  }
]
EOF

echo "  ✓ Fichero de emuladores generado en ${EMULATORS_JSON}"
