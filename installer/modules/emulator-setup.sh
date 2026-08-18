#!/usr/bin/env bash
# ==============================================================================
#  EMUBOX MODULE - EMULATOR DETECTION & JSON REGISTRATION
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=lib/logging.sh
. "${SCRIPT_DIR}/lib/logging.sh"
# shellcheck source=lib/paths.sh
. "${SCRIPT_DIR}/lib/paths.sh"

log_step "Detectando y registrando emuladores instalados..."

check_binary() {
  command -v "$1" >/dev/null 2>&1
}

RETROARCH_STATUS="inactive"
check_binary "retroarch" && RETROARCH_STATUS="active"

DUCKSTATION_STATUS="inactive"
(check_binary "duckstation-qt" || check_binary "duckstation-nogui") && DUCKSTATION_STATUS="active"

PCSX2_STATUS="inactive"
(check_binary "pcsx2-qt" || check_binary "pcsx2") && PCSX2_STATUS="active"

MGBA_STATUS="inactive"
(check_binary "mgba-qt" || check_binary "mgba") && MGBA_STATUS="active"

FLYCAST_STATUS="inactive"
check_binary "flycast" && FLYCAST_STATUS="active"

cat > "${EMUBOX_EMULATORS_FILE}" << EOF
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
  }
]
EOF

log_ok "Archivo de perfiles de emulación generado en ${EMUBOX_EMULATORS_FILE}"
